use std::any::{Any, TypeId};
use std::collections::HashMap;

use linkme::distributed_slice;
use once_cell::sync::Lazy;

use crate::hasher::BuildFastHasher;

mod hasher;

/// A distributed slice gathering constructor functions for [`Caster<T>`]s.
///
/// A constructor function returns `TypeId` of a concrete type involved in the casting
/// and a `Box` of a trait object backed by a [`Caster<T>`].
///
/// [`Caster<T>`]: ./struct.Caster.html
#[distributed_slice]
pub static CASTERS: [fn() -> (TypeId, Box<dyn Any + Send + Sync>)] = [..];

/// A `HashMap` mapping `TypeId` of a [`Caster<S, T>`] to an instance of it.
///
/// [`Caster<S, T>`]: ./struct.Caster.html
static CASTER_MAP: Lazy<HashMap<(TypeId, TypeId), Box<dyn Any + Send + Sync>, BuildFastHasher>> =
    Lazy::new(|| {
        CASTERS
            .iter()
            .map(|f| {
                let (type_id, caster) = f();
                ((type_id, (*caster).type_id()), caster)
            })
            .collect()
    });

/// A `Caster` knows how to cast a reference to or `Box` of a trait object of type `Any`
/// to a trait object of type `T`. Each `Caster` instance is specific to a concrete type.
/// That is, it knows how to cast to single specific trait implemented by single specific type.
///
/// An implementation of a trait for a concrete type doesn't need to manually provide
/// a `Caster`. Instead attach `#[cast_to]` to the `impl` block.
#[doc(hidden)]
pub struct Caster<T: ?Sized + 'static> {
    /// Casts a reference to a trait object of type `Any` from a concrete type `S`
    /// to a reference to a trait object of type `T`.
    cast_ref: fn(from: &dyn Any) -> Option<&T>,

    /// Casts a mutable reference to a trait object of type `Any` from a concrete type `S`
    /// to a mutable reference to a trait object of type `T`.
    cast_mut: fn(from: &mut dyn Any) -> Option<&mut T>,

    /// Casts a `Box` holding a trait object of type `Any` from a concrete type `S`
    /// to another `Box` holding a trait object of type `T`.
    cast_box: fn(from: Box<dyn Any>) -> Result<Box<T>, Box<dyn Any>>,
}

/// Returns a `Caster<S, T>` from a concrete type `S` to a trait `T` implemented by it.
fn caster<T: ?Sized + 'static>(type_id: TypeId) -> Option<&'static Caster<T>> {
    CASTER_MAP
        .get(&(type_id, TypeId::of::<Caster<T>>()))
        .and_then(|caster| caster.downcast_ref::<Caster<T>>())
}

/// `CastFrom` is extended by a trait if the trait wants to allow for casting
/// into another trait.
pub trait CastFrom {
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

/// A trait that is blanket-implemented for traits extending `Any` to allow for casting
/// to another trait.
pub trait CastTo {
    /// Casts a reference to this trait into that of type `T`.
    fn ref_to<T: ?Sized + 'static>(&self) -> Option<&T>;

    /// Casts a mutable reference to this trait into that of type `T`.
    fn mut_to<T: ?Sized + 'static>(&mut self) -> Option<&mut T>;

    /// Casts a box to this trait into that of type `T`.
    fn box_to<T: ?Sized + 'static>(self: Box<Self>) -> Option<Box<T>>;

    /// Tests if this trait object can be cast into `T`.
    fn ok<T: ?Sized + 'static>(&self) -> bool;
}

/// A blanket implementation of `CastTo` for traits extending `CastFrom`.
impl<S: ?Sized + CastFrom> CastTo for S {
    fn ref_to<T: ?Sized + 'static>(&self) -> Option<&T> {
        let any = self.ref_any();
        let caster = caster::<T>(any.type_id())?;
        (caster.cast_ref)(any)
    }

    fn mut_to<T: ?Sized + 'static>(&mut self) -> Option<&mut T> {
        let any = self.mut_any();
        let caster = caster::<T>((*any).type_id())?;
        (caster.cast_mut)(any)
    }

    fn box_to<T: ?Sized + 'static>(self: Box<Self>) -> Option<Box<T>> {
        let any = self.box_any();
        let caster = caster::<T>((*any).type_id())?;
        (caster.cast_box)(any).map(|b| b as Box<T>).ok()
    }

    fn ok<T: ?Sized + 'static>(&self) -> bool {
        CASTER_MAP.contains_key(&(self.ref_any().type_id(), TypeId::of::<Caster<T>>()))
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::fmt::{Debug, Display};

    use linkme::distributed_slice;

    use crate::CastFrom;

    use super::CastTo;
    use super::Caster;

    #[distributed_slice(super::CASTERS)]
    static TEST_CASTER: fn() -> (TypeId, Box<dyn Any + Send + Sync>) = create_test_caster;

    #[derive(Debug)]
    struct TestStruct;

    trait SourceTrait: CastFrom {}

    impl SourceTrait for TestStruct {}

    fn create_test_caster() -> (TypeId, Box<dyn Any + Send + Sync>) {
        let type_id = TypeId::of::<TestStruct>();
        let caster = Box::new(Caster::<dyn Debug> {
            cast_ref: |from| from.downcast_ref::<TestStruct>().map(|c| c as &dyn Debug),
            cast_mut: |from| {
                from.downcast_mut::<TestStruct>()
                    .map(|c| c as &mut dyn Debug)
            },
            cast_box: |from| from.downcast::<TestStruct>().map(|c| c as Box<dyn Debug>),
        });
        (type_id, caster)
    }

    #[test]
    fn ref_to() {
        let ts = TestStruct;
        let st: &dyn SourceTrait = &ts;
        let debug: Option<&dyn Debug> = st.ref_to();
        assert!(debug.is_some());
    }

    #[test]
    fn mut_to() {
        let mut ts = TestStruct;
        let st: &mut dyn SourceTrait = &mut ts;
        let debug: Option<&mut dyn Debug> = st.mut_to();
        assert!(debug.is_some());
    }

    #[test]
    fn box_to() {
        let ts = Box::new(TestStruct);
        let st: Box<dyn SourceTrait> = ts;
        let debug: Option<Box<dyn Debug>> = st.box_to();
        assert!(debug.is_some());
    }

    #[test]
    fn ref_to_wrong() {
        let ts = TestStruct;
        let st: &dyn SourceTrait = &ts;
        let debug: Option<&dyn Display> = st.ref_to();
        assert!(debug.is_none());
    }

    #[test]
    fn mut_to_wrong() {
        let mut ts = TestStruct;
        let st: &mut dyn SourceTrait = &mut ts;
        let debug: Option<&mut dyn Display> = st.mut_to();
        assert!(debug.is_none());
    }

    #[test]
    fn box_to_wrong() {
        let ts = Box::new(TestStruct);
        let st: Box<dyn SourceTrait> = ts;
        let debug: Option<Box<dyn Display>> = st.box_to();
        assert!(debug.is_none());
    }

    #[test]
    fn ref_to_from_any() {
        let ts = TestStruct;
        let st: &dyn Any = &ts;
        let debug: Option<&dyn Debug> = st.ref_to();
        assert!(debug.is_some());
    }

    #[test]
    fn mut_to_from_any() {
        let mut ts = TestStruct;
        let st: &mut dyn Any = &mut ts;
        let debug: Option<&mut dyn Debug> = st.mut_to();
        assert!(debug.is_some());
    }

    #[test]
    fn box_to_from_any() {
        let ts = Box::new(TestStruct);
        let st: Box<dyn Any> = ts;
        let debug: Option<Box<dyn Debug>> = st.box_to();
        assert!(debug.is_some());
    }

    #[test]
    fn ok_ref() {
        let ts = TestStruct;
        let st: &dyn SourceTrait = &ts;
        assert!(st.ok::<dyn Debug>());
    }

    #[test]
    fn ok_mut() {
        let mut ts = TestStruct;
        let st: &mut dyn SourceTrait = &mut ts;
        assert!((*st).ok::<dyn Debug>());
    }

    #[test]
    fn ok_box() {
        let ts = Box::new(TestStruct);
        let st: Box<dyn SourceTrait> = ts;
        assert!((*st).ok::<dyn Debug>());
    }

    #[test]
    fn not_ok_ref() {
        let ts = TestStruct;
        let st: &dyn SourceTrait = &ts;
        assert!(!st.ok::<dyn Display>());
    }

    #[test]
    fn not_ok_mut() {
        let mut ts = TestStruct;
        let st: &mut dyn Any = &mut ts;
        assert!(!(*st).ok::<dyn Display>());
    }

    #[test]
    fn not_ok_box() {
        let ts = Box::new(TestStruct);
        let st: Box<dyn SourceTrait> = ts;
        assert!(!st.ok::<dyn Display>());
    }
}
