//! [![GitHub license](https://img.shields.io/github/license/tomsik68/degeneric-macros?style=for-the-badge)](https://github.com/tomsik68/degeneric-macros/blob/master/LICENSE)
//! [![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/tomsik68/degeneric-macros/rust.yml?branch=master&style=for-the-badge)](https://github.com/tomsik68/degeneric-macros/actions/workflows/rust.yml)
//! [![Crates.io](https://img.shields.io/crates/v/degeneric-macros?style=for-the-badge)](https://crates.io/crates/degeneric-macros)
//! [![Crates.io (latest)](https://img.shields.io/crates/dv/degeneric-macros?style=for-the-badge)](https://crates.io/crates/degeneric-macros)
//!
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
//! #[degeneric(trait_decl = "pub trait ContainerTrait")]
//! /// This is doc for ContainerTrait!
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
//! #[degeneric(trait_decl = "pub trait ContainerTrait")]
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
//! #[degeneric(trait_decl = "trait ContainerTrait")]
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
//!         assert_eq!(cont.reference(), &42_i32);
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
//! The way this example works is, that your Container contains an impl GCon. This object is able
//! to produce [`galemu::Bound`]`<GCon::Transaction>`.
//!
//! The particular implementation of `GTran` is provided by [`galemu::create_gal_wrapper_type`].
//! One must manually implement GTran on it.
//!
//! In principle, galemu lifts the lifetime of `Transaction<'a>` into the [`galemu::BoundExt`] trait.
//! The lifetime inference happens in `Connection::transaction`. At that point, it's apparent that
//! the connection's lifetime is passed to Transaction.
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
//! #[degeneric(trait_decl = "pub trait ContainerTrait")]
//! struct Container<T: GCon> {
//!     conn: T,
//! }
//!
//! let conn = Connection { count : 0 };
//!
//! let cont = Container {
//!     conn,
//! };
//!
//! fn commit_transaction(mut c: impl ContainerTrait) {
//!     let conn = c.conn_mut();
//!     let tran = conn.create_transaction();
//!     GTran::commit(tran);
//! }
//!
//! commit_transaction(cont);
//! ```
//!
//! ## Degeneric + dynamize
//!
//! Degeneric supports dynamizing the generated trait. How does that work?
//!
//! Here's a minimal example on how to dynamize the generated trait:
//!
//! ```
//! use degeneric_macros::Degeneric;
//!
//! #[derive(Degeneric)]
//! #[degeneric(dynamize, trait_decl = "pub trait GeneratedContainerTrait")]
//! struct Container<T: std::any::Any> {
//!     item: T,
//! }
//! ```
//!
//! By convention, dynamize generates a `DynGeneratedContainerTrait` where the types are boxed.
//! Please refer to [dynamize documentation](https://docs.rs/dynamize/latest/dynamize/#dynamize)
//! for more information.
//!
//! ## Degeneric + haz
//!
//! Degeneric is able to serve as a derive macro for the excellent
//! [`haz`](https://crates.io/crates/haz) crate.
//!
//! ```
//! use degeneric_macros::Degeneric;
//! use haz::Has;
//!
//! # #[derive(Default)]
//! # struct Host;
//! # #[derive(Default)]
//! # struct Port;
//! # #[derive(Default)]
//! # struct Verbosity;
//! # #[derive(Default)]
//! # struct Restriction;
//!
//!  #[derive(Degeneric, Default)]
//!  #[degeneric(haz)]
//!  struct Config {
//!    host: Host,
//!    port: Port,
//!    verbosity: Verbosity,
//!    restriction: Restriction,
//!  }
//!
//!  fn assert_has_all_the_things<T: Has<Host> + Has<Port> + Has<Verbosity> + Has<Restriction>>(_: T) {}
//!  assert_has_all_the_things(Config::default());
//! ```
//!
//! # Degeneric understands where clause
//!
//! ```
//! use degeneric_macros::{Degeneric};
//! use std::fmt::Debug;
//!
//! #[derive(Degeneric)]
//! #[degeneric(trait_decl = "pub trait ContainerTrait")]
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
//! # Generate getters only for some fields
//!
//! The `no_getter` attribute can be used to skip generating a getter.
//!
//! ```compile_fail
//! use degeneric_macros::{Degeneric};
//!
//! #[derive(Degeneric)]
//! #[degeneric(trait_decl = "pub(crate) trait Something")]
//! struct Container<'a, T: 'a, S: 'a> {
//!     item: &'a T,
//!     item2: S,
//!     #[degeneric(no_getter)]
//!     dt: PhantomData<S>,
//! }
//!
//! let c = Container {
//!     item: "hello",
//!     item2: format!("this won't have getter!"),
//!     dt: PhantomData<S>,
//! };
//!
//! fn accept_container<C: Something>(c: C) {
//!     /// ERROR: dt doesn't have a getter!
//!     assert_eq!(c.dt(), format!("this won't have getter!"));
//! }
//!
//! accept_container(c);
//! ```
//!
//! # Degeneric figures out mutability
//!
//! Some fields may have mutable getters, some not. Degeneric recognizes immutable pointers and
//! references and skips generating mutable getter for them.
//!
//! ```
//! use degeneric_macros::{Degeneric};
//! #[derive(Degeneric)]
//! #[degeneric(trait_decl = "pub(crate) trait Something")]
//! struct Container<'a, T: 'a> {
//!     x: &'a T,
//!     y: T,
//! }
//!
//! let mut c = Container {
//!     x: &(),
//!     y: (),
//! };
//!
//! fn accept_container<'a>(mut c: impl Something<'a>) {
//!     // OK
//!     c.x();
//!     c.y();
//!     c.y_mut();
//! }
//!
//! accept_container(c);
//! ```
//!
//! ```compile_fail
//! use degeneric_macros::{Degeneric};
//!
//! #[derive(Degeneric)]
//! #[degeneric(trait_decl = "pub(crate) trait Something")]
//! struct Container<'a, T> {
//!     x: &'a T,
//! }
//!
//! let c = Container {
//!     x: &(),
//! };
//!
//! fn accept_container<'a>(c: impl Something<'a>) {
//!     // ERROR: x is a reference which can't be made mut
//!     c.x_mut();
//! }
//! ```
//!
//! # Add attributes everywhere!
//!
//! For some attributes, you can just add them on the field and they'll be forwarded to all getters automatically.
//! Here's a list of such attributes:
//! - `#[allow]`
//! - `#[doc]`
//! - `#[cfg(...)]`
//! - `#[cfg_attr(...)]`
//!
//! If you need more granularity, you can add attributes only on:
//!
//! - Trait declaration: `#[degeneric(trait_decl_attr = "#[doc = \"Trait declaration\"]")]`
//! - Trait impl block: `#[degeneric(trait_impl_attr = "#[doc = \"Trait implementation\"]")]`
//! - Field immutable getter implementation: `#[degeneric(getter_impl_attr = "#[doc = \"Getter implementation\"])]`
//! - Field mutable getter declaration: `#[degeneric(mut_getter_decl_attr = "#[doc = \"Mutable Getter declaration\"])]`
//!
//! ```compile_fail
//! use degeneric_macros::Degeneric;
//!
//! #[derive(Degeneric)]
//! #[degeneric(trait_decl = "pub(crate) trait Something")]
//! #[degeneric(trait_decl_impl_attr = "#[cfg(foo)]")]
//! /// This is documentation for the `Something` trait
//! struct Container<T> {
//!     x: T,
//! }
//!
//! // this will error because the Something trait exists only in the foo configuration
//! #[cfg(not(foo))]
//! fn accept_container(c: impl Something) {}
//! ```
//!
//! # Crates degeneric plays nice with
//!
//! + [galemu](https://lib.rs/galemu) - hide lifetimes!
//! + [trait-set](https://lib.rs/trait-set) - shorten and DRY up trait bounds
//! + [typed-builder](https://lib.rs/typed-builder) - generate a builder for your trait
//! + [easy-ext](https://lib.rs/easy-ext) - extend your trait with more methods
//!
//! # CloneExt
//!
//! Apart from solving the dependency injection problem, degeneric also helps with cloning.
//! There might be a situation where you're holding a non-cloneable type inside another type. In
//! these situations, it might be possible to clone the value by different means.
//!
//! Failing example:
//!
//! ```compile_fail
//!
//! #[derive(Default)]
//! struct NonClone;
//!
//! #[derive(Clone)]
//! struct Container {
//!     nc: PhantomData<NonClone>,
//! }
//! ```
//!
//! In such situations, one can resort to degeneric's CloneExt derive macro. Currently, it
//! offers a single attribute to adjust the way fields are cloned:
//!
//! ```
//! #[derive(Default)]
//! struct NonClone;
//!
//! #[derive(Default, degeneric_macros::CloneExt)]
//! struct Container {
//!     #[clone_ext(clone_behavior(call_function="Default::default"))]
//!     nc: NonClone,
//! }
//!
//! Container::default().clone();
//! ```

/// proc_macro_error unwrap
macro_rules! pme_unwrap {
    ($e:expr, $span:expr, $($args:expr),*) => {
        {
            match $e {
                Ok(x) => x,
                #[allow(unused)]
                Err(err) => {
                    proc_macro_error::abort!($span, $($args),*);
                }
            }
        }
    };
}

mod clone_ext;
mod degeneric;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

#[proc_macro_derive(Degeneric, attributes(degeneric))]
#[proc_macro_error]
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
/// #[degeneric(trait_decl = "trait ContainerTrait")]
/// // attribute for both trait declaration and trait impl
/// #[degeneric(trait_impl_attr = "#[cfg(not(foo))]")]
/// /// ContainerTrait contains the implementation of `A` and `B` types.
/// struct Container<
///     /** The A type is the more important one. */ A: PartialEq<i32> + Debug,
///     /** You could live without B I guess. */ B: PartialEq<bool> + Debug> {
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
    let input = parse_macro_input!(input);
    let tokens =
        self::degeneric::process_struct(&input).unwrap_or_else(|err| err.to_compile_error());
    TokenStream::from(tokens)
}

#[proc_macro_derive(CloneExt, attributes(clone_ext))]
#[proc_macro_error]
/// There might be a situation where you're holding a non-cloneable type inside another type. In
/// these situations, it might be possible to clone the value by different means.
///
/// Failing example:
///
/// ```compile_fail
///
/// #[derive(Default)]
/// struct NonClone;
///
/// #[derive(Clone)]
/// struct Container {
///     nc: PhantomData<NonClone>,
/// }
/// ```
///
/// In such situations, one can resort to degeneric's CloneExt derive macro. Currently, it
/// offers a single attribute to adjust the way fields are cloned:
///
/// ```
/// #[derive(Default)]
/// struct NonClone;
///
/// #[derive(Default, degeneric_macros::CloneExt)]
/// struct Container {
///     #[clone_ext(clone_behavior(call_function="Default::default"))]
///     nc: NonClone,
/// }
///
/// Container::default().clone();
/// ```
pub fn clone_ext(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let tokens =
        self::clone_ext::process_struct(&input).unwrap_or_else(|err| err.to_compile_error());
    TokenStream::from(tokens)
}
