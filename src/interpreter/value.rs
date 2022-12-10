#![allow(dead_code)]
/*!

  A `Value` is a primitive non-composite value, which may or may not be denotable. A [`DenotableValue`], or `DValue` for
  short, is a (possibly composite) value that can be denoted.

*/

use std::hash::{Hash, Hasher};

use super::{
  Integer,
  Real,
  Variable
};

pub type ValueList = Vec<Value>;

#[derive(Clone, Debug)]
pub enum Value{
  Variable(Variable),    // Can be bound to denotable value with `Environment`
  Label(Variable),       // Can be bound to denotable value with `Environment`
  Integer(Integer),      // Denotable. See [`DenotableValue`].
  Real(Real),            // Denotable.
  String(String)         // Denotable.
}

impl PartialEq for Value {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {

      (Value::Variable(a), Value::Variable(b)) => {
        a==b
      },

      (Value::Label(a), Value::Label(b)) => {
        a==b
      },

      (Value::Integer(a), Value::Integer(b)) => {
        a==b
      },

      (Value::Real(a), Value::Real(b)) => {
        (a == b) || (a.is_nan() && b.is_nan())
      },

      (Value::String(a), Value::String(b)) => {
        a==b
      },

      _ => {
        false
      }
    }
  }
}

impl Eq for Value {}


impl Hash for Value{
      fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
      match self{

        Value::Real(r) => {
          r.to_bits().hash(state)
        },

        Value::Variable(val) => val.hash(state),

        Value::Label(val) => val.hash(state),

        Value::Integer(val) => val.hash(state),

        Value::String(val) => val.hash(state)

      }
    }
}
