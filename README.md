[![GitHub license](https://img.shields.io/github/license/tomsik68/degeneric-macros)](https://github.com/tomsik68/degeneric-macros/blob/master/LICENSE)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/tomsik68/degeneric-macros/lint?style=for-the-badge)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/tomsik68/degeneric-macros/publish?label=publish&style=for-the-badge)
![Crates.io](https://img.shields.io/crates/v/degeneric-macros?style=for-the-badge)

# degeneric-macros

## Quick Start

```toml
degeneric-macros = "0.2.0"
```

```rust
use degeneric_macros::{Degeneric};
use std::marker::PhantomData;

use trait_set::trait_set;
use typed_builder::TypedBuilder;

trait_set!(trait FactoryFn<T> = 'static + Send + Sync + Fn() -> T);

#[derive(Degeneric, TypedBuilder)]
struct Container<T: Default, A: FactoryFn<T>, B> {
    a: A,
    b: B,
    c: u32,
    #[builder(default)]
    _t: PhantomData<T>,
}

fn my_fact() -> String {
    format!("hello world!")
}

let c = Container::builder().a(my_fact).b(true).c(20).build();
do_something(&c);
access_inner_types(&c);

fn do_something(c: &impl ContainerTrait) {}
fn access_inner_types<C: ContainerTrait>(c: &C) {
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

let c = Container {
    logger: String::from("logger"),
    client: String::from("http"),
};

accepts_container(c);
fn accepts_container(c: impl ContainerTrait) {}
```

How is this different, you ask? Instead of accepting a whole lot of generic arguments, I can now write
the function without even using angular brackets and I think that's beautiful.
What is even more beautiful is that you can add more generics without having to modify the
signature of `accepts_container`.

## Degeneric understands lifetimes

```rust
use std::borrow::Cow;
use std::fmt::Debug;

use degeneric_macros::{Degeneric};
use typed_builder::TypedBuilder;

#[derive(Degeneric, TypedBuilder)]
struct Container<'a, T: 'a + PartialEq<i32> + Debug> {
    cow: &'a Cow<'a, str>,
    reference: &'a T,
}

let cow = Cow::Owned(String::from("hello lifetimes"));
{
    let reference = 42;
    let c = Container::builder().cow(&cow).reference(&reference).build();

    fn accept_container<'a>(cont: &impl ContainerTrait<'a>) {
        assert_eq!(cont.cow().as_ref(), "hello lifetimes");
        assert_eq!(*cont.reference(), &42_i32);
    }

    accept_container(&c);
}
```

## Crates degeneric plays nice with

+ [trait-set](https://lib.rs/trait-set) - shorten and DRY up trait bounds
+ [typed-builder](https://lib.rs/typed-builder) - generate a builder for your trait
+ [easy-ext](https://lib.rs/easy-ext) - extend your trait with more methods

License: MIT
