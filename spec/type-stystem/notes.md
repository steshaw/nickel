# Type system formalization

Those are personal working notes, provided as they are, without much structure.

## Quick Look

Is based on the idea of type guards. It means than most type constructors are
invariant. How limiting would that be for Nickel?

How do we know the type of arguments? Quick look just peeks at the type, so that
we can choose where to stop. In the paper they do only variables and nested
calls.

For us, there is no fallback case (monomorphic instantiation). Another
possibility, suggested by Richard: A Deep Look into subtyping. We would do full
type inference inside arguments, then check for a minimum. If none exist,
probably the typechecking will fail anyway? Otherwise, instantiate to the
maximum.

But, is doing inference then subtype checking as general as checking mode?
Probably not (see, for example, a rule for `Array Dyn`, versus an infer `Array _a
<: Array Dyn`).

## Baseline

One important point is: what are we trying to achieve? What exactly do we want
to type? When do we want to type them?

I'm personally not sold on impredicative polymorphism. It would have been nice
in, say, OCaml, but I haven't often said "oh my god if only we had impredicative
polymorphism" (although, we can still use it by boxing values in a record). It
does make thing a bit more complex, since they are order dependent. How does
haskell proceed, in the monomorphic case? Can Quick Look be combined with the
subtyping approach of Krishnawsami et al? If yes, then ok, why not.

**Correction**: it can't really. K. et al do deep instantiation (`1 -> forall a.
a <: 1 -> 1`) The whole point of K. is to preserve typeability in presence of
$\eta$-laws. But preserving $\eta$ plus impredicativity leads to undecidability.
See below. What we could do though, is to combine subtyping-like judgement of
K. et al. and adapt to impredicative polymorphism? What would be the gain,
though?

For subtyping: how often do we really want `T <: Dyn` ? Couldn't this just be
the checking rule of `Dyn`, instead of being a subtyping rule?

I can also see `{foo: T, bar: T} <: {_ : T}` being useful. Maybe we can remove
covariance of lists, co/contravariance of functions, covariance of records, if
that proves to be annoying when combined with Tabby-first problem.

Examples of what we want to typecheck?

### Several subtyping

If we end up with invariance for subtyping, does it really make sense to burry
instantiation into subtying? Maybe we should rather have:

1. A decidable, variant relation, for records and co.
2. Instantiation, that is mostly invariant

## Easy and complete bidir inference for higher rank polymorphism

The do deep instantiation through subtyping: typically, `1 -> forall a. a <: 1
-> 1`. For some reasons, Arnaud doesn't want this. Their core reference calculus
already has this baked in, to make $\eta$ laws preserve typing.

> Preserving the η-rule for functions comes at a cost. The subtyping relation
> induced by instantiation is undecidable for impredicative polymorphism

That explains why we may not want deep instantiation.

Side idea: do we care, if people need to eta-extend stuff? Could we split the
subtyping relations, to have different rules for different subtyping relations?

## $ML^f$

In systemF, there are incompatible types. The idea is to constrain types to
recover principality, to say: this variable can either be instantiated only by a
polymorphic type, or by a polytype OR their instances.

```mlf
auto : forall a. (a = sigma_id) a -> a
choose id : forall a. (a >= sigma_id) a -> a
```

## Plan

- We don't care about $\eta$-reduction. No deep instantiation. That means the
  approach of Krishnawasmi et al. is not really interesting, so let's rather go
  with a PTIAT + QuickLook.
- Do not instantiate stuff to object as a fallback. Rather try to find `max {
  possible instantiations }`. Possible instantiations being determined with a
  Quick-Look like phase, as for polymorphism. But then, we may want to have a
  of ML subtyping then. One possibility: infer, then determine the sutff. Do not
  loose inference, then.
- For subtyping, let's do something easy. As for QuickLook: try to do the
  not-too-stupid-stuff. If that doesn't work, fall back to "stupid,
  order-dependent" instantiation. Probably even if that doesn't work
  typechecking will fail anyway.

## Type system

- no sumsumption rule for polymorphism (we lose $\eta$-rules for
polymorphism).
- separate subtyping for `Dyn` types + `dict < record` from
    polymorphism-inducedsubtyping
- PTIAT + Quick-Look. Start with predicative PTIAT. Enhance with QuickLook after.
- QuickLook like-phase for subtyping. Start with simple inference. Later on may
  perform full type inference, with a way of reusing it. If this fails, then
  fallback to order-dependence of parameters.

```nickel
m : forall a. Array (a -> Num) -> a -> (Array Num, a)
m (Array ({_: Num} -> Num) {foo = 1}
a
Array (a -> Num) Array ({_ Num} -> Num)
match a <: {_: Num}
match a >: {foo : Num}
{foo : Num} <: a <: {_: Num} => coherence of lower bounds + max of lower bounds inf (if it exists) + coherence of upper bounds
```

### Join

```
join : { instantiations } -> Maybe Type
fst {foo = 1} {bar = 2}
```

```
join: find the min type inside the set
is_sub without unification
more involved join: {foo : Num} @+@ {bar : Num} => {_ : Num}
```

### First draft

- PTIAT without deep instantiation / sumsumption
- Quick Look at Subtyping without any bounds checking. Means subtyping is
  invariant (NO: actually, this gives unsound advice, that worst than the
  default thing: see example array above).

### Second iteration

- Quick look at polymorphism
- Quick look at subtyping: use bounds recording.

### Third iteration

- Deep look at subtyping: use MLSub like type relation to record bounds
  around instantiation variable. Depend on the subtyping relation.

## Call w/ Arnaud

```nickel
{_ : Num}
{foo : Num}

record.update "foo" 2 {foo = 1}

let x = {foo = 1} in
record.update "foo" 2 x

(let x = { foo = 1 } in x : { _ : Num })

?a -> ?b <: ?c
(x : Dyn) : ?c

builtin.is_num "foo"

let f = fun x => ... ( type apparent Dyn -> Dyn)

g : forall a. Num -> a
let g' : Dyn = g in
let g'' | Num -> Dyn = g in
builtin.is_num (g'' 0)

Array a <: Array Dyn

first : forall a. a -> a -> a
let foo : Num = .. in
let bar : Dyn = .. in
first bar foo (a >: Num, Dyn) => a := num
first foo bar
```

`(u : forall a. t)` instantiate eagerly (ForallRight) when checking polymorphic
type.

pb: `Dyn :> ?a` because `?a` may become a rigid type variable.
we never infer dyn if there is no Dyn written in the term.

```nickel
if b then 2 else "foo" # Fail

(if b then 2 else "foo") : Dyn #succeed
```

Idea: take the approach of Parreaux. Record upper and lower bound of unification
variables, and solve in phases.
1st phase : inequalities that comes down to unification.
2nd phase : `?a >: _` -> unification: we never infer Dyn
3rd phase : Only `Dyn >: ?a`. Either it's unsound, or we don't care.

`Num -> forall a. t   ?= ?b -> ?c`: this may introduce forall on the left. What
to do with this? We want to avoid having foralls being part of the subtyping
relation.

### Example of inference

Let's try to see an example:

```nickel

let foo : Num -> forall a. (a -> a) = fun _ => id in

fun f => let id = f 0 in f == foo # (==): forall a. a -> a -> bool
```

#### 1st phase

```math
_a >: Num -> forall a. (a -> a)
_a >: _b
_b <: _c -> _d
_c >: Num
_c -> _d =?
_f <: _b -> Bool
```

Generates

```math
_f := _b -> Bool

_b : should we order the thing?
Should we perform unidirecitonal unification?

_b := _c -> _d
_b := _a
_a := _c -> _d ?

After:
_c -> _d >: Num -> forall a. (a -> a)
_c >: Num
```

#### 2nd phase

```math
_c -> _d >: Num -> forall a. (a -> a)
_c >: Num
```

Generates

```math
_c <: Num
_c >: Num
_d >: forall a. (a -> a)
```

#### 3rd phase

```math
_c := Num
_d := forall a. (a -> a)
```

### Other examples (polymorphic instantiation + subtyping)

```math
f : forall a. a -> a -> a
let g = f {foo : Num} {_ : Num}

_a >: {foo : _a}
_a >: {_ : _b}
```

So what to do when `_a >: { ... }` ?
$\exists i . \forall j, t_j \lt t_i$

QuickMin: eliminate cases of Dyn.
We would like to characterize what works: if there is no Dyn in code, it should
be equivalent to unification.

What algorithm to use to do that? Causes unfications?
