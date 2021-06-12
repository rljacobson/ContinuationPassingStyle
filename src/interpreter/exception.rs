/*!

  The interpreter/virtual machine can raise an overflow exception or a division by zero exception.

*/

use strum::Display;

use super::denotable_value::DenotableValue;
use crate::interpreter::denotable_value::DValue;
use crate::interpreter::continuation::Answer;
use std::rc::Rc;

// TODO: This is ugly. Find a better way.
const OVERFLOW_EXCEPTION: DenotableValue = DenotableValue::Integer(0);
const DIVIDE_BY_ZERO_EXCEPTION: DenotableValue = DenotableValue::Integer(0);

#[derive(Copy, Clone, Display, Debug, Hash)]
pub enum Exception {
  Overflow,
  DivideByZero,
  InvalidAccess,      // Attempt to access a field of a non-`Record`
  Undefined,          // Exception for interpreter/host code, not the interpreted program.
  // Bind,
  // Match,
  IndexOutOfBounds,   // Called `Nth` in [Appel], an invalid subscript.
}

impl Exception {
  pub fn as_denotable_value(self) -> DenotableValue {
    DValue::Exception(self)
  }

  pub fn as_answer(&self) -> Answer {
    Answer{
      f: Rc::new(
        | parameters, store | {
          if let [DValue::Exception(e)] = parameters[..] {
            store.raise_exception(e)
          } else{
            unreachable!("Internal error: could not unpack exception.")
          }
        }
      ),
      parameters: vec![self.as_denotable_value()]
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
