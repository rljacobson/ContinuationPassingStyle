/*!

  The interpreter/virtual machine can raise an overflow exception or a division by zero exception.

*/

use std::rc::Rc;

use strum::Display;

use crate::{
  interpreter::{
    cps::{
      continuation::Answer,
      denotable_value::{DenotableValue, DValue}
    }
  }
};

#[derive(Copy, Clone, Eq, PartialEq, Display, Debug, Hash)]
pub enum Exception {
  Overflow,
  DivideByZero,
  InvalidAccess,    // Attempt to access a field of a non-`Record`
  // Bind,
  // Match,
  Undefined,
  IndexOutOfBounds, // Called `Nth` in [Appel], an invalid subscript.
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


#[derive(Copy, Clone, Display, Debug, Hash)]
pub enum InternalException {
  Undefined,
  WrongNumberOfParameters
}

pub fn raise_exception(exception: InternalException) {
  eprint!("Internal exception raised:: {}", exception);
}

pub fn raise_exception_msg(exception: InternalException, msg: &str) {
  eprint!("Internal exception raised:: {}: {}", exception, msg);
}
