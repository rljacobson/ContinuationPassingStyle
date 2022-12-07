# Appel/ML vs. Me/Rust

## CPS Representation

| Name         | ML Item                 | ML Type                                                      | Rust Item   | Rust Type                                                    |
| ------------ | ----------------------- | ------------------------------------------------------------ | ----------- | ------------------------------------------------------------ |
| Value        | Value                   | `datatype value = VAR of var | LABEL of var | INT of int | REAL of string | STRING of string` | Value       | `enum Value{ Variable(Variable), Label(Variable), Integer(Integer), Real(Real), String(String) }` |
| Values       | -                       | -                                                            | `ValueList` | `Vec<Value>`                                                 |
| Continuation | Continuation Expression | `CExp`                                                       | `CExp`      |                                                              |
|              |                         |                                                              |             |                                                              |
|              |                         |                                                              |             |                                                              |
|              |                         |                                                              |             |                                                              |
|              |                         |                                                              |             |                                                              |
|              |                         |                                                              |             |                                                              |
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
