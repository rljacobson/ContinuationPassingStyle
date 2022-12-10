#![allow(dead_code)]
/*!

  A Continuation is a function that takes a `DValueList` and a `Store` and produces an answer:

  ```ml
  cont: dvalue list -> store -> answer
  ```

  A `ContinuationExpression` is a thing that can be turned into a `Continuation`.

*/


use std::rc::Rc;
use std::cmp::Eq;


use crate::{
  interpreter::{
    cps::{
      denotable_value::DenotableValueList,
      store::Store
    }
  }
};

use super::{
  environment::Environment,
  exception::Exception,
  primitive_op::PrimitiveOp,
  value::{
    Value,
    ValueList,
  },
  Variable,
  VariableList
};

// Defined below
pub type Parameters = DenotableValueList; // todo: Reference to `DValueList`, or `Rc`, or...?
pub type RawContinuation = dyn Fn(&Parameters, &Store) -> Answer;
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
#[derive(Clone)]
pub struct Continuation{
  pub f: Rc<RawContinuation>, // (parameters, store) -> answer
}

impl Eq for Continuation {}

impl PartialEq for Continuation {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.f, &other.f)
  }
}

// region impl Fn<DValueList> for Continuation
// impl Fn<(Parameters,)> for Continuation {
//   /// A `Continuation` `c` is callable as `c(parameters)` and returns an `Answer`. To
//   /// call it with both parameters and a store, call the wrapped `RawContinuation` as
//   /// `c.f(parameters, store)`.
//   extern "rust-call" fn call(&self, parameters: (Parameters,)) -> Self::Output {
//     self.call_once(parameters)
//   }
// }
//
// impl FnMut<(Parameters,)> for Continuation {
//   /// Delegates to `Fn::call`
//   extern "rust-call" fn call_mut(&mut self, parameters: (Parameters,)) -> Self::Output {
//     self.call_once(parameters)
//   }
// }

impl FnOnce<(Parameters,)> for Continuation {
  type Output = Answer;

  /// Delegates to `Fn::call`
  extern "rust-call" fn call_once(self, parameters: (Parameters,)) -> Self::Output {
    // self.call(parameters)
    Answer{
      f: self.f.clone(),
      parameters: parameters.0
    }
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
#[derive(Clone)]
pub struct Answer {
  pub(crate) f: Rc<RawContinuation>,
  pub(crate) parameters: Parameters
}

// region impl Fn<Store> for Answer
impl Fn<(&Store,)> for Answer {
  /// An `Answer` `c` is callable as `c(store)` and returns an `Answer`.
  extern "rust-call" fn call(&self, store: (&Store,)) -> Self::Output {
    (self.f)(&self.parameters, store.0)
  }
}

impl FnMut<(&Store,)> for Answer {
  /// Delegates to `Fn::call`
  extern "rust-call" fn call_mut(&mut self, store: (&Store,)) -> Self::Output {
    self.call(store)
  }
}

impl FnOnce<(&Store,)> for Answer {
  type Output = Answer;

  /// Delegates to `Fn::call`
  extern "rust-call" fn call_once(self, store: (&Store,)) -> Self::Output {
    self.call(store)
  }
}

// endregion
