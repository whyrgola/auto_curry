# auto_curry
Procedural macro for currying most functions in Rust.
This procedural macro can be thought of as a complete version of
the unfinished Rust library, [`cutlass`](https://crates.io/crates/cutlass).

## `add` Example (/examples/add.rs):
``` rust
use auto_curry::curry;

#[curry]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    assert_eq!(add(1)(2), 3);

    println!("{} = {}", add(1)(2), 3);
}
```

## How it expands:
As of version 0.1.0, the add function in the example above expands exactly to:
``` rust
fn add(a: i32) -> impl Fn(i32) -> i32 {
    move |b| { a + b }
}
```
As far as I am aware, this is the most performant expansion in stable rust.

## Capabilities:
1. Can handle funtions with a self receiver.
2. Can handle functions with generics and GAT's.
3. Works on stable.

## Known issues:
- waiting on `impl_trait_in_fn_trait_return` (or alternatively, `type_alias_impl_trait`) to be able to significantly
optimize curried functions.
- cannot curry pure function signatures without bodies, like those seen in traits
- somewhat clunky code for Boxing series of `Fn`'s
- limited number of tests and examples

The last 3 known issues are ones that will be resolved as development of the library progresses.
