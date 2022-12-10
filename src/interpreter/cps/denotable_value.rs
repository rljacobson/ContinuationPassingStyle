/*!

  A [`Value`] is a primitive non-composite value, which may or may not be denotable.
  A [`DenotableValue`], or `DValue` for short, is a (possibly composite) value that
  can be denoted. A value is *denotable* if it can be held in one machine word (as
  a pointer into the heap or as a single-precision integer/float). These values
  may be bound to variables, passed as parameters, and stored in data structures.

  From \[Appel]:

  A denotable value can be a `RECORD` containing a list of denotable values. It is
  possible to point into the middle of a record, not just at the beginning, so the
  denotable value for records also has an integer indicating the offset into the record.

  `ARRAY` values are represented using the store, so their contents can be modified
  after they are created. An array of length n is represented in the semantics as an
  arbitrary list of locations, though presumably in an implementation the locations
  will be consecutive. Note that records are not kept in the store, and thus record
  values are “pure” and cannot be modified once created. There are two kinds of arrays:
  `ARRAY`s can contain arbitrary denotable values, and `UARRAY`s can contain only integers.

  A denotable value can be a `STRING` of characters or a `BYTEARRAY`. The same operations
  apply to strings and byte arrays, except that byte arrays can be stored into (modified)
  and strings cannot. The elements of strings and byte arrays must be small (byte-sized)
  integers, in contrast to the elements of `UARRAY`s, which can be larger (word-sized)
  integers, and elements of `ARRAY`s, which can be of any type (including integer).

*/


use std::ops::Range;
use std::rc::Rc;
use crate::{
  interpreter::{
    exception::{Exception, InternalException, raise_exception},
    Integer,
    Location,
    Real,
    arbitrarily,
    value::ValueList
  }
};

use super::{
  continuation::Continuation,
  store::{Store, AccessPath},
};

// use saucepan::{Span, Source, Sources, ByteIndex, ByteOffset};

/// A `DenotableFunction` has the form
///
/// ```rust
/// pub type RawContinuation
///     = dyn Fn(Parameters, Store) -> Answer;
///
/// pub struct Continuation{
///   pub f: Rc<RawContinuation>,
/// }
/// ```
pub type DenotableFunction = Continuation;

// pub type DenotableValueList = Rc<Vec<DenotableValue>>; // Defined below
pub type DenotableValueList = Vec<DenotableValue>; // Defined below

// Shorthand
pub type DValue = DenotableValue; // Defined below
pub type DValueList = DenotableValueList;

pub(crate) const ZERO: DenotableValue = DValue::Integer(0);
pub(crate) const EMPTY: DenotableValueList = vec![];


#[derive(Clone)]
pub enum DenotableValue {
  Record{
    values: DValueList, // A list of denotable values.
    idx: Location
  }, // A structure.

  Integer(Integer),

  Real(Real),

  /// `String`s are immutable.
  String(String),

  // TODO: Are these arrays literally just arrays?

  /// A `ByteArray` differs from a `String` in that a `ByteArray` is mutable. An
  /// `*Array` holds the range of indices into the `Store` where the array values are
  /// stored, the values themselves.
  ByteArray(Range<Location>),

  /// An `Array` holds the range of indices into the `Store` where the array values
  /// are stored. The store holds pointers to DenotableValues at these indices,
  /// i.e. the values are boxed. Thus, an `Array` can hold any `DenotableValue`.
  Array(Range<Location>),

  /// An array of `Integers`. An `UnboxedArray` holds the range of indices into the `Store` at
  /// which the integers are stored. The store holds the _values_ of the integers at these
  /// indices. (See `ByteArray`.)
  UnboxedArray(Range<Location>),

  /// A `DenotableFunction` is a type alias for `Continuation`.
  Function(DenotableFunction),

  Exception(Exception)
}

impl PartialEq for DValue {
  /// This definition of equality models the semantics of pointer equality. The
  /// instances of `arbitrarily` correspond to variant instances that may be
  /// pointers to the same memory location (equal) or pointers to two different
  /// memory locations regardless of whether the value is the same (not equal).
  ///
  /// Note: This is the `eq` function of \[Appel].
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (
        DValue::Record { values: values_lhs, idx: idx_lhs },
        DValue::Record { values: values_rhs, idx: idx_rhs }
      ) => {
        // Strings and records are "pure values" and cannot be reliably compared by pointer.
        //
        // "But, in order to accommodate implementations as described in the previous paragraph,
        // testing the equality of two records is permitted to be 'conservative': If the two records
        // are unequal, eq will return false, but if they are equal, eq might return true or false."
        //
        // If the pointers are the same, the values are the same. If the pointers are different,
        // the values might be either the same or different.
        arbitrarily(
          values_lhs == values_rhs && (idx_lhs == idx_rhs),
          false
        )
      },

      (DValue::Integer(a), DValue::Integer(b)) => {
        a == b
      },

      (DValue::Real(a), DValue::Real(b)) => {
        // The `arbitrarily` function is meant to only apply to pointer types. Since real
        // numbers are immediate values in our implementation, we don't need it here. (In
        // \[Appel], real numbers are boxed.
        //
        // Also, all NaN values are equal to each other for `OrderedFloat`, so it is not a
        // special case.
        // arbitrarily(a == b, false) // || (a.is_nan() && b.is_nan())
        a == b
      },

      (DValue::String(a), DValue::String(b)) => {
        // Strings and records are "pure values" and cannot be reliably compared by pointer.
        // See the comment on `DValue::Record` equality.
        arbitrarily(a.as_ptr() == b.as_ptr(), false)
      },

      (DValue::ByteArray(range_a), DValue::ByteArray(range_b))
      | (DValue::Array(range_a), DValue::Array(range_b))
      | (DValue::UnboxedArray(range_a), DValue::UnboxedArray(range_b))
      => {
        // Two `*Array`s of the same type are equal if and only if they represent the same range of
        // indices in the `Store`. Thus, two distinct `*Array`s might hold the same values and
        // yet still be unequal.
        range_a == range_b
      }

      (DValue::Function(_), DValue::Function(_)) => {
        // Todo: What should we do when we want the interpreter itself to raise an exception as
        //       opposed to the user program raising an exception?
        raise_exception(InternalException::Undefined);
        false
      }

      _ => {
        // Exceptions and unlike variants are incomparable.
        false
      }
    } // end match
  }
}

impl From<Exception> for DValue {
  fn from(exception: Exception) -> Self {
    DValue::Exception(exception)
  }
}

impl From<Integer> for DValue {
  fn from(i: Integer) -> Self {
    DValue::Integer(i)
  }
}

impl From<Real> for DValue {
  fn from(r: Real) -> Self {
    DValue::Real(r)
  }
}

impl From<String> for DValue {
  fn from(value: String) -> Self {
    DValue::String(value)
  }
}

/// Accesses the value of the field pointed to by an `AccessPath`. This is function `F` in [Appel].
// Todo: Do we return a value or a reference to a value?
pub fn resolve_field(value: DValue, access_path: Rc<AccessPath>) -> DValue {
  match (value, access_path.as_ref()) {

    (x, AccessPath::Offset(0)) => x,

    (DValue::Record {values, idx}, AccessPath::Offset(j))
      => DValue::Record {values, idx: idx + j},

    (DValue::Record {values, idx}, AccessPath::Select {offset, access_path})
      => resolve_field(values[idx + offset].clone(), access_path.clone()),

    (_, _) => {
      DValue::Exception(Exception::InvalidAccess)
    }

  }
}


