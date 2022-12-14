# Data Flow

@startuml
hide empty description
top to bottom direction

Enum Value {
+Variable
+Label
+Integer
+Real
+String
}

Class Variable {
}


Class Environment {
bindings: Bindings
{method} value_to_denotable_value(Value) -> DValue
}

class ContinuationExpression::PrimitiveOp {
operation  : PrimitiveOp
values     : ValueList
variables  : VariableList
expressions: Vec<CExp>
}
package CPS {

class Store {
next_unused_address: Location
exception_handler: Location
values: Vec<DenotableValue>
integer_values: Vec<Integer>
}

class Continuation {
    f: Rc<RawContinuation>
}

Enum DenotableValue {
+Record
+Integer
+Real
+String
+ByteArray
+Array
+UnboxedArray
+Function
+Exception
}

class Answer{
    f: Continuation
    parameters: Vec<DenotableValue>
}

}

Environment <--> Store
Value <--> DenotableValue
Variable <--> Continuation
ContinuationExpression::PrimitiveOp <--> Answer


@enduml



Enum ContinuationExpression {
+Record
+Select
+Offset
+Apply
+Fix
+Switch
+PrimativeOp
}

ContinuationExpression <--> ContinuationExpression::PrimitiveOp


# Appel/ML vs. Me/Rust

## CPS Representation

| Name         | ML Item                 | ML Type                                                      | Rust Item   | Rust Type                                                    |
| ------------ | ----------------------- | ------------------------------------------------------------ | ----------- | ------------------------------------------------------------ |
| Value        | Value                   | `datatype value = VAR of var | LABEL of var | INT of int | REAL of string | STRING of string` | Value       | `enum Value{ Variable(Variable), Label(Variable), Integer(Integer), Real(Real), String(String) }` |
| Values       | -                       | -                                                            | `ValueList` | `Vec<Value>`                                                 |
| Continuation | Continuation Expression | `CExp`                                                       | `CExp`      |                                                              |
|              |                         |                                                              |             |                                                              |



## Denotation Representation

| Name                      | ML item                                              | ML type                                | Rust item                                                    | Rust type                                                    |
| :------------------------ | :--------------------------------------------------- | :------------------------------------- | :----------------------------------------------------------- | :----------------------------------------------------------- |
| Denotable Values          | `dvalue list`                                        | `dvalue list`                          | `DValueList` / `DenotableValueList`                          | `Vec<DenotableValue>`                                        |
| Parameters                | -                                                    | -                                      | `Parameters`, wrapper for `DValueList`                       | `Rc<DValueList>`                                             |
| Store                     | `store`: (*next unused, value store, integer store*) | `loc * (loc -> dvalue) * (loc -> int)` | `Store`                                                      | `struct Store{ next_unused_address: Location, current_exception_handler: Location, values: DenotableValueList, integer_values: IntegerList }` |
| Continuation              | `dvalue FUNC`                                        | `dvalue list -> store -> answer`       | `RawContinuation`                                            | `dyn Fn(Parameters, Store) -> Answer`                        |
| Continuation              | -                                                    | -                                      | `Continuation` / `DenotableFunction`, wrapper for `RawContinuation` | `pub struct Continuation{pub f: Rc<RawContinuation>}`        |
| Answer                    | Curried continuation: `f [p1 p2 ...]` (no store)     | `store -> answer`                      | `Answer`                                                     | `pub struct Answer {f: Rc<RawContinuation>,   parameters: Parameters}` |
| Current Exception Handler | Global variable                                      | `val handler_ref : loc`                | Member of `Store`                                            | `struct Store{ current_exception_handler: Location, ... }`   |
|                           |                                                      |                                        |                                                              |                                                              |

`ContinuationExpression` Evaluates to `Continuation` (`DenotableFunction`)  Evaluates to  Answer`.
