#![feature(unboxed_closures)] // To make `ContinuationExpression` a callable struct
#![feature(fn_traits)]
#![feature(arc_unwrap_or_clone)]        // To make `ContinuationExpression` a callable struct

mod interpreter;

fn main() {
    println!("Hello, world!");
}
