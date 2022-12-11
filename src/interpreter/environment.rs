/*!

An `Environment` stores the bindings of variables to values.

*/

use std::collections::HashMap;
use std::ops::{Deref, Index};
use std::rc::Rc;

use crate::interpreter::cps::denotable_value::{DValue, DValueList};

use crate::interpreter::{
  Variable,
  VariableList,
  value::Value,
};

pub type Bindings = HashMap<Variable, DValue>;
pub type RcEnvironment = Rc<Environment>;

#[derive(Clone)]
pub struct Environment {
  pub(crate) bindings: Rc<Bindings>
}


impl Environment {

  pub fn new() -> Self {
    Environment{
      bindings: Rc::new(HashMap::new())
    }
  }

  pub fn with_bindings(bindings: Bindings) -> Environment {
    Environment{
      bindings: Rc::new(bindings)
    }
  }

  /// Creates an environment in which `variable` is bound to `value`, consuming `self`.
  pub fn bind(&self, variable: Variable, value: DValue) -> Environment{
    // Avoid making a new environment if `variable` is already bound to `value`.
    if let Some(found) = self.get(&variable){
      if *found == value {
        return self.clone();
      }
    }

    // The following will always clone, because `self` will always have a strong reference to
    // `self.bindings`. Since there is no way to know if self will still be needed afterwards, we
    // cannot steal the bindings vector from this environment.
    let mut new_environment: Environment = self.deep_copy();
    unsafe {
      Rc::get_mut_unchecked(&mut new_environment.bindings).insert(variable, value);
    }
    new_environment
  }

  /// Creates a copy of the environment in which the given list of variables and values are bound.
  /// Unlike bind, does not optimize the case that the variables are already bound.
  pub fn bindn(&self, variables: &VariableList, values: &DValueList) -> Environment {
    let mut new_environment = self.deep_copy();
    new_environment.extend(variables.iter().cloned().zip(values.iter().cloned()));
    new_environment
  }

  pub fn deep_copy(&self) -> Environment {
    let new_bindings: Bindings = self.bindings.deref().clone();
    Environment{ bindings: Rc::new(new_bindings) }
  }

  // Creates an environment in which `variable` is free.
  pub fn unbind(&self, variable: &Variable) -> Environment{
    // Avoid making a new environment if `variable` is already unbound.
    if !self.bindings.contains_key(&variable){
      return self.clone();
    }

    let mut new_environment = self.deep_copy();
    unsafe {
      Rc::get_mut_unchecked(&mut new_environment.bindings).remove(&variable);
    }
    new_environment
  }

  pub fn get(&self, variable: &Variable) -> Option<&DValue> {
    self.bindings.get(variable)
  }

  /// This method is trivial for number variants and strings. `Value::Variable`s and
  /// `Value::Label`s must be looked up in the environment. This is function `V` in [Appel].
  pub fn value_to_denotable_value(&self, value: &Value) -> DValue {
    match value{

      | Value::Variable(v)
      | Value::Label(v) => {
        // Todo: What if `v` is not bound?
        self[v].clone()
      },

      Value::Integer(i) => DValue::Integer(*i),

      Value::Real(r) => DValue::Real(*r),

      Value::String(s) => DValue::String(s.clone()),

    }
  }

  pub fn extend<T>(&mut self, iterator: T)
    where T: IntoIterator<Item = (Variable, DValue)>
  {
    let new_bindings = Rc::make_mut(&mut self.bindings);
    new_bindings.extend(iterator);
  }

}

impl Index<&Variable> for Environment {
  type Output = DValue;

  fn index(&self, index: &Variable) -> &Self::Output {
    &self.bindings[index]
  }
}

