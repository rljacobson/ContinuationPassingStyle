#![allow(dead_code)]

use std::sync::mpsc::TryRecvError::Empty;

use crate::interpreter::{Integer, Location, next_location};

use super::{
  continuation::{Answer, ContinuationList, Parameters},
  denotable_value::{DValue, EMPTY},
  exception::Exception,
};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum PrimitiveOp {
  Multiply,
  Add,
  Subtract,
  Divide,
  Tilde,
  IEqual,
  INEqual,
  Less,
  LessEqual,
  Greater,
  GreaterEqual,

  /// usage: rangechk i j
  /// type : `int -> int -> bool`
  /// When two’s complement is used to represent negative numbers, and `j` is nonnegative,
  /// the test `0 ≤ i < j` can be most efficiently accomplished using an unsigned
  /// comparison operator. The `rangechk` is just “unsigned less than;” the nested
  /// if statements here just express unsigned comparison using signed operators.
  RangeCheck,

  /// Usage: `a!`
  /// type : `['a] -> 'a`
  /// Equivalent to `a[0]`.
  Bang,

  /// Usage: `a[i]`
  /// type : `['a] -> int -> 'a`
  /// Returns the value at index `i` stored in the array `a`.
  Subscript,

  /// Usage: `ordof a, i`
  /// type : `string -> int -> int`
  /// Returns the nth byte as its ASCII code (`DValue::Integer`)
  OrdinalOffset, // todo: What does `ordof` stand for?

  /// Usage: `a := i`
  /// type : array -> int -> unit
  /// Updates the value of `a` at index `0`.
  ColonEqual,

  /// A cheaper version of assignment used when we know the value is not boxed.
  UnboxedAssign,
  Update,
  /// A cheaper version of assignment used when we know the value is not boxed.
  UnboxedUpdate,
  Store,
  MakeRef,
  MakeRefUnboxed,
  ArrayLength,
  SLength,
  GetHandler,
  SetHandler,
  Boxed,
  FAdd,
  FSubtract,
  FMultiply,
  FDivide,
  FEqual,
  FNEqual,
  FGreaterEqual,
  FGreater,
  FLessEqual,
  FLess,
  RShift,
  LShift,
  OrBinary,
  AndBinary,
  XOrBinary,
  NotBinary,
}

impl PrimitiveOp{
  pub fn eval(&self, parameters: Parameters, continuation_list: ContinuationList) -> Answer{

    match (self, *parameters[..], &continuation_list[..]) {
      (
        PrimitiveOp::Multiply,
        [DValue::Integer(i), DValue::Integer(j)],
        [c]
      ) =>  {
              if Some(k) = i.checked_mul(j){
                c(vec![DValue::Integer(k)])
              } else {
                Exception::Overlow.as_answer()
              }
            },

      (PrimitiveOp::Add,
        [DValue::Integer(i), DValue::Integer(j)],
        [c]
      ) =>  {
              if Some(k) = i.checked_add(j){
                c(vec![DValue::Integer(k)])
              } else {
                Exception::Overlow.as_answer()
              }
            },

      (PrimitiveOp::Subtract,
        [DValue::Integer(i), DValue::Integer(j)],
        [c]
      ) =>  {
              if Some(k) = i.checked_sub(j){
                c(vec![DValue::Integer(k)])
              } else {
                Exception::Overlow.as_answer()
              }
            },


      (
        PrimitiveOp::Divide,
        [DValue::Integer(_i), DValue::Integer(0)],
        [c]
      ) =>  {
        Exception::DivideByZero.as_answer()
      },

      (
        PrimitiveOp::Divide,
        [DValue::Integer(i), DValue::Integer(j)],
        [c]
      ) =>  {
        if Some(k) = i.checked_div(j){
          c(vec![DValue::Integer(k)])
        } else {
          Exception::Overflow.as_answer()
        }
      },


      (
        PrimitiveOp::Tilde,
        [DValue::Integer(i)],
        [c]
      ) =>  {
              if Some(k) = 0i32.checked_sub(i){
                c(vec![DValue::Integer(k)])
              } else {
                Exception::Overlow.as_answer()
              }
            },


      (PrimitiveOp::IEqual, [a, b], [t, f]) => {
        if a==b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::INEqual, [a, b], [t, f]) => {
        if a==b {
          f(EMPTY)
        } else {
          t(EMPTY)
        }
      },

      (
        PrimitiveOp::Less,
        [DValue::Integer(i), DValue::Integer(j)],
        [t, f]
      ) => {
        if i<j {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (
        PrimitiveOp::LessEqual,
        [DValue::Integer(i), DValue::Integer(j)],
        [t, f]
      ) => {
        if i<=j {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (
        PrimitiveOp::Greater,
        [DValue::Integer(i), DValue::Integer(j)],
        [t, f]
      ) => {
        if i>j {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (
        PrimitiveOp::GreaterEqual,
        [DValue::Integer(i), DValue::Integer(j)],
        [t, f]
      ) => {
        if i>=j {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (
        PrimitiveOp::RangeCheck,
        [DValue::Integer(i), DValue::Integer(j)],
        [t, f]
      ) => {
        if j<0 {
          if i<0 {
            if i<j {
              t(EMPTY)
            } else {
              f(EMPTY)
            }
          } else {
            t(EMPTY)
          }
        }  else if i < 0 {
          f(EMPTY)
        } else if i<j {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::Bang, [a], [c]) => {
        PrimitiveOp::Subscript.eval(vec![a, DValue::Integer(0)], vec![*c])
      },

      (
        PrimitiveOp::Subscript,
        [DValue::Array(array_range), DValue::Integer(n)],
        [c]
      ) => {
        // The `Subscript` operation requires that we fetch a value from the store. However, we
        // do not have access to a `Store` at this point. The solution is to construct a closure
        // that fetches the right value when given a store, and wrap that closure into an answer.
        Answer{
          // We capture the needed parameters instead of packing and unpacking the `Answer`'s
          // parameters member
          f: move | _, store | {
            let i = store.fetch(array_range.start+n);
            c.f(vec![i], store)
          },
          parameters: EMPTY
        }
      }

      (
        PrimitiveOp::Subscript,
        [DValue::UnboxedArray(array_range), DValue::Integer(n)],
        [c]
      ) => {
        // The `Subscript` operation requires that we fetch a value from the store. However, we
        // do not have access to a `Store` at this point. The solution is to construct a closure
        // that fetches the right value when given a store, and wrap that closure into an answer.
        Answer{
          // We capture the needed parameters instead of packing and unpacking the `Answer`'s
          // parameters member.
          f: move | _, store | {
            let i = store.fetch_integer(array_range.start + n);
            c.f(vec![i], store)
          },
          parameters: EMPTY
        }
      }

      (
        PrimitiveOp::Subscript,
        [DValue::Record { values, idx: i }, DValue::Integer(n)],
        [c]
      ) => {
        c(vec![values[i+j]])
      },

      (
        PrimitiveOp::OrdinalOffset,
        [DValue::String(a), DValue::Integer(i)],
        [c]
      ) => {
        c( DValue::Integer(a.as_bytes()[i as usize] as i32) )
      },

      (
        PrimitiveOp::ColonEqual,
        [array @ DValue::Array(_), value],
        c
      ) => {
        PrimitiveOp::Update.eval(vec![array, ZERO, value], c.into())
      }

      (PrimitiveOp::UnboxedAssign, [a, v], c) => {
        PrimitiveOp::UnboxedUpdate.eval(vec![a, ZERO, v], c.into())
      },

      (
        PrimitiveOp::Update,
        [DValue::Array(array_range), DValue::Integer(n), value],
        [c]
      ) => {
        // The `Update` operation requires that we update a value in the store. However, we
        // do not have access to a `Store` at this point. The solution is to construct a closure
        // that updates the right value when given a store, and wrap that closure into an answer.

        Answer{
          // We capture the needed parameters instead of packing and unpacking.
          f: move | _, store | {

            let new_store = store.update(array_range.start as Location + n, value);
            c.f(EMPTY, new_store)
          },
          parameters: EMPTY
        }
      },


      (
        PrimitiveOp::Update,
        [DValue::UnboxedArray(array_range), DValue::Integer(n), value @ DValue::Integer(_)],
        [c]
      ) => {
        // The `Update` operation requires that we update a value in the store. However, we
        // do not have access to a `Store` at this point. The solution is to construct a closure
        // that updates the right value when given a store, and wrap that closure into an answer.

        Answer{
          // We capture the needed parameters instead of packing and unpacking.
          f: move | _, store | {

            let new_store =
                store.update_integer(array_range.start as Location + n, value);
            c.f(EMPTY, new_store)
          },
          parameters: EMPTY
        }
      },

      (
        PrimitiveOp::UnboxedUpdate,
        [
          DValue::Array(array_range),
          DValue::Integer(n),
          value @ DValue::Integer(_)
        ],
        [c]
      ) => {
        Answer{
          f: move | _, store | {
            let new_store = store.update(array_range.start + n, value);
            c.f(EMPTY, new_store)
          },
          parameters: EMPTY
        }
      },

      (
        PrimitiveOp::UnboxedUpdate,
        [
          DValue::UnboxedArray(array_range),
          DValue::Integer(n),
          value @ DValue::Integer(_)
        ],
        [c]
      ) => {
        Answer{
          f: move | _, store | {
            let new_store =
                store.update_integer(array_range.start + n as Location, value);
            c.f(EMPTY, new_store)
          },
          parameters: EMPTY
        }
      },

      (
        PrimitiveOp::Store,
        [DValue::ByteArray(array_range), DValue::Integer(i), DValue::Integer(v)],
        [c]
      ) => {
        if v < 0 || v >= 256 {
          Exception::Undefined.as_answer()
        } else {
          Answer{
            f: move | _, store | {
              let new_store
                  = store.update_integer(array_range.start + i, v);
              c.f(EMPTY, new_store)
            },
            parameters: EMPTY
          }
        }
      },

      (PrimitiveOp::MakeRef, [value], [c]) => {
        Answer{
          f: move | _, store | {
            let last_address = store.next_unused_address;
            let mut new_store =
                store.update(*last_address, value);
            new_store.next_unused_address = next_location(last_address);
            c.f([DValue::Array(l..l+1)], new_store)
          },
          parameters: EMPTY
        }
      }

      (PrimitiveOp::MakeRefUnboxed, [DValue::Integer(value)], [c]) => {
        Answer{
          f: move | _, store | {
            let last_address = store.next_unused_address;
            let mut new_store =
                store.update_integer(*last_address, value);
            new_store.next_unused_address = next_location(last_address);
            c.f([DValue::Array(l..l+1)], new_store)
          },
          parameters: EMPTY
        }
      },

      (PrimitiveOp::ArrayLength, [DValue::Array(array_range)], [c]) => {
        c([DValue::Integer(array_range.len() as i32)])
      },

      (PrimitiveOp::ArrayLength, [DValue::UnboxedArray(array_range)], [c]) => {
        c([DValue::Integer(array_range.len() as i32)])
      },

      (PrimitiveOp::SLength, [DValue::ByteArray(array_range)], [c]) => {
        c([DValue::Integer(array_range.len() as i32)])
      },

      (PrimitiveOp::SLength, [DValue::String(String)], [c]) => {
        c([DValue::Integer(String.len() as i32)])
      },

      (PrimitiveOp::GetHandler, [], [c]) => {
        Answer{
          f: move | _, store | {
            c.f(vec![store.fetch(store.exception_handler)], store)
          },
          parameters: EMPTY
        }
      },

      (PrimitiveOp::SetHandler, [new_handler], [c]) => {
        Answer{
          f: move | _, store | {
            let new_store = store.update(store.exception_handler, new_handler);
            c.f(EMPTY, new_store)
          },
          parameters: EMPTY
        }
      },

      (PrimitiveOp::Boxed, [DValue::Integer(_)], [t, f]) => {
        f(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::Real(_)], [t, f]) => {
        f(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::Record { .. }], [t, f]) => {
        t(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::String(_)], [t, f]) => {
        t(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::Array(_)], [t, f]) => {
        t(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::UnboxedArray(_)], [t, f]) => {
        t(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::ByteArray(_)], [t, f]) => {
        t(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::Function(_)], [t, f]) => {
        t(EMPTY)
      },

      (PrimitiveOp::FAdd, [DValue::Real(a), DValue::Real(b)], [c]) => {
        if Some(k) = a.checked_add(b){
          c(vec![DValue::Real(k)])
        } else {
          Exception::Overlow.as_answer()
        }
      },

      (PrimitiveOp::FSubtract, [DValue::Real(a), DValue::Real(b)], [c]) => {
        if Some(k) = a.checked_sub(b){
          c(vec![DValue::Real(k)])
        } else {
          Exception::Overlow.as_answer()
        }
      },

      (PrimitiveOp::FMultiply, [DValue::Real(a), DValue::Real(b)], [c]) => {
        if Some(k) = a.checked_mul(b){
          c(vec![DValue::Real(k)])
        } else {
          Exception::Overlow.as_answer()
        }
      },

      (PrimitiveOp::FDivide, [DValue::Real(a), DValue::Real(b)], [c]) => {

      },

      (
        PrimitiveOp::FDivide,
        [DValue::Real(_a), DValue::Real(0.0)],
        [c]
      ) =>  {
        Exception::DivideByZero.as_answer()
      },

      (
        PrimitiveOp::FDivide,
        [DValue::Real(a), DValue::Real(b)],
        [c]
      ) =>  {
        if Some(k) = a.checked_div(b){
          c(vec![DValue::Real(k)])
        } else {
          Exception::Overflow.as_answer()
        }
      },


      (PrimitiveOp::FEqual, [DValue::Real(a), DValue::Real(b)], [t, f]) => {
        if a == b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::FNEqual, [DValue::Real(a), DValue::Real(b)], [t, f]) => {
        if a != b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::FGreaterEqual, [DValue::Real(a), DValue::Real(b)], [t, f]) => {
        if a >= b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::FGreater, [DValue::Real(a), DValue::Real(b)], [t, f]) => {
        if a > b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::FLessEqual, [DValue::Real(a), DValue::Real(b)], [t, f]) => {
        if a <= b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::FLess, [DValue::Real(a), DValue::Real(b)], [t, f]) => {
        if a < b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      /*
      (PrimitiveOp::RShift, [DValue::Real(a), DValue::Real(b)], [c]) => {

      },

      (PrimitiveOp::LShift, [DValue::Real(a), DValue::Real(b)], [c]) => {

      },

      (PrimitiveOp::OrBinary, [DValue::Real(a), DValue::Real(b)], [c]) => {

      },

      (PrimitiveOp::AndBinary, [DValue::Real(a), DValue::Real(b)], [c]) => {

      },

      (PrimitiveOp::XOrBinary, [DValue::Real(a), DValue::Real(b)], [c]) => {

      },

      (PrimitiveOp::NotBinary, [DValue::Real(a), DValue::Real(b)], [c]) => {

      },
      */

    }

  }

}
