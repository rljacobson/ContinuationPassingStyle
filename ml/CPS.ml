signature CPS = sig

    eqtype var

    datatype value =
        VAR of var
        | LABEL of var
        | INT of int
        | REAL of string
        | STRING of string

    datatype accesspath =
        OFFp of int
        | SELp of int * accesspath

    datatype primop =
        * | + | - | div | ~
        | ieql | ineq | < | <= | > | >= | rangechk
        | ! | subscript | ordof
        | := | unboxedassign | update | unboxedupdate | store
        | makeref | makerefunboxed | alength | slength
        | gethdlr | sethdlr
        | boxed
        | fadd | fsub | fdiv | fmul
        | feql | fneq | fge | fgt | fle | flt
        | rshift | lshift | orb | andb | xorb | notb

    datatype cexp =
        RECORD of (value * accesspath) list * var * cexp
        | SELECT of int * value * var * cexp
        | OFFSET of int * value * var * cexp
        | APP of value * value list
        | FIX of (var * var list * cexp) list * cexp
        | SWITCH of value * cexp list
        | PRIMOP of primop * value list * var list * cexp list
end
