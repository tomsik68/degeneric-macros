//! # Quick Start
//!
//! ```
//! use degeneric_macros::{Degeneric, named_requirement};
//! use std::marker::PhantomData;
//!
//! named_requirement!(FactoryFn<T>: 'static + Send + Sync + Fn() -> T);
//!
//! #[derive(Degeneric)]
//! struct Container<T: Default, A: FactoryFn<T>, B> {
//!     a: A,
//!     b: B,
//!     c: u32,
//!     _t: PhantomData<T>,
//! }
//!
//! fn my_fact() -> String {
//!     format!("hello world!")
//! }
//!
//! let c = Container::builder()
//!     .with_a(my_fact)
//!     .with_b(true)
//!     .with_c(20)
//!     .with__t(Default::default())
//!     .build();
//!
//! fn do_something(c: impl ContainerTrait) {}
//! fn access_inner_types<C: ContainerTrait>(c: C) {
//!     let same_as_a: C::A;
//! }
//! ```
//!
//! # Elevator pitch
//!
//! ## The problem
//!
//! Degeneric is a utility library that solves the common problem of having too many generics.
//! Let's say we want to construct a dependency container like this:
//! ```
//! struct Container<Logger, HttpClient> {
//!     logger: Logger,
//!     client: HttpClient,
//!     // ...and so on...
//! }
//!
//! let container = Container {
//!     logger: String::from("logger"),
//!     client: String::from("http"),
//! };
//!
//! accepts_container(container);
//! // now to consume such a container, one needs to write the function like this:
//! fn accepts_container<Logger, HttpClient>(c: Container<Logger, HttpClient>) {}
//! ```
//!
//! This creates a problem of ever growing list of generics in all functions that touch the
//! container and pollutes APIs with unnecessary generics.
//!
//! ## Degeneric solution
//!
//! Degeneric proposes solution to this problem by creating a trait and stuffing all of the generic
//! types into the trait as associated types. Instead of the pattern above, you'll end up with
//! this:
//! ```
//! use degeneric_macros::Degeneric;
//!
//! #[derive(Degeneric)]
//! struct Container<Logger, HttpClient> {
//!     logger: Logger,
//!     client: HttpClient,
//! }
//!
//! let c = Container::builder()
//!     .with_logger(String::from("logger"))
//!     .with_client(String::from("http"))
//!     .build();
//!
//! accepts_container(c);
//! fn accepts_container(c: impl ContainerTrait) {}
//! ```
//!
//! How is this different, you ask? Instead of accepting a whole lot of generic arguments, I can now write
//! the function without even using angular brackets and I think that's beautiful.
//! What is even more beautiful is that you can add more generics without having to modify the
//! signature of `accepts_container`.
use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::ItemStruct;

mod degeneric;
mod named_requirement;
mod type_tools;

#[proc_macro_derive(Degeneric)]
/// Usable only on structs.
///
/// Example:
/// ```
/// use degeneric_macros::{Degeneric, named_requirement};
///
/// use std::cmp::PartialEq;
/// use std::fmt::Debug;
///
/// named_requirement!(Peq<T>: PartialEq<T> + Debug);
///
/// #[derive(Degeneric)]
/// struct Container<A: Peq<i32>, B: Peq<bool>> {
///     a: A,
///     b: B,
///     c: u32,
/// }
///
/// let c = Container::builder().with_a(10).with_b(true).with_c(42).build();
/// test_container(c);
/// fn test_container(c: impl ContainerTrait) {
///     assert_eq!(c.a(), &10);
/// }
/// ```
pub fn degeneric(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let tokens = degeneric::process_struct(&input);
    TokenStream::from(tokens)
}

#[proc_macro]
/// This macro helps to create "named requirements".
///
/// Named requirements are traits without methods that can be used instead of naming the long chain
/// of bounds.
///
/// Problem:
/// ```
/// struct FunctionStorage<F: 'static + Send + Sync + Sized + Fn() -> String> {
///     f: F,
/// }
///
/// impl<F: 'static + Send + Sync + Sized + Fn() -> String> FunctionStorage<F> {
///     pub fn new(f: F) -> Self {
///         Self { f }
///     }
/// }
///
/// fn accept_function_storage<F: 'static + Send + Sync + Sized + Fn() -> String>(f: FunctionStorage<F>) {}
/// ```
///
/// Solution:
/// ```
/// use degeneric_macros::named_requirement;
/// named_requirement!(Func: 'static + Send + Sync + Sized + Fn() -> String);
///
/// struct FunctionStorage<F: Func> {
///     f: F,
/// }
///
/// impl<F: Func> FunctionStorage<F> {
///     pub fn new(f: F) -> Self {
///         Self { f }
///     }
/// }
///
/// fn accept_function_storage<F: Func>(f: FunctionStorage<F>) {}
/// ```
///
/// How it works in the background:
/// ```
/// use degeneric_macros::named_requirement;
/// named_requirement!(Func: 'static + Send + Sync + Sized + Fn() -> String);
/// // generates this:
/// trait Func_ : 'static + Send + Sync + Sized + Fn() -> String {}
/// impl<T: 'static + Send + Sync + Sized + Fn() -> String> Func_ for T {}
/// ```
///
pub fn named_requirement(input: TokenStream) -> TokenStream {
    use named_requirement::*;
    let input = parse_macro_input!(input as NrInput);
    let tokens = named_requirement::process_input(input);
    TokenStream::from(tokens)
}
