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

pub mod exception;
pub mod primitive_op;
pub mod value;
pub mod environment;
pub mod continuation_expression;
pub mod cps;

use std::collections::HashMap;
use std::rc::Rc;

use ordered_float::OrderedFloat;

use crate::{
  interpreter::{
    continuation_expression::ContinuationExpression,
    environment::Environment,
    exception::{InternalException, raise_exception},
    cps::{
      continuation::Answer,
      denotable_value::DenotableValueList
    }
  }
};


#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Variable{
  name: String
  // TODO: If `name` is the only field, define `Variable(String)` instead.
  // TODO: Use interned strings.
}

pub type VariableList = Vec<Variable>;

pub type Integer     = i64;
pub type IntegerList = Vec<Integer>;
pub type Real        = OrderedFloat<f32>;
pub type Location    = usize;

/// "arbitrarily" selects one of the two provided alternatives.
// Todo: It's not clear what this function should do.
pub fn arbitrarily(a: bool, b: bool) -> bool {
  a
}


/// The entry point of the interpreter, `evaluate` takes a `VariableList`, a
/// `ContinuationExpression`, and a list of values to be bound to the corresponding variables in
/// the`VariableList`, and returns the denotation of the expression in the resulting environment.
pub fn evaluate(
  mut variables: VariableList,
  mut values   : DenotableValueList,
  expression   : ContinuationExpression,
) -> Answer
{
  if variables.len() != values.len() {
    raise_exception(InternalException::WrongNumberOfParameters);
    panic!();
  }

  let bindings = variables.drain(..)
                          .zip(values.drain(..))
                          .collect::<HashMap<_, _>>();
  let environment = Environment::with_bindings(bindings);

  expression.evaluate(environment)
}
