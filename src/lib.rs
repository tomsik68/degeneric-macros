//! # Quick Start
//!
//! ```
//! use degeneric_macros::{Degeneric};
//! use std::marker::PhantomData;
//!
//! use trait_set::trait_set;
//! use typed_builder::TypedBuilder;
//!
//! trait_set!(trait FactoryFn<T> = 'static + Send + Sync + Fn() -> T);
//!
//! #[derive(Degeneric, TypedBuilder)]
//! struct Container<T: Default, A: FactoryFn<T>, B> {
//!     a: A,
//!     b: B,
//!     c: u32,
//!     #[builder(default)]
//!     _t: PhantomData<T>,
//! }
//!
//! fn my_fact() -> String {
//!     format!("hello world!")
//! }
//!
//! let c = Container::builder().a(my_fact).b(true).c(20).build();
//! do_something(&c);
//! access_inner_types(&c);
//!
//! fn do_something(c: &impl ContainerTrait) {}
//! fn access_inner_types<C: ContainerTrait>(c: &C) {
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
//! let c = Container {
//!     logger: String::from("logger"),
//!     client: String::from("http"),
//! };
//!
//! accepts_container(c);
//! fn accepts_container(c: impl ContainerTrait) {}
//! ```
//!
//! How is this different, you ask? Instead of accepting a whole lot of generic arguments, I can now write
//! the function without even using angular brackets and I think that's beautiful.
//! What is even more beautiful is that you can add more generics without having to modify the
//! signature of `accepts_container`.
//!
//! # Degeneric understands lifetimes
//!
//! ```
//! use std::borrow::Cow;
//! use std::fmt::Debug;
//!
//! use degeneric_macros::{Degeneric};
//! use typed_builder::TypedBuilder;
//!
//! #[derive(Degeneric, TypedBuilder)]
//! struct Container<'a, T: 'a + PartialEq<i32> + Debug> {
//!     cow: &'a Cow<'a, str>,
//!     reference: &'a T,
//! }
//!
//! let cow = Cow::Owned(String::from("hello lifetimes"));
//! {
//!     let reference = 42;
//!     let c = Container::builder().cow(&cow).reference(&reference).build();
//!
//!     fn accept_container<'a>(cont: &impl ContainerTrait<'a>) {
//!         assert_eq!(cont.cow().as_ref(), "hello lifetimes");
//!         assert_eq!(*cont.reference(), &42_i32);
//!     }
//!
//!     accept_container(&c);
//! }
//! ```
//!
//! ## Degeneric can be used with galemu!
//!
//! If you're into hiding generics, you'll be surprised that the [galemu](https://crates.io/crates/galemu) crate makes it possible to
//! hide even lifetimes!
//!
//! ```
//! use std::fmt::Debug;
//! use std::borrow::Cow;
//! use std::ops::Deref;
//!
//! use degeneric_macros::Degeneric;
//!
//! use galemu::{Bound, BoundExt, create_gal_wrapper_type};
//!
//! // begin galemu
//!
//! struct Connection {
//!     count: usize
//! }
//!
//! struct Transaction<'conn> {
//!     conn: &'conn mut Connection
//! }
//!
//! impl Connection {
//!     fn transaction(&mut self) -> Transaction {
//!         Transaction { conn: self }
//!     }
//! }
//!
//! trait GCon {
//!     type Transaction: GTran;
//!
//!     fn create_transaction(&mut self) -> Bound<Self::Transaction>;
//! }
//!
//! trait GTran: for<'s> BoundExt<'s> {
//!     fn commit<'s>(me: Bound<'s, Self>);
//!     fn abort<'s>(me: Bound<'s, Self>);
//! }
//!
//! create_gal_wrapper_type!{ struct TransWrap(Transaction<'a>); }
//!
//! impl GCon for Connection {
//!     type Transaction = TransWrap;
//!
//!     fn create_transaction(&mut self) -> Bound<Self::Transaction> {
//!         let transaction = self.transaction();
//!         TransWrap::new(transaction)
//!     }
//! }
//!
//! impl GTran for TransWrap {
//!     fn commit<'s>(me: Bound<'s, Self>) {
//!         let trans = TransWrap::into_inner(me);
//!         trans.conn.count += 10;
//!     }
//!
//!     fn abort<'s>(me: Bound<'s, Self>) {
//!         let trans = TransWrap::into_inner(me);
//!         trans.conn.count += 3;
//!     }
//! }
//!
//! // end galemu
//!
//! #[derive(Degeneric)]
//! struct Container<C: GCon> {
//!     conn: C,
//! }
//!
//! let cont = Container {
//!     conn: Connection { count: 0 },
//! };
//!
//! fn check_container(mut c: impl ContainerTrait) {
//!     GTran::commit(c.conn_mut().create_transaction())
//! }
//!
//! check_container(cont);
//!
//! ```
//!
//! # Degeneric understands where clause
//!
//! ```
//! use degeneric_macros::{Degeneric};
//! use std::fmt::Debug;
//!
//! #[derive(Degeneric)]
//! struct Container<T> where T: Default + Debug + PartialEq {
//!     item: T,
//! }
//!
//! let c = Container {
//!     item: vec![""],
//! };
//!
//! fn construct_default_value<C: ContainerTrait>(c: C) {
//!     let v: C::T = Default::default();
//!     assert_eq!(v, Default::default());
//! }
//!
//! construct_default_value(c);
//!
//!
//! ```
//!
//! # Crates degeneric plays nice with
//!
//! + [trait-set](https://lib.rs/trait-set) - shorten and DRY up trait bounds
//! + [typed-builder](https://lib.rs/typed-builder) - generate a builder for your trait
//! + [easy-ext](https://lib.rs/easy-ext) - extend your trait with more methods
use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::ItemStruct;

mod degeneric;
mod type_tools;

#[proc_macro_derive(Degeneric)]
/// Usable only on structs.
///
/// Example:
/// ```
/// use degeneric_macros::Degeneric;
///
/// use std::cmp::PartialEq;
/// use std::fmt::Debug;
///
/// #[derive(Degeneric)]
/// struct Container<A: PartialEq<i32> + Debug, B: PartialEq<bool> + Debug> {
///     a: A,
///     b: B,
///     c: u32,
/// }
///
/// let c = Container {
///     a: 10,
///     b: true,
///     c: 42,
/// };
///
/// test_container(c);
/// fn test_container(c: impl ContainerTrait) {
///     assert_eq!(c.a(), &10);
///     assert_eq!(c.b(), &true);
///     assert_eq!(c.c(), &42);
/// }
/// ```
pub fn degeneric(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let tokens = degeneric::process_struct(&input);
    TokenStream::from(tokens)
}
