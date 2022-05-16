---
feature: typechecking
start-date: 2022-05-16
author: Yann Hamdaoui
---

# Typechecking

The goals of this RFC are the following:

1. Identify the main shortcomings of the current implementation of typechecking
   in particular with respect to the interaction with untyped code.
2. Write a proper formalization of a type system and corresponding type
   inference algorithm which overcomes those limitations, if possible. At least,
   a proper formalization and clarification of the current type system would
   already be a positive outcome.

The motivation for the first step is self-explanatory: a more expressive type
system directly translate to users being able to write more programs (or clearer
versions of the same program, without superfluous annotations).

The second step is motivated by improving the experience of extending the type
system in the future and maintaining the Nickel codebase. We already hit edge
cases that led to unsatisfying [ad-hoc
fixes](https://github.com/tweag/nickel/pull/586). The current implementation
interleaves a number of different phase, which makes it harder to get into and
to modify. Finally, a clean and well designed specification often leads to a
simpler implementation, by removing accumulations of ad-hoc treatments that
become subsumed by a more generic case.

## 

## Motivation

This sections attempts to motivate the goals of the RFC in practical terms.
There is a consequent literature on type systems for programming languages, and
many variations have been explored and implemented in a myriad of languages,
some of them both cutting-edge and of industrial strength. For the purely static
part, we shall inspiration from other ML languages (or their remote cousins)
such as Haskell, OCaml, Scala, Typescript, Purescript, etc. The role of the
static type system of Nickel is to be able to type seamlessly various generic
functions operating on primitive types, and to do so doesn't seem to require new
developments or very fancy types. In the spirit of the current implementation,
an ML-like polymorphic type system with row types should do.

However, Nickel is also peculiar. Statically typed code co-exists with
dynamically typed code. And while we often brand Nickel as a gradually typed
language for simplicity, it technically isn't really. So-called gradual type
systems, derived from the original work of Siek and Taha, statically accept
unsafe conversions from and to the dynamic type `Dyn` (and more complex types
like `Dyn -> Num` and `Num -> Num`: such compatible types are said to be
_consistent_ with each others). Such implicit conversions may or may not be
guarded at runtime by a check (sound vs unsound gradual typing).

```nickel
# If Nickel was gradually typed, this would be accepted
{
  add : Num -> Num -> Num = fun x y => x + y,
  mixed : Dyn -> Num -> Num = fun x y => add x (y + 1),
}
```

In Nickel, such implicit conversions are purposely not supported. Running this
examples gives:

```text
error: incompatible types
  ┌─ repl-input-0:3:46
  │
3 │   mixed : Dyn -> Num -> Num = fun x y => add x (y + 1),
  │                                              ^ this expression
  │
  = The type of the expression was expected to be `Num`
  = The type of the expression was inferred to be `Dyn`
  = These types are not compatible
```

However, Nickel has contract annotations, which are in some way an explicit
version of the usually implicit casts of gradual typing. If the user wants to
use a dynamic value in statically typed code, they just have to write a contract
annotation that makes clear which type they expect the expression to have:

```nickel
# this is accepted
{
  add : Num -> Num -> Num = fun x y => x + y,
  mixed : Dyn -> Num -> Num = fun x y => add (x | Num) (y + 1),
}
```
