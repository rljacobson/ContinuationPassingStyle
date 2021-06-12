/*!

An `Environment` stores the bindings of variables to values.

*/

use std::collections::HashMap;

use super::{
  denotable_value::DValue,
  value::Variable,
  value::{Value, ValueList},
  VariableList
};
use std::ops::Index;


#[derive(Clone, Debug)]
pub struct Environment {
  bindings: HashMap<Variable, DValue>
}

impl Environment {
  pub fn bind(self, variable: Variable, value: DValue) -> Environment{
    // Avoid making a new environment if `variable` is already bound to `value`.
    if let Some(found) = self.get(&variable){
      if found == value {
        return self;
      }
    }
    let mut new_environment = self.clone();
    new_environment.bindings.insert(variable, value);
    new_environment
  }

  pub fn bindn(&self, variables: &VariableList, values: &ValueList) -> Environment {
    let mut new_environment = self.clone();
    new_environment.extend(variables.iter().zip(values));
    new_environment
  }

  pub fn unbind(self, variable: &Variable) -> Environment{
    // Avoid making a new environment if `variable` is already unbound.
    if !self.bindings.contains_key(&variable){
      return self;
    }
    let mut new_environment = self.clone();
    new_environment.bindings.remove(&variable);
    new_environment
  }

  pub fn get(&self, variable: &Variable) -> Option<&DValue> {
    self.bindings.get(variable)
  }

  /// This method is trivial for number variants and strings. `Value::Variable`s and
  /// `Value::Label`s must be looked up in the environment. This is function `V` in [Appel].
  pub fn value_to_denotable_value(&self, value: Value) -> DValue {
    match value{

      | Value::Variable(v)
      | Value::Label(v) => *self[v],

      Value::Integer(i) => DValue::Integer(i),

      Value::Real(r) => DValue::Real(r),

      Value::String(s) => DValue::String(s),

    }
  }
}

impl Index<Variable> for Environment {
  type Output = DValue;

  fn index(&self, index: Variable) -> &Self::Output {
    self.bindings[index]
  }
}



#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
