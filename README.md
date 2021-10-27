[![Rust build](https://github.com/tomsik68/degeneric-macros/actions/workflows/rust.yml/badge.svg)](https://github.com/tomsik68/degeneric-macros/actions/workflows/rust.yml)
![Crates.io](https://img.shields.io/crates/v/degeneric-macros?style=for-the-badge)

# degeneric-macros

## Quick Start

```rust
use degeneric_macros::{Degeneric, named_requirement};
use std::marker::PhantomData;

named_requirement!(FactoryFn<T>: 'static + Send + Sync + Fn() -> T);

#[derive(Degeneric)]
struct Container<T: Default, A: FactoryFn<T>, B> {
    a: A,
    b: B,
    c: u32,
    _t: PhantomData<T>,
}

fn my_fact() -> String {
    format!("hello world!")
}

let c = Container::builder()
    .with_a(my_fact)
    .with_b(true)
    .with_c(20)
    .with__t(Default::default())
    .build();

fn do_something(c: impl ContainerTrait) {}
fn access_inner_types<C: ContainerTrait>(c: C) {
    let same_as_a: C::A;
}
```

## Elevator pitch

### The problem

Degeneric is a utility library that solves the common problem of having too many generics.
Let's say we want to construct a dependency container like this:
```rust
struct Container<Logger, HttpClient> {
    logger: Logger,
    client: HttpClient,
    // ...and so on...
}

let container = Container {
    logger: String::from("logger"),
    client: String::from("http"),
};

accepts_container(container);
// now to consume such a container, one needs to write the function like this:
fn accepts_container<Logger, HttpClient>(c: Container<Logger, HttpClient>) {}
```

This creates a problem of ever growing list of generics in all functions that touch the
container and pollutes APIs with unnecessary generics.

### Degeneric solution

Degeneric proposes solution to this problem by creating a trait and stuffing all of the generic
types into the trait as associated types. Instead of the pattern above, you'll end up with
this:
```rust
use degeneric_macros::Degeneric;

#[derive(Degeneric)]
struct Container<Logger, HttpClient> {
    logger: Logger,
    client: HttpClient,
}

let c = Container::builder()
    .with_logger(String::from("logger"))
    .with_client(String::from("http"))
    .build();

accepts_container(c);
fn accepts_container(c: impl ContainerTrait) {}
```

How is this different, you ask? Instead of accepting a whole lot of generic arguments, I can now write
the function without even using angular brackets and I think that's beautiful.
What is even more beautiful is that you can add more generics without having to modify the
signature of `accepts_container`.
