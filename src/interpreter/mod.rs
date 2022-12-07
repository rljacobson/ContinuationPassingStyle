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

pub mod denotable_value;
pub mod store;
pub mod exception;
pub mod primitive_op;
pub mod continuation;
pub mod value;
pub mod environment;
mod continuation_expression;


#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Variable{
  name: String
  // TODO: If `name` is the only field, define `Variable(String)` instead.
  // TODO: Use interned strings.
}

// TODO: If `variables` are bound to  `values`, just use a `HashMap<Variable, DValue>` instead of two vectors.
pub type VariableList = Vec<Variable>;

pub type Integer = i32;
pub type IntegerList = Vec<Integer>;
pub type Real = f32;
pub type Location = usize;


/// "arbitrarily" selects one of the two provided alternatives.
pub fn arbitrarily(a: bool, b: bool) -> bool {
  a
}
