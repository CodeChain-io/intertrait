//! A library providing direct casting among trait objects implemented by a type.
//!
//! In Rust, an object of a sub-trait of [`std::any::Any`] can be downcast to a concrete type
//! at runtime if the type is known. But no direct casting between two trait objects
//! (i.e. without involving the concrete type of the backing value) are possible
//! (even no coercion from a trait object to that of its super-trait yet).
//!
//! With this crate, any trait object with [`CastFrom`] as its super-trait can be cast directly
//! to another trait object implemented by the underlying type if the target traits are
//! registered beforehand with the macros provided by this crate.
//!
//! # Usage
//! ```
//! use intertrait::*;
//! use intertrait::cast::*;
//!
//! struct Data;
//!
//! trait Source: CastFrom {}
//!
//! trait Greet {
//!     fn greet(&self);
//! }
//!
//! #[cast_to]
//! impl Greet for Data {
//!     fn greet(&self) {
//!         println!("Hello");
//!     }
//! }
//!
//! impl Source for Data {}
//!
//! fn main() {
//!     let data = Data;
//!     let source: &dyn Source = &data;
//!     let greet = source.cast::<dyn Greet>();
//!     greet.unwrap().greet();
//! }
//! ```
//!
//! Target traits must be explicitly designated beforehand. There are three ways to do it:
//!
//! * [`#[cast_to]`][cast_to] to `impl` item
//! * [`#[cast_to(Trait)]`][cast_to] to type definition
//! * [`castable_to!(Type: Trait1, Trait2)`][castable_to]
//!
//! Refer to the documents for each of macros for details.
//!
//! For casting, refer to [`CastTo`].
//!
//! [cast_to]: ./attr.cast_to.html
//! [castable_to]: ./macro.castable_to.html
//! [`CastTo`]: ./trait.CastTo.html
//! [`Any`]: https://doc.rust-lang.org/std/any/trait.Any.html
use std::any::{Any, TypeId};
use std::collections::HashMap;

use linkme::distributed_slice;
use once_cell::sync::Lazy;

pub use intertrait_macros::*;

use crate::hasher::BuildFastHasher;

pub mod cast;
mod hasher;

#[doc(hidden)]
pub type BoxedCaster = Box<dyn Any + Send + Sync>;

#[cfg(doctest)]
doc_comment::doctest!("../README.md");

/// A distributed slice gathering constructor functions for [`Caster<T>`]s.
///
/// A constructor function returns `TypeId` of a concrete type involved in the casting
/// and a `Box` of a trait object backed by a [`Caster<T>`].
///
/// [`Caster<T>`]: ./struct.Caster.html
#[doc(hidden)]
#[distributed_slice]
pub static CASTERS: [fn() -> (TypeId, BoxedCaster)] = [..];

/// A `HashMap` mapping `TypeId` of a [`Caster<S, T>`] to an instance of it.
///
/// [`Caster<S, T>`]: ./struct.Caster.html
static CASTER_MAP: Lazy<HashMap<(TypeId, TypeId), BoxedCaster, BuildFastHasher>> =
    Lazy::new(|| {
        CASTERS
            .iter()
            .map(|f| {
                let (type_id, caster) = f();
                ((type_id, (*caster).type_id()), caster)
            })
            .collect()
    });

/// A `Caster` knows how to cast a reference to or `Box` of a trait object for `Any`
/// to a trait object of trait `T`. Each `Caster` instance is specific to a concrete type.
/// That is, it knows how to cast to single specific trait implemented by single specific type.
///
/// An implementation of a trait for a concrete type doesn't need to manually provide
/// a `Caster`. Instead attach `#[cast_to]` to the `impl` block.
#[doc(hidden)]
pub struct Caster<T: ?Sized + 'static> {
    /// Casts a reference to a trait object for `Any` to a reference to a trait object
    /// for trait `T`.
    pub cast_ref: fn(from: &dyn Any) -> &T,

    /// Casts a mutable reference to a trait object for `Any` to a mutable reference
    /// to a trait object for trait `T`.
    pub cast_mut: fn(from: &mut dyn Any) -> &mut T,

    /// Casts a `Box` holding a trait object for `Any` to another `Box` holding a trait object
    /// for trait `T`.
    pub cast_box: fn(from: Box<dyn Any>) -> Box<T>,
}

/// Returns a `Caster<S, T>` from a concrete type `S` to a trait `T` implemented by it.
fn caster<T: ?Sized + 'static>(type_id: TypeId) -> Option<&'static Caster<T>> {
    CASTER_MAP
        .get(&(type_id, TypeId::of::<Caster<T>>()))
        .and_then(|caster| caster.downcast_ref::<Caster<T>>())
}

/// `CastFrom` must be extended by a trait that wants to allow for casting into another trait.
///
/// It is used for obtaining an object of [`Any`] from an object of a sub-trait of `CastFrom`,
/// and blanket implemented for all `Sized + 'static` types.
///
/// # Examples
/// ```ignore
/// trait Source: CastFrom {
///     ...
/// }
/// ```
///
/// **Note**: [`CastFrom`] will become obsolete and be replaced with [`std::any::Any`]
/// once the [unsized coercion](https://doc.rust-lang.org/reference/type-coercions.html#unsized-coercions)
/// from a trait object to another for its super-trait is implemented in the stable Rust.
///
/// [`Any`]: https://doc.rust-lang.org/std/any/trait.Any.html
pub trait CastFrom: Any + 'static {
    /// Returns a immutable reference to `Any`, which is backed by the type implementing this trait.
    fn ref_any(&self) -> &dyn Any;

    /// Returns a mutable reference to `Any`, which is backed by the type implementing this trait.
    fn mut_any(&mut self) -> &mut dyn Any;

    /// Returns a `Box` of `Any`, which is backed by the type implementing this trait.
    fn box_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Sized + 'static> CastFrom for T {
    fn ref_any(&self) -> &dyn Any {
        self
    }

    fn mut_any(&mut self) -> &mut dyn Any {
        self
    }

    fn box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl CastFrom for dyn Any {
    fn ref_any(&self) -> &dyn Any {
        self
    }

    fn mut_any(&mut self) -> &mut dyn Any {
        self
    }

    fn box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::fmt::{Debug, Display};

    use linkme::distributed_slice;

    use crate::{BoxedCaster, CastFrom};

    use super::cast::*;
    use super::*;

    #[distributed_slice(super::CASTERS)]
    static TEST_CASTER: fn() -> (TypeId, BoxedCaster) = create_test_caster;

    #[derive(Debug)]
    struct TestStruct;

    trait SourceTrait: CastFrom {}

    impl SourceTrait for TestStruct {}

    fn create_test_caster() -> (TypeId, BoxedCaster) {
        let type_id = TypeId::of::<TestStruct>();
        let caster = Box::new(Caster::<dyn Debug> {
            cast_ref: |from| from.downcast_ref::<TestStruct>().unwrap(),
            cast_mut: |from| from.downcast_mut::<TestStruct>().unwrap(),
            cast_box: |from| from.downcast::<TestStruct>().unwrap(),
        });
        (type_id, caster)
    }

    #[test]
    fn cast_ref() {
        let ts = TestStruct;
        let st: &dyn SourceTrait = &ts;
        let debug = st.cast::<dyn Debug>();
        assert!(debug.is_some());
    }

    #[test]
    fn cast_mut() {
        let mut ts = TestStruct;
        let st: &mut dyn SourceTrait = &mut ts;
        let debug = st.cast::<dyn Debug>();
        assert!(debug.is_some());
    }

    #[test]
    fn cast_box() {
        let ts = Box::new(TestStruct);
        let st: Box<dyn SourceTrait> = ts;
        let debug = st.cast::<dyn Debug>();
        assert!(debug.is_ok());
    }

    #[test]
    fn cast_ref_wrong() {
        let ts = TestStruct;
        let st: &dyn SourceTrait = &ts;
        let display = st.cast::<dyn Display>();
        assert!(display.is_none());
    }

    #[test]
    fn cast_mut_wrong() {
        let mut ts = TestStruct;
        let st: &mut dyn SourceTrait = &mut ts;
        let display = st.cast::<dyn Display>();
        assert!(display.is_none());
    }

    #[test]
    fn cast_box_wrong() {
        let ts = Box::new(TestStruct);
        let st: Box<dyn SourceTrait> = ts;
        let display = st.cast::<dyn Display>();
        assert!(display.is_err());
    }

    #[test]
    fn cast_ref_from_any() {
        let ts = TestStruct;
        let st: &dyn Any = &ts;
        let debug = st.cast::<dyn Debug>();
        assert!(debug.is_some());
    }

    #[test]
    fn cast_mut_from_any() {
        let mut ts = TestStruct;
        let st: &mut dyn Any = &mut ts;
        let debug = st.cast::<dyn Debug>();
        assert!(debug.is_some());
    }

    #[test]
    fn cast_box_from_any() {
        let ts = Box::new(TestStruct);
        let st: Box<dyn Any> = ts;
        let debug = st.cast::<dyn Debug>();
        assert!(debug.is_ok());
    }

    #[test]
    fn impls_ref() {
        let ts = TestStruct;
        let st: &dyn SourceTrait = &ts;
        assert!(st.impls::<dyn Debug>());
    }

    #[test]
    fn impls_mut() {
        let mut ts = TestStruct;
        let st: &mut dyn SourceTrait = &mut ts;
        assert!((*st).impls::<dyn Debug>());
    }

    #[test]
    fn impls_box() {
        let ts = Box::new(TestStruct);
        let st: Box<dyn SourceTrait> = ts;
        assert!((*st).impls::<dyn Debug>());
    }

    #[test]
    fn impls_not_ref() {
        let ts = TestStruct;
        let st: &dyn SourceTrait = &ts;
        assert!(!st.impls::<dyn Display>());
    }

    #[test]
    fn impls_not_mut() {
        let mut ts = TestStruct;
        let st: &mut dyn Any = &mut ts;
        assert!(!(*st).impls::<dyn Display>());
    }

    #[test]
    fn impls_not_box() {
        let ts = Box::new(TestStruct);
        let st: Box<dyn SourceTrait> = ts;
        assert!(!st.impls::<dyn Display>());
    }
}
