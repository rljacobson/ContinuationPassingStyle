/*

*/

use std::rc::Rc;

use crate::{
  interpreter::{
    Location,
    Variable,
    VariableList,
    cps::{
      continuation::{Answer, Continuation, RawContinuation},
      denotable_value::{DenotableFunction, DValue, DValueList, resolve_field},
      store::AccessPath
    },
    environment::Environment,
    exception::Exception,
    primitive_op::PrimitiveOp,
    value::{Value, ValueList}
  }
};
use crate::interpreter::cps::continuation::ContinuationList;
use crate::interpreter::environment::RcEnvironment;


pub type CExp = Box<ContinuationExpression>;
pub type FunctionDefinitionList = Vec<FunctionDefinition>;
pub type RcFunctionDefinition = Rc<FunctionDefinition>;
pub type RcFunctionDefinitionList = Rc<Vec<RcFunctionDefinition>>;

#[derive(Clone, Eq, PartialEq)]
pub struct FunctionDefinition {
  name             : Variable,
  formal_parameters: VariableList,
  body             : ContinuationExpression
}

#[derive(Clone, PartialEq, Eq)]
pub enum ContinuationExpression {
  /// In the expression `Record(vl, w, e)` the scope of `w` is just the expression `e`.
  Record {
    values    : Vec<(Value, Rc<AccessPath>)>,
    variable  : Variable,
    expression: CExp
  },

  /// In `Select(i,v,w,e)` the scope of `w` is just `e`.
  Select {
    location  : Location,
    value     : Value,
    variable  : Variable,
    expression: CExp,
  },

  /// In `Offset(i,v,w,e)` the scope of `w` is just `e`.
  Offset {
    location  : Location,
    value     : Value,
    variable  : Variable,
    expression: CExp,
  },

  /// `Apply` does not bind variables and thus needs no scope rule.
  /// The `Value` should be a label or variable representing a function.
  Apply {
    /// A label or variable representing a function
    function : Value,
    /// The arguments to the function
    arguments: ValueList,
  },

  /**
  Mutually recursive functions are defined using the Fix operator. The effect of evaluating FIX
  (f⃗,E) is just to define the functions in the list f⃗ and then to evaluate E.

  In `Fix([(v,[w1, w2, ...], b)], e)` the scope of `wi` is just `b`,
  and the scope of `v` includes exactly `b` and `e`. This generalizes
  for a mutually recursive function definition: In the expression

  ```text
  Fix([ (f_1, [v11, v12, ..., v1m1 ], B1),
        (f_2, [v21, v22, ..., v2m2 ], B2),
        ···
        (f_n, [vn1, vn2, ..., vnmn ], Bn)], E)
  ```

  the scope of each `fi` includes all of the `Bj` and `E`; the scope
  of `vij` is just `Bi`.
  */
  Fix {
    function_defs: RcFunctionDefinitionList,
    expression   : CExp
  },

  /// `Switch` does not bind variables and thus needs no scope rule.
  Switch {
    value: Value,
    arms : Vec<CExp>
  },

  /// In the expression `PrimitiveOp(p, vl, [w], [e1, e2, ...])` the scope
  /// of `w` is the expressions `e1`, `e2`, ... .
  PrimitiveOp {
    operation  : PrimitiveOp,
    values     : ValueList,
    variables  : VariableList,
    expressions: Vec<CExp>,
  }
}

impl ContinuationExpression {
  pub fn evaluate(self, environment: Environment) -> Answer{
    match self {

      ContinuationExpression::Record { values, variable, expression } => {

        // Convert each `Value` to a `DValue`, and Resolve any access paths to the value it
        // ultimately points to.
        let d_values: DValueList = values.iter().map(
            | (value, access_path) | {
              let d_value = environment.value_to_denotable_value(value);
              resolve_field(d_value, access_path.clone())
            }
          ).collect();

        let record = DValue::Record {
            values: d_values,
            idx: 0
          };

        let new_environment = environment.bind(variable.clone(), record);

        expression.evaluate(new_environment)

      }

      ContinuationExpression::Select {
        location  : i,
        value     : v_value,
        variable  : w_variable,
        expression: e_cexp
      } => {
        if let DValue::Record {values, idx} = environment.value_to_denotable_value(&v_value) {
          let new_environment = environment.bind(w_variable.clone(), values[i+idx].clone());
          e_cexp.evaluate(new_environment)
        } else {
          Exception::InvalidAccess.as_answer()
        }
      }

      ContinuationExpression::Offset {
        location  : i,
        value     : v_value,
        variable  : w_variable,
        expression: e_cexp
      } => {
        if let DValue::Record {values, idx} = environment.value_to_denotable_value(&v_value) {
          let bind_value = DValue::Record{values, idx:i+idx};
          let new_environment = environment.bind(w_variable.clone(), bind_value);
          e_cexp.evaluate(new_environment)
        } else {
          Exception::InvalidAccess.as_answer()
        }
      }

      ContinuationExpression::Apply {
        function : f_value, // A label/variable bound to a function.
        arguments: l_values
      } => {
        if let DValue::Function(denotable_function) = environment.value_to_denotable_value(&f_value) {
          let parameters = l_values.iter()
                                   .map(|x|environment.value_to_denotable_value(x))
                                   .collect();
          denotable_function(parameters) // : Answer
        } else {
          Exception::Undefined.as_answer()
        }
      }

      // False positive for "Binding `fl_list` never used."
      ContinuationExpression::Fix {
        function_defs: fl_list,
        expression   : e_cexp
      } => {
        /**
          The  functions  `h` and `g` defined below are mutually recursive. They cannot
          be defined as closures, because they would both have to capture each other,
          but the second closure wouldn't exist yet for the first closure to capture.

          Thus `h` and `g` must be inner functions. But inner functions cannot capture
          variables in the ambient environment, and `h` and `g` want to both capture
          `fl_list`. The solution is to have `h` and `g` take a `&mut` to `fl_list`.

          The function h defines an individual function; it takes an environment r1 and a function
          definition (f,vl,b), where f is the function name, vl is the list of formal parameters,
          and b is the function body. The result is a function (fn al => ...) that takes a list
          of actual parameters al, and augments r1 in two ways: First it applies g to redefine
          all the (mutually recursive) functions in fl, and then it binds the actual parameters
          to the formal parameters. The resulting environment is then used to evaluate the body b.

          ~~This function captures `fl_list` indirectly through `g` (defined below).~~
        */
        fn h(
             r1_environment: RcEnvironment,
             function_def  : RcFunctionDefinition,
             fl_list       : RcFunctionDefinitionList
        ) -> DValue
        {
          let continuation: Rc<RawContinuation> =
            Rc::new(move
              | actual_parameters, store | {
                let bound_r1_environment = g(r1_environment.clone(), fl_list.clone());
                let new_environment =
                    bound_r1_environment.bindn(&function_def.formal_parameters, actual_parameters);

                (function_def.body.clone().evaluate(new_environment))(store)
              }
            );

          DValue::Function(DenotableFunction{f: continuation})
        }
        /// The function `g` takes an environment `r` as an argument and returns `r` augmented
        /// by binding all the function names (map #1 fl) to the function bodies (map (h r) fl).
        /// ~~This function captures `fl_list`.~~
        fn g(r: RcEnvironment, fl_list: RcFunctionDefinitionList) -> Environment {
          let function_names: VariableList = fl_list.iter().map(|fd | fd.name.clone() ).collect();

          let function_values = fl_list.iter()
                                       .map(|fd| h(r.clone(), fd.clone(), fl_list.clone()))
                                       .collect::<DValueList>();
          r.bindn(
            &function_names,
            &function_values
          )
        }



        e_cexp.evaluate(g(Rc::new(environment), fl_list))
      }

      ContinuationExpression::Switch {
        value,
        arms: el_cexp_list
      } => {
        if let DValue::Integer(i) = environment.value_to_denotable_value(&value){
          el_cexp_list[i as usize].clone().evaluate(environment)
        } else {
          Exception::IndexOutOfBounds.as_answer()
        }
      }

      /*
      To evaluate a primitive operator, it is first necessary to extract all the atomic arguments
      from the environment (map (V env) vl). Then the cexp arguments are all converted to functions
      of type dvalue list -> store -> answer. The list of atomic arguments and the list of
      continuations are handed (along with the operator p) to the evalprim function, which performs
      the appropriate operation and then selects one of the continuations to hand the result to.
      */
      ContinuationExpression::PrimitiveOp {
        operation  : p,
        values     : vl,
        variables  : wl,
        expressions: el
      } => {
        let d_values = vl.iter().map(|v| (&environment).value_to_denotable_value(v)).collect();
        let mut continuations: ContinuationList = Vec::new();
        let rc_environment = Rc::new(environment);
        let rc_wl = Rc::new(wl);
        for c in el.into_iter() {
          // let ce: ContinuationExpression = *c;
          let environment = rc_environment.clone();
          let wl = rc_wl.clone();
          continuations.push(
            Continuation {
              f: Rc::new(move |parameters, store| {
                c.clone().evaluate(environment.bindn(&wl, &parameters)).call_once((store,))
              })
            }
          )
        }

        p.evaluate(d_values, continuations)
      }

    }
  }
}

