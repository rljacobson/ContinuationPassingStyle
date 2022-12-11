#![allow(dead_code)]


use std::rc::Rc;

use ordered_float::OrderedFloat;

use crate::{
  interpreter::{
    cps::{
      denotable_value::{DValue, EMPTY, ZERO},
      continuation::{Answer, ContinuationList, Parameters}
    },
    exception::{Exception},
    Integer,
    Location
  }
};
use crate::interpreter::cps::continuation::Continuation;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum PrimitiveOp {
  Multiply,     // *
  Add,          // +
  Subtract,     // -
  Divide,       // div
  Tilde,        // ~
  IEqual,       // ieql
  INEqual,      // ineq
  Less,         // <
  LessEqual,    // <=
  Greater,      // >
  GreaterEqual, // >=

  /// usage: rangechk i j
  ///
  /// type : `int -> int -> bool`
  ///
  /// When two’s complement is used to represent negative numbers, and `j` is nonnegative,
  /// the test `0 ≤ i < j` can be most efficiently accomplished using an unsigned
  /// comparison operator. The `rangechk` is just “unsigned less than;” the nested
  /// if statements here just express unsigned comparison using signed operators.
  RangeCheck, // rangechk

  /// Usage: `a!`
  ///
  /// type : `['a] -> 'a`
  ///
  /// Equivalent to `a[0]`.
  Bang, // !

  /// Usage: `a[i]`
  /// type : `['a] -> int -> 'a`
  /// Returns the value at index `i` stored in the array `a`.
  Subscript, // subscript

  /// Usage: `ordof a, i`
  ///
  /// type : `string -> int -> int`
  ///
  /// Returns the nth byte as its ASCII code (`DValue::Integer`)
  OrdinalOf,

  /// Usage: `a := i`
  ///
  /// type : array -> int -> unit
  ///
  /// Updates the value of `a` at index `0`.
  ColonEqual, // :=

  /// A cheaper version of assignment used when we know the value is not boxed.
  UnboxedAssign,  // unboxedassign
  Update,         // update
  /// A cheaper version of assignment used when we know the value is not boxed.
  UnboxedUpdate,  // unboxedupdate
  Store,          // store
  MakeRef,        // makeref
  MakeRefUnboxed, // makerefunboxed
  ArrayLength,    // alength
  StringLength,   // slength
  GetHandler,     // gethdlr
  SetHandler,     // sethdlr
  Boxed,          // boxed
  FAdd,           // fadd
  FSubtract,      // fsub
  FMultiply,      // fmul
  FDivide,        // fdiv
  FEqual,         // feql
  FNEqual,        // fneq
  FGreaterEqual,  // fge
  FGreater,       // fgt
  FLessEqual,     // fle
  FLess,          // flt
  // RShift,         // rshift
  // LShift,         // lshift
  // OrBinary,       // orb
  // AndBinary,      // andb
  // XOrBinary,      // xorb
  // NotBinary,      // notb
}

impl PrimitiveOp{
  pub fn evaluate(self, mut parameters: Parameters, mut continuation_list: ContinuationList) -> Answer{
    let continuation_count = continuation_list.len();

    // Have to work around inability to destructure ContinuationList.
    match (self, parameters.as_mut_slice(), continuation_count) {
      (
        PrimitiveOp::Multiply,
        [DValue::Integer(i), DValue::Integer(j)],
        1
      ) =>  {
              let c = continuation_list.pop().unwrap();
              if let Some(k) = i.checked_mul(*j){
                c(vec![DValue::Integer(k)])
              } else {
                Exception::Overflow.as_answer()
              }
            },

      (
        PrimitiveOp::Add,
        [DValue::Integer(i), DValue::Integer(j)],
        1
      ) =>  {
        let c = continuation_list.pop().unwrap();
              if let Some(k) = i.checked_add(*j){
                c(vec![DValue::Integer(k)])
              } else {
                Exception::Overflow.as_answer()
              }
            },

      (
        PrimitiveOp::Subtract,
        [DValue::Integer(i), DValue::Integer(j)],
        1
      ) =>  {
              let c = continuation_list.pop().unwrap();
              if let Some(k) = i.checked_sub(*j){
                c(vec![DValue::Integer(k)])
              } else {
                Exception::Overflow.as_answer()
              }
            },


      (
        PrimitiveOp::Divide,
        [DValue::Integer(_i), DValue::Integer(0)],
        _
      ) =>  {
        Exception::DivideByZero.as_answer()
      },

      (
        PrimitiveOp::Divide,
        [DValue::Integer(i), DValue::Integer(j)],
        1
      ) =>  {
        let c = continuation_list.pop().unwrap();
        if let Some(k) = i.checked_div(*j){
          c(vec![DValue::Integer(k)])
        } else {
          Exception::Overflow.as_answer()
        }
      },


      (
        PrimitiveOp::Tilde,
        [DValue::Integer(i)],
        1
      ) =>  {
              let c = continuation_list.pop().unwrap();
              if let Some(k) = 0i64.checked_sub(*i){
                c(vec![DValue::Integer(k)])
              } else {
                Exception::Overflow.as_answer()
              }
            },


      (PrimitiveOp::IEqual, [a, b], 2) => {
        let f = continuation_list.pop().unwrap();
        let t = continuation_list.pop().unwrap();
        if a==b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::INEqual, [a, b], 2) => {
        let f = continuation_list.pop().unwrap();
        let t = continuation_list.pop().unwrap();
        if a==b {
          f(EMPTY)
        } else {
          t(EMPTY)
        }
      },

      (
        PrimitiveOp::Less,
        [DValue::Integer(i), DValue::Integer(j)],
        2
      ) => {
        let f = continuation_list.pop().unwrap();
        let t = continuation_list.pop().unwrap();
        if i<j {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (
        PrimitiveOp::LessEqual,
        [DValue::Integer(i), DValue::Integer(j)],
        2
      ) => {
        let f = continuation_list.pop().unwrap();
        let t = continuation_list.pop().unwrap();
        if i<=j {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (
        PrimitiveOp::Greater,
        [DValue::Integer(i), DValue::Integer(j)],
        2
      ) => {
        let f = continuation_list.pop().unwrap();
        let t = continuation_list.pop().unwrap();
        if i>j {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (
        PrimitiveOp::GreaterEqual,
        [DValue::Integer(i), DValue::Integer(j)],
        2
      ) => {
        let f = continuation_list.pop().unwrap();
        let t = continuation_list.pop().unwrap();
        if i>=j {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (
        PrimitiveOp::RangeCheck,
        [DValue::Integer(i), DValue::Integer(j)],
        2
      ) => {
        let f = continuation_list.pop().unwrap();
        let t = continuation_list.pop().unwrap();
        if *j<0 {
          if *i<0 {
            if i<j {
              t(EMPTY)
            } else {
              f(EMPTY)
            }
          } else {
            t(EMPTY)
          }
        }  else if *i < 0 {
          f(EMPTY)
        } else if i<j {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::Bang, [a], 1) => {
        PrimitiveOp::Subscript.evaluate(vec![a.clone(), DValue::Integer(0)], continuation_list)
      },

      (
        PrimitiveOp::Subscript,
        [DValue::Array(array_range), DValue::Integer(n)],
        1
      ) => {
        let continuation: Continuation = continuation_list.pop().unwrap();
        let range = array_range.clone();
        let m  = *n;
        // The `Subscript` operation requires that we fetch a value from the store. However, we
        // do not have access to a `Store` at this point. The solution is to construct a closure
        // that fetches the right value when given a store, and wrap that closure into an answer.
        Answer{
          // We capture the needed parameters instead of packing and unpacking the `Answer`'s
          // parameters member.
          f: Rc::new(move | _, store | {
            let i = store.fetch(range.start + m as usize);
            (continuation.f)(&vec![i.clone()], store)
          }),
          parameters: EMPTY
        }
      }

      (
        PrimitiveOp::Subscript,
        [DValue::UnboxedArray(array_range), DValue::Integer(n)],
        1
      ) => {
        let continuation: Continuation = continuation_list.pop().unwrap();
        let range = array_range.clone();
        let m  = *n;
        // The `Subscript` operation requires that we fetch a value from the store. However, we
        // do not have access to a `Store` at this point. The solution is to construct a closure
        // that fetches the right value when given a store, and wrap that closure into an answer.
        Answer{
          // We capture the needed parameters instead of packing and unpacking the `Answer`'s
          // parameters member.
          f: Rc::new(move | _, store | {
            let i = store.fetch_integer(range.start + m as usize);
            (continuation.f)(&vec![i], store)
          }),
          parameters: EMPTY
        }
      }

      (
        PrimitiveOp::Subscript,
        [DValue::Record { values, idx: i }, DValue::Integer(j)],
        1
      ) => {
        let c: Continuation = continuation_list.pop().unwrap();
        c(vec![values[*i + *j as usize].clone()])
      },

      (
        PrimitiveOp::OrdinalOf,
        [DValue::String(a), DValue::Integer(i)],
        1
      ) => {
        let c: Continuation = continuation_list.pop().unwrap();
        c( vec![DValue::Integer(a.as_bytes()[*i as usize] as Integer)] )
      },

      (
        PrimitiveOp::ColonEqual,
        [array @ DValue::Array(_), value],
        1
      ) => {
        PrimitiveOp::Update.evaluate(vec![array.clone(), ZERO.clone(), value.clone()], continuation_list)
      }

      (PrimitiveOp::UnboxedAssign, [a, v], 1) => {
        PrimitiveOp::UnboxedUpdate.evaluate(vec![a.clone(), ZERO.clone(), v.clone()], continuation_list)
      },

      (
        PrimitiveOp::Update,
        [DValue::Array(array_range), DValue::Integer(n), value],
        1
      ) => {
        let continuation: Continuation = continuation_list.pop().unwrap();
        let range = array_range.clone();
        let m  = *n;
        let v = value.clone();
        // The `Update` operation requires that we update a value in the store. However, we
        // do not have access to a `Store` at this point. The solution is to construct a closure
        // that updates the right value when given a store, and wrap that closure into an answer.
        Answer{
          // We capture the needed parameters instead of packing and unpacking.
          f: Rc::new(move | _, store | {
            let new_store = store.update(range.start as Location + m as usize, v.clone());
            (continuation.f)(&EMPTY, &new_store)
          }),
          parameters: EMPTY
        }
      },


      (
        PrimitiveOp::Update,
        [DValue::UnboxedArray(array_range), DValue::Integer(n), DValue::Integer(value)],
        1
      ) => {
        let continuation = continuation_list.pop().unwrap();
        let range = array_range.clone();
        let m  = *n;
        let v = value.clone();
        // The `Update` operation requires that we update a value in the store. However, we
        // do not have access to a `Store` at this point. The solution is to construct a closure
        // that updates the right value when given a store, and wrap that closure into an answer.
        Answer{
          // We capture the needed parameters instead of packing and unpacking.
          f: Rc::new(move | _, store | {

            let new_store =
                store.update_integer(range.start as Location + m as usize, v.clone());
            (continuation.f)(&EMPTY, &new_store)
          }),
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
        1
      ) => {
        let continuation = continuation_list.pop().unwrap();
        let range = array_range.clone();
        let m  = *n;
        let v = value.clone();
        Answer{
          f: Rc::new(move | _, store | {
            let new_store = store.update(range.start + m as Location, v.clone());
            (continuation.f)(&EMPTY, &new_store)
          }),
          parameters: EMPTY
        }
      },

      (
        PrimitiveOp::UnboxedUpdate,
        [
          DValue::UnboxedArray(array_range),
          DValue::Integer(n),
          DValue::Integer(value)
        ],
        1
      ) => {
        let continuation = continuation_list.pop().unwrap();
        let range = array_range.clone();
        let m  = *n;
        let v = value.clone();
        Answer{
          f: Rc::new(move | _, store | {
            let new_store =
                store.update_integer(range.start + m as Location, v.clone());
            (continuation.f)(&EMPTY, &new_store)
          }),
          parameters: EMPTY
        }
      },

      (
        PrimitiveOp::Store,
        [DValue::ByteArray(array_range), DValue::Integer(i), DValue::Integer(v)],
        1
      ) => {
        if *v < 0 || *v >= 256 {
          // The value of `v` must fit into a byte.
          Exception::Overflow.as_answer()
        } else {
          let continuation = continuation_list.pop().unwrap();
          let range = array_range.clone();
          let j = *i;
          let u = *v;
          Answer{
            f: Rc::new(move | _, store | {
              let new_store
                  = store.update_integer(range.start + j as Location, u);
              (continuation.f)(&EMPTY, &new_store)
            }),
            parameters: EMPTY
          }
        }
      },

      (PrimitiveOp::MakeRef, [value], 1) => {
        let v = value.clone();
        let continuation = continuation_list.pop().unwrap();
        Answer{
          f: Rc::new(move | _, store | {
            let last_address = store.next_unused_address;
            let mut new_store =
                store.update(last_address, v.clone());
            new_store.next_unused_address = next_location(last_address);
            // Todo: Should this be `DValue::Array(last_address..last_address+1)]` or
            //       `DValue::Array(new_store.next_unused_address..new_store.next_unused_address+1)]`?
            (continuation.f)(&vec![DValue::Array(last_address..last_address+1)], &new_store)
          }),
          parameters: EMPTY
        }
      }

      (PrimitiveOp::MakeRefUnboxed, [DValue::Integer(value)], 1) => {
        let v = *value;
        let continuation = continuation_list.pop().unwrap();
        Answer{
          f: Rc::new(move | _, store | {
            let last_address = store.next_unused_address;
            let mut new_store =
                store.update_integer(last_address, v);
            new_store.next_unused_address = next_location(last_address);
            // Todo: Should this be `DValue::Array(last_address..last_address+1)]` or
            //       `DValue::Array(new_store.next_unused_address..new_store.next_unused_address+1)]`?
            (continuation.f)(&vec![DValue::Array(last_address..last_address+1)], &new_store)
          }),
          parameters: EMPTY
        }
      },

      (PrimitiveOp::ArrayLength, [DValue::Array(array_range)], 1) => {
        let c = continuation_list.pop().unwrap();
        c(vec![DValue::Integer(array_range.len() as Integer)])
      },

      (PrimitiveOp::ArrayLength, [DValue::UnboxedArray(array_range)], 1) => {
        let c = continuation_list.pop().unwrap();
        c(vec![DValue::Integer(array_range.len() as Integer)])
      },

      // The StringLength operator is used for `ByteArray`s, as they are considered mutable strings.
      (PrimitiveOp::StringLength, [DValue::ByteArray(array_range)], 1) => {
        let c = continuation_list.pop().unwrap();
        c(vec![DValue::Integer(array_range.len() as Integer)])
      },

      (PrimitiveOp::StringLength, [DValue::String(s)], 1) => {
        let c = continuation_list.pop().unwrap();
        c(vec![DValue::Integer(s.len() as Integer)])
      },

      (PrimitiveOp::GetHandler, [], 1) => {
        let continuation = continuation_list.pop().unwrap();
        Answer{
          f: Rc::new(move | _, store | {
            (continuation.f)(&vec![store.fetch(store.exception_handler).clone()], store)
          }),
          parameters: EMPTY
        }
      },

      (PrimitiveOp::SetHandler, [new_handler], 1) => {
        let continuation = continuation_list.pop().unwrap();
        let handler = new_handler.clone();
        Answer{
          f: Rc::new(move | _, store | {
            let new_store = store.update(store.exception_handler, handler.clone());
            (continuation.f)(&EMPTY, &new_store)
          }),
          parameters: EMPTY
        }
      },

      (PrimitiveOp::Boxed, [DValue::Integer(_)], 2) => {
        let f = continuation_list.pop().unwrap();
        f(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::Real(_)], 2) => {
        let f = continuation_list.pop().unwrap();
        f(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::Record { .. }], 2) => {
        let t = continuation_list.remove(0);
        t(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::String(_)], 2) => {
        let t = continuation_list.remove(0);
        t(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::Array(_)], 2) => {
        let t = continuation_list.remove(0);
        t(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::UnboxedArray(_)], 2) => {
        let t = continuation_list.remove(0);
        t(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::ByteArray(_)], 2) => {
        let t = continuation_list.remove(0);
        t(EMPTY)
      },

      (PrimitiveOp::Boxed, [DValue::Function(_)], 2) => {
        let t = continuation_list.remove(0);
        t(EMPTY)
      },

      (PrimitiveOp::FAdd, [DValue::Real(a), DValue::Real(b)], 1) => {
        let c = continuation_list.pop().unwrap();
        c(vec![DValue::Real(OrderedFloat(a.0 + b.0))])

        // No overflow detection for reals.
        // if let Some(k) = a.0.checked_add(b){
        //   c(vec![DValue::Real(k)])
        // } else {
        //   Exception::Overflow.as_answer()
        // }
      },

      (PrimitiveOp::FSubtract, [DValue::Real(a), DValue::Real(b)], 1) => {
        let c = continuation_list.pop().unwrap();
        c(vec![DValue::Real(OrderedFloat(a.0 - b.0))])

        // No overflow detection for reals.
        // if let Some(k) = a.0.checked_sub(b){
        //   c(vec![DValue::Real(k)])
        // } else {
        //   Exception::Overflow.as_answer()
        // }
      },

      (PrimitiveOp::FMultiply, [DValue::Real(a), DValue::Real(b)], 1) => {
        let c = continuation_list.pop().unwrap();
        c(vec![DValue::Real(OrderedFloat(a.0 * b.0))])

        // No overflow detection for reals.
        // if let Some(k) = a.0.checked_mul(b){
        //   c(vec![DValue::Real(k)])
        // } else {
        //   Exception::Overflow.as_answer()
        // }
      },

      #[allow(illegal_floating_point_literal_pattern)]
      (
        PrimitiveOp::FDivide,
        [DValue::Real(_a), DValue::Real(OrderedFloat(0.0))],
        _
      ) =>  {
        Exception::DivideByZero.as_answer()
      },

      (
        PrimitiveOp::FDivide,
        [DValue::Real(a), DValue::Real(b)],
        1
      ) => {
        let c = continuation_list.pop().unwrap();
        c(vec![DValue::Real(OrderedFloat(a.0 / b.0))])

        // No overflow detection for reals.
        // if let Some(k) = a.0.checked_div(b){
        //   c(vec![DValue::Real(k)])
        // } else {
        //   Exception::Overflow.as_answer()
        // }
      },


      (PrimitiveOp::FEqual, [DValue::Real(a), DValue::Real(b)], 2) => {
        let f = continuation_list.pop().unwrap();
        let t = continuation_list.pop().unwrap();
        if a == b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::FNEqual, [DValue::Real(a), DValue::Real(b)], 2) => {
        let f = continuation_list.pop().unwrap();
        let t = continuation_list.pop().unwrap();
        if a != b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::FGreaterEqual, [DValue::Real(a), DValue::Real(b)], 2) => {
        let f = continuation_list.pop().unwrap();
        let t = continuation_list.pop().unwrap();
        if a >= b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::FGreater, [DValue::Real(a), DValue::Real(b)], 2) => {
        let f = continuation_list.pop().unwrap();
        let t = continuation_list.pop().unwrap();
        if a > b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::FLessEqual, [DValue::Real(a), DValue::Real(b)], 2) => {
        let f = continuation_list.pop().unwrap();
        let t = continuation_list.pop().unwrap();
        if a <= b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      (PrimitiveOp::FLess, [DValue::Real(a), DValue::Real(b)], 2) => {
        let f = continuation_list.pop().unwrap();
        let t = continuation_list.pop().unwrap();
        if a < b {
          t(EMPTY)
        } else {
          f(EMPTY)
        }
      },

      _ => {
        unreachable!()
      }

      // Todo: \[Appel] doesn't implement these operations?
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



fn next_location(last_address: Location) -> Location {//-> Span<'static> {
  last_address + std::mem::size_of::<DValue>()
}

