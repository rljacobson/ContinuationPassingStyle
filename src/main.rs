#![feature(unboxed_closures)] // To make `ContinuationExpression` a callable struct
#![feature(fn_traits)]        // To make `ContinuationExpression` a callable struct

mod continuation;
mod primitive_op;
mod value;
mod interpreter;

fn main() {
    println!("Hello, world!");
}
