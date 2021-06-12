#![allow(dead_code)]

/*!

  This module implements the program of [Appel, Chapter 3].

  From [Appel, Chapter 3, p. 23]:

  > We might also want to treat this semantics as \[a] program that could serve
  > as an interpreter for the CPS language. For erroneous programs, the interpreter
  > will raise an... exception: either Undefined (declared here) or one of the...
  > exceptions Bind, Match, or Nth. This is not to be confused with a CPS program
  > invoking a CPS exception handler

*/

use  continuation::{Answer, ContinuationExpression};
use  denotable_value::{Answer, DValue, DValueList};
use  exception::Exception;
use  store::Store;
use value::VariableList;

pub mod denotable_value;
pub mod store;
pub mod exception;
pub mod primitive_op;
pub mod continuation;
pub mod value;
pub mod environment;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Variable{
  name: String
  // TODO: If `name` is the only field, define `Variable(String)` instead.
  // TODO: Use interned strings.
}

pub type VariableList = Vec<Variable>;


fn next_location(last_address: Location) -> Location {//-> Span<'static> {
  last_address + std::mem::size_of::<DValue>()
}

/* From [Appel, Chapter 1] I think?
// TODO: If `variables` are bound to  `values`, just use a `HashMap<Variable, DValue>` instead of two vectors.
pub fn eval(
  variables: VariableList,
  continuation: ContinuationExpression,
  dvalues: DValueList,
  store: Store
) -> Answer {
  Answer{}
}
*/

/*
The function `overflow` and `overflowr` plays in [Appel] has been inlined, because we leverage
the existing methods in `num::CheckedMul`, `num::CheckedAdd`, etc.

fn overflow(
  evaluate_number: Box<dyn FnOnce()->Option<Integer>>,
  values: DValueList,
  continuation: ContinuationExpression
) -> DValue {
  if let Some(n) = evaluate_number() {
    CExp(DValue::Integer(n))
  } else{
    raise_exception(DValue::Exception(Exception::Overflow))
  }
}
 */

// todo: How to implement this function?
/// Models unpredictability, a sort of Phi node.
fn arbitrarily<T>(lhs: T, _rhs: T) -> T {
  lhs
}

pub type Integer = i32;
pub type IntegerList = Vec<Integer>;
pub type Real = f32;
pub type Location = usize;

