# InterTrait
This library provides direct casting among trait objects implemented by a type.

In Rust, an object of a sub-trait of [`std::any::Any`] can be downcast to a concrete type at runtime if the type is known. But no direct casting between two trait objects (i.e. without involving the concrete type of the backing value) are possible (even no coercion from a trait object to that of its super-trait yet).

With this crate, any trait object with [`CastFrom`] as its super-trait can be cast directly to another trait object implemented by the underlying value if the target traits are registered beforehand with the macros provided by this crate.

# Usage
```
use intertrait::*;

struct Data;

trait Source: CastFrom {}

trait Greet {
    fn greet(&self);
}

#[cast_to]
impl Greet for Data {
    fn greet(&self) {
        println!("Hello");
    }
}

impl Source for Data {}

fn main() {
    let data = Data;
    let source: &dyn Source = &data;
    let greet = source.ref_to::<dyn Greet>();
    greet.unwrap().greet();
}
```

Target traits must be explicitly designated beforehand. There are three ways to do it:

## `#[cast_to]` to `impl` item
The trait implemented is designated as a target trait.

```
# use intertrait::*;
# struct Data;
# trait Greet { fn greet(&self); }
#[cast_to]
impl Greet for Data {
    fn greet(&self) {
        println!("Hello");
    }
}
```

## `#[cast_to(Trait)]` to type definition
For the type, the traits specified as arguments to the `#[cast_to(...)]` attribute are designated as target traits.

```
# use intertrait::*;
# trait Greet { fn greet(&self); }
# impl Greet for Data {
#     fn greet(&self) {
#         println!("Hello");
#     }
# }
#[cast_to(Greet, std::fmt::Debug)]
#[derive(std::fmt::Debug)]
struct Data;
```

## `castable_to!(Type: Trait1, Trait2)`
For the type, the traits following `:` are designated as target traits.

```
# use intertrait::*;
# #[derive(std::fmt::Debug)]
# struct Data;
# trait Greet { fn greet(&self); }
# impl Greet for Data {
#     fn greet(&self) {
#         println!("Hello");
#     }
# }
// Only in an item position due to the current limitation in the stable Rust.
// https://github.com/rust-lang/rust/pull/68717
castable_to!(Data: Greet, std::fmt::Debug);
# fn main() {}
```

# How it works
First of all, [`CastFrom`] trait makes it possible to retrieve an object of [`std::any::Any`] from an object of a sub-trait of [`CastFrom`]. 

> [`CastFrom`] will become obsolete and be replaced with [`std::any::Any`] once the [unsized coercion](https://doc.rust-lang.org/reference/type-coercions.html#unsized-coercions) from a trait object to an object of its super-trait is implemented in the stable Rust.

And the macros provided by `intertrait` generates trampoline functions for downcasting a trait object of [`std::any::Any`] back to its concrete type and then creating the target trait object from it.

Those trampoline functions are aggregated into a global registry using [`linkme`](https://github.com/dtolnay/linkme/) crate, which involves no (generally discouraged) life-before-main trick. The registry is keyed with a pair of [`TypeId`]s, which are for the concrete type backing an object of a sub-trait of [`CastFrom`] and the target trait (the actual implementation is a bit different here, but conceptually so).

In the course, it doesn't rely on any unstable Rust implementation details such as the layout of trait objects that may be changed in the future.

# Credits
`intertrait` has taken much of its core ideas from the great [`traitcast`](https://github.com/bch29/traitcast) crate. This crate enhances mainly in the ergonomics.

# License
Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[`std::any::Any`]: https://doc.rust-lang.org/std/any/trait.Any.html
[`TypeId`]: https://doc.rust-lang.org/std/any/struct.TypeId.html
[`CastFrom`]: https://docs.rs/intertrait/*/intertrait/trait.CastFrom.html