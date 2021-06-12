#![allow(dead_code)]
/*!

  A Continuation is a function that takes a `DValueList` and a `Store` and produces an answer:
    `cont: dvalue list -> store -> answer`

  A `ContinuationExpression` is a thing that can be turned into a `Continuation`.

*/


use std::rc::Rc;

use super::{
  primitive_op::PrimitiveOp,
  value::{
    AccessPath,
    Value,
    ValueList,
    Variable,
  },
  denotable_value::{
    AccessPath,
    DValueList,
    DenotableValueList,
    DenotableValue::{Record, Exception},
    resolve_field,
    DValue
  },
  store::Store,
  environment::Environment,
  exception::Exception,
  VariableList,
  Location,
  denotable_value::DenotableFunction
};

pub type CExp = Rc<ContinuationExpression>;     // Defined below
pub type Parameters = DValueList;               // todo: Reference to `DValueList`, or `Rc`, or...?
pub type RawContinuation = dyn Fn(Parameters, Store) -> Answer;
pub type ContinuationList = Vec<Continuation>;  // Defined below.

struct FunctionDefinition {
  name: Variable,
  formal_parameters: VariableList,
  body: Box<DenotableFunction>
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ContinuationExpression {
  /// In the expression `Record(vl, w, e)` the scope of `w` is just the expression `e`.
  Record(Vec<(Value, Rc<AccessPath>)>, Variable, CExp),

  /// In `Select(i,v,w,e)` the scope of `w` is just `e`.
  Select(Location, Value, Variable, CExp),

  /// In `Offset(i,v,w,e)` the scope of `w` is just `e`.
  Offset(Location, Value, Variable, CExp),

  /// `Apply` does not bind variables and thus needs no scope rule.
  Apply(Value, ValueList),

  /**
  In `Fix([(v,[w1, w2, ...], b)], e)` the scope of `wi` is just `b`,
  and the scope of `v` includes exactly `b` and `e`. This generalizes
  for a mutually recursive function definition: In the expression

  ```text
  Fix([ (f1, [v11, v12, ..., v1m1 ], B1),
        (f2, [v21, v22, ..., v2m2 ], B2),
        ···
        (fn, [vn1, vn2, ..., vnmn ], Bn)], E)
  ```

  the scope of each `fi` includes all of the `Bj` and `E`; the scope
  of `vij` is just `Bi`.
  */
  Fix(Vec<FunctionDefinition>, CExp),

  /// `Swtich` does not bind variables and thus needs no scope rule.
  Switch(Value, Vec<CExp>),

  /// In the expression `PrimitiveOp(p, vl, [w], [e1, e2, ...])` the scope
  /// of `w` is the expressions `e1`, `e2`, ... .
  PrimitiveOp(PrimitiveOp, ValueList, VariableList, Vec<CExp>)
}

impl ContinuationExpression {
  pub fn evaluate(self, environment: Environment) -> DValue{
    match self {

      ContinuationExpression::Record(values, variable, continuation_expression) => {

        // Convert each `Value` to a `DValue`, and Resolve any access paths to the value it
        // ultimately points to.
        let d_values = values.iter().map(
            | (value, access_path) | {
              let d_value = environment.value_to_denotable_value(*value);
              resolve_field(d_value, access_path.clone())
            }
          ).collect();

        let record = Record {
            values: d_values,
            idx: 0
          };

        let new_environment = environment.bind(variable, record);

        continuation_expression.evaluate(new_environment)

      }

      ContinuationExpression::Select(i, v_value, w_variable, e_cexp) => {
        if let DValue::Record {values, idx} = environment.value_to_denotable_value(v_value) {
          let new_environment = environment.bind(w_variable, *values[i+idx]);
          e_cexp.evaluate(new_environment)
        } else {
          DValue::Exception(Exception::InvalidAccess)
        }
      }

      ContinuationExpression::Offset(i, v_value, w_variable, e_cexp) => {
        if let DValue::Record {values, idx} = environment.value_to_denotable_value(v_value) {
          let bind_value = DValue::Record{values, idx:i+idx};
          let new_environment = environment.bind(w_variable, bind_value);
          e_cexp.evaluate(new_environment)
        } else {
          DValue::Exception(Exception::InvalidAccess)
        }
      }

      ContinuationExpression::Apply(f_value, l_values) => {
        if let DValue::Function(denotable_function) = environment.value_to_denotable_value(f_value){
          let parameters = l_values.iter().map(environment.value_to_denotable_value ).collect();
          denotable_function(parameters)
        } else {
          DValue::Exception(Exception::Undefined)
        }
      }

      ContinuationExpression::Fix(fl_list, e_cexp) => {

        /**
          The function h defines an individual function; it takes an environment r1 and a function
          definition (f,vl,b), where f is the function name, vl is the list of formal parameters,
          and b is the function body. The result is a function (fn al => ...) that takes a list
          of actual parameters al, and augments r1 in two ways: First it applies g to redefine
          all the (mutually recursive) functions in fl, and then it binds the actual parameters
          to the formal parameters. The resulting environment is then used to evaluate the body b.

          This function captures `fl_list` indirectly through `g` (defined below).
        */
        fn h(
             r1_environment: Environment,
             function_def: &FunctionDefinition,
        ) -> DValue {
          let bound_r1_environment = g(r1_environment);
          let f: RawContinuation =
            |actual_parameters: &ValueList | -> DValue{
              let new_environment =
                  bound_r1_environment.bindn(&function_def.formal_parameters, actual_parameters);
              function_def.body.evaluate(new_environment)
            };
          DValue::Function(DenotableFunction{f})
        }

        /// The function `g` takes an environment `r` as an argument and returns `r` augmented
        /// by binding all the function names (map #1 fl) to the function bodies (map (h r) fl).
        /// This function captures `fl_list`.
        fn g(r: Environment) -> Environment {
          let function_names = fl_list.iter().map(| triple | triple.0 ).collect();
          let function_bodies = fl_list.iter().map(|triple| h(r, triple)).collect::<Vec<DValue>>();
          r.bindn(
            function_names,
            function_bodies
          )
        };

        e_cexp.evaluate(g(environment))
      }

      ContinuationExpression::Switch(_, _) => {

      }

      ContinuationExpression::PrimitiveOp(_, _, _, _) => {

      }

    }
  }
}


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
  pub f: RawContinuation,
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
  pub(crate) f: RawContinuation,
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
