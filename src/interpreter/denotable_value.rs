/*!

  A value is *denotable* if it can be held in one machine word (as a pointer into the
  heap or as a single-precision integer/float). These values may be bound to
  variables, passed as parameters, and stored in data structures.

  A [`Value`] is a primitive non-composite value, which may or may not be denotable.
  A [`DenotableValue`], or `DValue` for short, is a (possibly composite) value that
  can be denoted, that is,

*/


use std::{ops::Range};
use std::rc::Rc;

use crate::interpreter::{Integer, Location, Real};
use crate::interpreter::continuation::Continuation;

use super::{
  arbitrarily,
  exception::Exception,
  store::Store,
  value::ValueList
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

const ZERO: DenotableValue = DValue::Integer(0i32);
pub(crate) const EMPTY: DenotableValueList = vec![];



#[derive(Clone, Debug, Hash)]
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
  /// A `ByteArray` differs from a `String` in that a `ByteArray` is mutable.
  ByteArray(Range<Location>),

  /// An `Array` holds the range of indices into the `Store` where the array values are stored.
  /// Note that the `DValue::Array` does not hold the values themselves, just the indices.
  Array(Range<Location>),
  /// An array of `Integers`.
  UnboxedArray(Range<Location>),

  /// A `DenotableFunction` is the same thing as a `Continuation`..
  Function(DenotableFunction),

  Exception(Exception)
}

/// This definition of equality models the semantics of pointer equality. The instances of `arbitrarily` correspond to
/// variant instances that may be pointers to the same memory location (equal) or pointers to two different memory
/// locations regardless of whether the value is the same (not equal).
impl PartialEq for DValue {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (
        DValue::Record { values: values_lhs, idx: idx_lhs },
        DValue::Record { values: values_rhs, idx: idx_rhs }
      ) => {
        arbitrarily(
          values_lhs == values_rhs && (idx_lhs == idx_rhs),
          false
        )
      },

      (DValue::Integer(a), DValue::Integer(b)) => {
        a == b
      },

      (DValue::Real(a), DValue::Real(b)) => {
        arbitrarily(a == b, false) // || (a.is_nan() && b.is_nan())
      },

      (DValue::String(a), DValue::String(b)) => {
        arbitrarily(a == b, false)
      },

      (DValue::Function(_), DValue::Function(_)) => {
        raise_exception(Exception::Undefined);
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


/// An access path is a selection chain through linked `DValue::Record`s terminating at a
/// non-`Record` `DValue`.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AccessPath{
  Offset(Location),
  Select { offset: Location, access_path: Rc<AccessPath> }
}

/// Accesses the value of the field pointed to by an `AccessPath`. This is function `F` in [Appel]
pub fn resolve_field(value: DValue, access_path: Rc<AccessPath>) -> DValue {
  match (value, *access_path) {

    (x, AccessPath::Offset(0)) => x,

    (DValue::Record {values, idx}, AccessPath::Offset(j))
      => DValue::Record {values, idx: idx + j},

    (DValue::Record {values, idx}, AccessPath::Select {offset, access_path})
      => resolve_field(values[idx + offset].clone, access_path.clone()),

    (_, _) => {
      DValue::Exception(Exception::InvalidAccess)
    }

  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
