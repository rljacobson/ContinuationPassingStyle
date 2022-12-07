#![allow(dead_code)]
/*!

  A Continuation is a function that takes a `DValueList` and a `Store` and produces an answer:

  ```ml
  cont: dvalue list -> store -> answer
  ```

  A `ContinuationExpression` is a thing that can be turned into a `Continuation`.

*/


use std::rc::Rc;

use super::{
  denotable_value::{
    AccessPath,
    DenotableValue,
    DenotableValueList,
    DValue,
    DValueList,
    resolve_field
  },
  denotable_value::DenotableFunction,
  environment::Environment,
  exception::Exception,
  Location,
  primitive_op::PrimitiveOp,
  store::Store,
  value::{
    Value,
    ValueList,
    Variable,
  },
  VariableList
};

// Defined below
pub type Parameters = DValueList;               // todo: Reference to `DValueList`, or `Rc`, or...?
pub type RawContinuation = dyn Fn(Parameters, Store) -> Answer;
pub type ContinuationList = Vec<Continuation>;  // Defined below.


/// A Continuation is a wrapper fpr:
/// ```rust
/// Rc< dyn
///   Fn(parameters: DValueList, store: Store) -> Answer
/// >
/// ```
/// The wrapper allows currying of continuations to produce `Answer`s. A `Continuation` `c`
/// is callable as `c(parameters)` and returns an `Answer`. To call it with both parameters
/// and a store, call the wrapped `RawContinuation` as `c.f(parameters.clone(), store)`.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Continuation{
  pub f: Rc<RawContinuation>, // (parameters, store) -> answer
}

// region impl Fn<DValueList> for Continuation
/// A `Continuation` `c` is callable as `c(parameters)` and returns an `Answer`. To
/// call it with both parameters and a store, call the wrapped `RawContinuation` as
/// `c.f(parameters, store)`.
impl<DValueList> Fn<DValueList> for Continuation {
  extern "rust-call" fn call(&self, parameters: Parameters) -> Self::Output {
    // Do stuff.
    Answer{
      f: self.f.clone(),
      parameters
    }
  }
}

/// Delegates to `Fn::call`
impl<Parameters> FnMut<Parameters> for Continuation {
  extern "rust-call" fn call_mut(&mut self, parameters: Parameters) -> Self::Output {
    self.call(parameters)
  }
}

/// Delegates to `Fn::call`
impl<Parameters> FnOnce<Parameters> for Continuation {
  type Output = Answer;

  extern "rust-call" fn call_once(self, parameters: Parameters) -> Self::Output {
    self.call(parameters)
  }
}

// endregion

// todo: Is it actually worth using a struct for `Answer`?
/**
Stands in for the result of the execution of a program. It is morally a curried Continuation.
Evaluating an `Answer` with a `Store` produces another `Answer`. A struct is used instead
of a closure, because it allows us to un-curry an answer to obtain (an equivalent of)
the `Continuation` that produced it as well as provide debugging/visualization utilities.

`Answer`s are cheaply cloneable.
*/
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Answer {
  pub(crate) f: Rc<RawContinuation>,
  pub(crate) parameters: Parameters
}

// region impl Fn<Store> for Answer
/// An `Answer` `c` is callable as `c(store)` and returns an `Answer`.
impl<Store> Fn<Store> for Answer {
  extern "rust-call" fn call(&self, store: Store) -> Self::Output {
    self.f(self.parameters.clone(), store)
  }
}

/// Delegates to `Fn::call`
impl<Store> FnMut<Store> for Answer {
  extern "rust-call" fn call_mut(&mut self, parameters: Parameters) -> Self::Output {
    self.call(parameters)
  }
}

/// Delegates to `Fn::call`
impl<Store> FnOnce<Store> for Answer {
  type Output = Answer;

  extern "rust-call" fn call_once(self, parameters: Parameters) -> Self::Output {
    self.call(parameters)
  }
}

// endregion
