/*!

  A `Store` is a data store, i.e. a mapping of addresses to denotable values.

*/

use std::rc::Rc;

use crate::interpreter::{Integer, IntegerList, Location};
use crate::interpreter::exception::Exception;

use super::{
  continuation::Answer,
  denotable_value::{
    DenotableValueList,
    DValue
  },
};

/// From [Appel, p.25]:
///   The store (whose type is `(loc*(loc->dvalue)*(loc->int)))` has three components: the next unused location, a mapping
///   from locations to denotable values, and a mapping from locations to integers.
///
/// Instead of using a global variable for the current exception handler as in \[Appel], we keep it
/// with the `Store`.
#[derive(Clone)]
pub struct Store{
  pub(crate) next_unused_address: Location,
  pub(crate) exception_handler: Location,
  values: DenotableValueList, // "mapping" from locations to denotable values
  integer_values: IntegerList,
}

impl Store{
  // TODO: Should `fetch` return a clone of the `DValue`?
  /// Returns a reference to the (non `Integer`) `DValue` at `Location idx` in the `Store`. For `Integer`s, use
  /// `fetch_integer`.
  pub fn fetch(&self, idx: Location) -> &DValue {
    // TODO: Determine if bounds checking is necessary here. I suspect that index out of bounds is a bug in the
    //       interpreter, not in the user program.
    self.values.get(idx).unwrap()
  }

  // TODO: Should this return a `&DValue`?
  pub fn fetch_integer(&self, idx: Location) -> DValue {
    let n = *self.integer_values.get(idx).unwrap();
    DValue::Integer(n)
  }

  /// Produces a new `Store` which is identical to the current store except that the value at `Location idx` has value
  /// `value`.
  pub fn update(&self, idx: Location, value: DValue) -> Store {
    let mut updated_store: Store = self.clone();

    // Accommodate the integer GC optimization.
    if let DValue::Integer(i) = value {
      updated_store.integer_values[idx] = i;
    } else {
      updated_store.values[idx] = value;
    }

    return updated_store;
  }

  /// Produces a new `Store` which is identical to the current store except that the integer at `Location idx` has value
  /// `value`. If the integer is wrapped in a `DValue` you may use `update`.
  pub fn update_integer(&self, idx: Location, value: Integer) -> Store {
    let mut updated_store: Store = self.clone();
    updated_store.integer_values[idx] = value;
    return updated_store;
  }

  /// Uses the `Store`'s exception handler to handle the given exception.
  pub fn raise_exception(&self, exception: Exception) -> Answer{
    eprintln!("Exception raised: {:?}", &exception);

    if let DValue::Function(continuation) = &self.values[self.exception_handler] {
      (continuation.f).call((&vec![exception.into()], self))
    } else {
      // Todo: Handle exceptions raised by the interpreter rather than the program being
      //       interpreted.
      panic!("The exception handler pointer does not point to a function.")
    }
  }

}

/// An access path is a selection chain through linked `DValue::Record`s terminating at a
/// non-`Record` `DValue`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AccessPath{
  Offset(Location),
  Select { offset: Location, access_path: Rc<AccessPath> }
}
