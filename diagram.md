# Class Diagram

@startuml
hide empty description
allowmixing

Enum Value {
    +Variable
    +Label
    +Integer
    +Real
    +String
}

Class Environment {
    bindings: Bindings
    {method} value_to_denotable_value(Value) -> DValue
}

Class Binding {
    variable: Variable
    value: Value
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

Value ..> DenotableValue : Environment.value_to_denotable_value(value)
Environment::bindings o-- Binding
Binding::value <- Value


Enum ContinuationExpression {
    +Record 
    +Select
    +Offset
    +Apply
    +Fix
    +Switch
    +PrimativeOp
}

struct ContExp::Record {
    values    : Vec<Value>
    variable  : Variable
    expression: CExp
}

struct ContExp::Select {
    location  : Location
    value     : Value
    variable  : Variable
    expression: CExp
}


struct ContExp::Offset {
    location  : Location
    value     : Value
    variable  : Variable
    expression: CExp
}


struct ContExp::Apply {
    function     : Value
    argument_list: Vec<Value>
}

struct ContExp::Fix {
    function_defs: Vec<FunctionDefinition>
    expression   : CExp
}

struct ContExp::Switch {
    value: Value,
    arms : Vec<CExp>
}

struct ContExp::PrimitiveOp {
    operation  : PrimitiveOp
    values     : Vec<Value>
    variables  : Vec<Variable>
    expressions: Vec<CExp>
}

ContinuationExpression::Record o.. ContExp::Record
ContinuationExpression::Select o.. ContExp::Select
ContinuationExpression::Offset o.. ContExp::Offset
ContinuationExpression::Apply o.. ContExp::Apply
ContinuationExpression::Fix o.. ContExp::Fix
ContinuationExpression::Switch o.. ContExp::Switch
ContinuationExpression::PrimitiveOp o.. ContExp::PrimitiveOp

struct Continuation{
    f: Rc<RawContinuation>
}

struct CExp{
    Rc<ContinuationExpression>
}

Class Answer {
  f: Continuation
  parameters: Vec<DenotableValue>
}

ContExp::Apply --> Answer: ContExp::Apply.evaluate(environment)

Answer -> Store: Answer.evaluate(store)

@enduml

