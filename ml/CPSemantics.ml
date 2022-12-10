functor CPSsemantics(

    structure CPS  : CPS
    val minint     : int  val maxint  : int
    val minreal    : real val maxreal : real
    val string2real: string -> real

    eqtype loc

    val nextloc    : loc -> loc
    val arbitrarily: 'a * 'a -> 'a

    type answer

    datatype dvalue = RECORD of dvalue list * int
        | INT of int
        | REAL of real
        | FUNC of dvalue list ->
            (loc*(loc->dvalue)*(loc->int)) -> answer
        | STRING of string
        | BYTEARRAY of loc list
        | ARRAY of loc list
        | UARRAY of loc list

    val handler_ref : loc
    val overflow_exn: dvalue
    val div_exn     : dvalue
        ) :
    sig val eval: CPS.var list * CPS.cexp ->
                  dvalue list ->
                  (loc*(loc->dvalue)*(loc->int)) ->
                  answer
    end =
struct

type store = loc * (loc -> dvalue) * (loc -> int)

fun fetch ((_, f, _): store) (l: loc) = f l
fun upd ((n, f, g):store, l: loc, v: dvalue) =
    (n, fn i => if i=l then v else f i, g)
fun fetchi ((_, _, g): store) (l: loc) = g l
fun updi ((n, f, g):store, l: loc, v: int) =
    (n, f, fn i => if i=l then v else g i)

exception Undefined

fun eq(RECORD(a, i), RECORD(b, j)) =
        arbitrarily(i=j andalso eqlist(a, b), false)
        | eq(INT i, INT j) = i=j
        | eq(REAL a, REAL b) = arbitrarily(a=b, false)
        | eq(STRING a, STRING b) = arbitrarily(a=b, false)
        | eq(BYTEARRAY nil, BYTEARRAY nil) = true
        | eq(BYTEARRAY(a::_), BYTEARRAY(b::_)) = a=b
        | eq(ARRAY nil, ARRAY nil) = true
        | eq(ARRAY(a::_), ARRAY(b::_)) = a=b
        | eq(UARRAY nil, UARRAY nil) = true
        | eq(UARRAY(a::_), UARRAY(b::_)) = a=b
        | eq(FUNC a, FUNC b) = raise Undefined
        | eq(_, _) = false
    and eqlist(a::al, b::bl) = eq(a, b) andalso eqlist(al, bl)
        | eqlist(nil, nil) = true

fun do_raise exn s = let val FUNC f = fetch s handler_ref
        in f [exn] s
    end

fun overflow(n: unit->int,
             c: dvalue list -> store -> answer) =
    if (n() >= minint andalso n() <= maxint)
        handle Overflow=> false
    then c [INT(n())]
    else do_raise overflow_exn

fun overflowr(n, c) =
    if (n() >= minreal andalso n() <= maxreal)
        handle Overflow => false
    then c [REAL(n())]
    else do_raise overflow_exn

fun evalprim (CPS.+ : CPS.primop,
              [INT i, INT j]: dvalue list,
              [c]: (dvalue list -> store -> answer) list) =
                overflow(fn()=>i+j, c)
    | evalprim (CPS.-, [INT i, INT j], [c]) =
        overflow(fn()=>i-j, c)
    | evalprim (CPS.*, [INT i, INT j], [c]) =
        overflow(fn()=>i*j, c)
    | evalprim (CPS.div, [INT i, INT 0], [c]) = do_raise div_exn
    | evalprim (CPS.div, [INT i, INT j], [c]) =
        overflow(fn()=>i div j, c)
    | evalprim (CPS.~, [INT i], [c]) = overflow(fn()=>0-i, c)
    | evalprim (CPS.<, [INT i, INT j], [t, f]) =
        if i<j then t[] else f[]
    | evalprim (CPS.<=, [INT i, INT j], [t, f]) =
        if j<i then f[] else t[]
    | evalprim (CPS.>, [INT i, INT j], [t, f]) =
        if j<i then t[] else f[]
    | evalprim (CPS.>=, [INT i, INT j], [t, f]) =
        if i<j then f[] else t[]
    | evalprim (CPS.ieql, [a, b], [t, f]) =
        if eq(a, b) then t[] else f[]
    | evalprim (CPS.ineq, [a, b], [t, f]) =
        if eq(a, b) then f[] else t[]
    | evalprim (CPS.rangechk, [INT i, INT j], [t, f]) =
        if j<0
        then if i<0
            then if i<j then t[] else f[]
            else t[]
        else if i<0
            then f[]
            else if i<j then t[]
            else f[]
    | evalprim (CPS.boxed, [INT _], [t, f]) = f[]
    | evalprim (CPS.boxed, [RECORD _], [t, f]) = t[]
    | evalprim (CPS.boxed, [STRING _], [t, f]) = t[]
    | evalprim (CPS.boxed, [ARRAY _], [t, f]) = t[]
    | evalprim (CPS.boxed, [UARRAY _], [t, f]) = t[]
    | evalprim (CPS.boxed, [BYTEARRAY _], [t, f]) = t[]
    | evalprim (CPS.boxed, [FUNC _], [t, f]) = t[]
    | evalprim (CPS.!, [a], [c]) =
        evalprim(CPS.subscript, [a, INT 0], [c])
    | evalprim (CPS.subscript, [ARRAY a, INT n], [c]) =
        (fn s => c [fetch s (nth(a, n))] s)
    | evalprim (CPS.subscript, [UARRAY a, INT n], [c]) =
        (fn s => c [INT(fetchi s (nth(a, n)))] s)
    | evalprim (CPS.subscript, [RECORD(a, i), INT j], [c]) =
        c [nth(a, i+j)]
    | evalprim (CPS.ordof, [STRING a, INT i], [c]) =
        c [INT(String.ordof(a, i))]
    | evalprim (CPS.ordof, [BYTEARRAY a, INT i], [c]) =
        (fn s => c [INT(fetchi s(nth(a, i)))] s)
    | evalprim (CPS.:=, [a, v], [c]) =
        evalprim(CPS.update, [a, INT 0, v], [c])
    | evalprim (CPS.update, [ARRAY a, INT n, v], [c]) =
        (fn s => c [] (upd(s, nth(a, n), v)))
    | evalprim (CPS.update, [UARRAY a, INT n, INT v], [c]) =
        (fn s => c [] (updi(s, nth(a, n), v)))
    | evalprim (CPS.unboxedassign, [a, v], [c]) =
        evalprim(CPS.unboxedupdate, [a, INT 0, v], [c])
    | evalprim (CPS.unboxedupdate,
                [ARRAY a, INT n, INT v], [c]) =
        (fn s => c [] (upd(s, nth(a, n), INT v)))
    | evalprim (CPS.unboxedupdate,
                [UARRAY a, INT n, INT v], [c]) =
            (fn s => c [] (updi(s, nth(a, n), v)))
    | evalprim (CPS.store,
                [BYTEARRAY a, INT i, INT v], [c]) =
            if v < 0 orelse v >= 256
                then raise Undefined
                else (fn s => c [] (updi(s, nth(a, n), v)))
    | evalprim (CPS.makeref, [v], [c]) = (fn (l, f, g) =>
        c [ARRAY[l]] (upd((nextloc l, f, g), l, v)))
    | evalprim (CPS.makerefunboxed, [INT v], [c]) = (fn (l, f, g) =>
        c [UARRAY[l]] (updi((nextloc l, f, g), l, v)))
    | evalprim (CPS.alength, [ARRAY a], [c]) =
        c [INT(List.length a)]
    | evalprim (CPS.alength, [UARRAY a], [c]) =
        c [INT(List.length a)]
    | evalprim (CPS.slength, [BYTEARRAY a], [c]) =
        c [INT(List.length a)]
    | evalprim (CPS.slength, [STRING a], [c]) =
        c [INT(String.size a)]
    | evalprim (CPS.gethdlr, [], [c]) =
        (fn s => c [fetch s handler_ref] s)
    | evalprim (CPS.sethdlr, [h], [c]) =
        (fn s => c [] (upd(s, handler_ref, 1)))
    | evalprim (CPS.fadd, [REAL a, REAL b], [c]) =
        overflowr(fn()=>a+b, c)
    | evalprim (CPS.fsub, [REAL a, REAL b], [c]) =
        overflowr(fn()=>a-b, c)
    | evalprim (CPS.fmul, [REAL a, REAL b], [c]) =
        overflowr(fn()=>a*b, c)
    | evalprim (CPS.fdiv, [REAL a, REAL 0.0], [c]) =
        do_raise div_exn
    | evalprim (CPS.fdiv, [REAL a, REAL b], [c]) =
        overflowr(fn()=>a/b, c)
    | evalprim (CPS.feql, [REAL a, REAL b], [t, f]) =
        if a=b then t[] else f[]
    | evalprim (CPS.fneq, [REAL a, REAL b], [t, f]) =
        if a=b then f[] else t[]
    | evalprim (CPS.flt, REAL i, REAL j], [t, f]) =
        if i<j then t[] else f[]
    | evalprim (CPS.fle, REAL i, REAL j], [t, f]) =
        if j<=i then f[] else t[]
    | evalprim (CPS.fgt, REAL i, REAL j], [t, f]) =
        if j>i then t[] else f[]
    | evalprim (CPS.fge, REAL i, REAL j], [t, f]) =
        if i>=j then f[] else t[]

type env = CPS.var -> dvalue

(* `Value` to `DValue` using environment. *)
fun V env (CPS.INT i) = INT i
    | V env (CPS.REAL r) = REAL(string2real r)
    | V env (CPS.STRING s) = STRING s
    | V env (CPS.VAR v) = env v
    | V env (CPS.LABEL v) = env v

fun bind(env:env, v:CPS.var, d) =
    fn w => if v=w then d else env w

fun bindn(env, v::vl, d::dl) =
    bindn(bind(env, v, d), vl, dl)
    | bindn(env, nil, nil) = env

(* Resolve field of a `Record`. *)
fun F (x, CPS.OFFp 0) = x
    | F (RECORD(l, i), CPS.OFFp j) = RECORD(l, i+j)
    | F (RECORD(l, i), CPS.SELp(j, p)) = F(nth(l, i+j), p)

(* Apply `ContinuationExpression` to an `Environment`. *)
fun E (CPS.SELECT(i, v, w, e)) env =
        let val RECORD(l, j) = V env v
            in E e (bind(env, w, nth(l, i+j)))
        end
    | E (CPS.OFFSET(i, v, w, e)) env =
        let val RECORD(l, j) = V env v
            in E e (bind(env, w, RECORD(l, i+j)))
        end
    | E (CPS.APP(f, vl)) env =
        let val FUNC g = V env f
            in g (map (V env) vl)
        end
    | E (CPS.RECORD(vl, w, e)) env =
        E e (bind(env, w,
             RECORD(map (fn (x, p) =>
            F(V env x, p)) vl, 0)))
    | E (CPS.SWITCH(v, el)) env =
        let val INT i = V env v
            in E (nth(el, i)) env
        end
    | E (CPS.PRIMOP(p, vl, wl, el)) env =
        evalprim(p,
            map (V env) vl,
            map (
                    fn e => fn al =>
                    E e (bindn(env, wl, al))
                )
            el
        )
    | E (CPS.FIX(fl, e)) env =
        let fun h r1 (f, vl, b) =
                FUNC(fn al => E b (bindn(g r1, vl, al)))
                and g r = bindn(r, map #1 fl, map (h r) fl)
            in E e (g env)
        end

val env0 = fn x => raise Undefined

fun eval (vl, e) dl = E e (bindn(env0, vl, dl))

end
