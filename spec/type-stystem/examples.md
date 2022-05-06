# Collection of typing examples

## Multiple lower bounds

### With max

```nickel
f : forall a. a -> a -> a
let x : {foo : {bar: Num}, bar: {baz2: Dyn}} = .. in
let y : {_: Num} = .. in
f x y

# constraints
a: ?a
?a >: {foo : {baz: Num}, bar: {baz2: Dyn}}
?a >: {_ : {_ : Dyn}}

# expected
works
a instantiated to {_ : {_ : Dyn}}
```

## Multiple upper bounds

```nickel
fun x =>
  if (builtin.is_num x) then x + 1 else x

x: ?a
?a <: Dyn
?a <: Num

# expected
works
x: Num
```

```nickel
fun x =>
  if x then x + 1

x: ?a
?a <: Num
?a <: Bool

# expected
fails
Num <> Bool
```

```nickel
fun x =>
  let y : Dyn = null in
  let _ign = record.insert 1 "foo" x in
  record.insert null "bar" x

x: ?x
?a1 first instantiation
?a2 snd instantiation
?r return type

# fst call gives
Num <: ?a1
?x <: {_: ?a1}

# snd call gives
?a2 >: Dyn => a2 := Dyn
?x <: {_ : Dyn}
?r := {_ : Dyn}

# resolution, phase 1
# unification-like constraint
?x := {_: ?x1} <: {_: ?a1}
# generates
?x1 <: ?a1
?x1 <: Dyn

# state
Num <: ?a1
?x1 <: ?a1
?x1 <: Dyn

# what do we do? unify? what if ?x1 <: ?a3 ?
# Or we do ?a1 >: max (Num, ?x1), setting ?x1 to Num
# ... ?
# Would it be possible to have ?x1 <: ?a1, ?x2 <: ?a2, ?a1 <: Num, ?a2 <: Dyn
# plus other bounds preventing from doing substitution?

# expected
works
{_: Num} -> {_: Dyn}
```

Questions on this: what constraint do we pick? Do max of lower bound should
always provoke unification, like `max ?a Num`?

## Both lower and upper bounds
