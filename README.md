# Intertrait

![Build Status](https://github.com/CodeChain-io/intertrait/workflows/ci/badge.svg)
[![Latest Version](https://img.shields.io/crates/v/intertrait.svg)](https://crates.io/crates/intertrait)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/intertrait)

This library provides direct casting among trait objects implemented by a type.

In Rust, a trait object for a sub-trait of [`std::any::Any`] can be downcast to a concrete type at runtime
if the type is known. But no direct casting between two trait objects (i.e. without involving the concrete type
of the backing value) are possible (even no coercion from a trait object for a trait to that for its super-trait yet).

With this crate, any trait object for a sub-trait of [`CastFrom`] can be cast directly to a trait object
for another trait implemented by the underlying type if the target traits are registered beforehand
with the macros provided by this crate.

# Usage
```rust
use intertrait::*;
use intertrait::cast::*;

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
    let greet = source.cast::<dyn Greet>();
    greet.unwrap().greet();
}
```

Target traits must be explicitly designated beforehand. There are three ways of doing it:

### `#[cast_to]` to `impl` item
The trait implemented is designated as a target trait.

```rust
use intertrait::*;

struct Data;
trait Greet { fn greet(&self); }

#[cast_to]
impl Greet for Data {
    fn greet(&self) {
        println!("Hello");
    }
}
```

### `#[cast_to(Trait)]` to type definition
For the type, the traits specified as arguments to the `#[cast_to(...)]` attribute are designated as target traits.

```rust
use intertrait::*;

trait Greet { fn greet(&self); }

impl Greet for Data {
    fn greet(&self) {
        println!("Hello");
    }
}

#[cast_to(Greet, std::fmt::Debug)]
#[derive(std::fmt::Debug)]
struct Data;
```

### `castable_to!(Type => Trait1, Trait2)`
For the type, the traits following `:` are designated as target traits.

```rust
use intertrait::*;

#[derive(std::fmt::Debug)]
struct Data;
trait Greet { fn greet(&self); }
impl Greet for Data {
    fn greet(&self) {
        println!("Hello");
    }
}
// Only in an item position due to the current limitation in the stable Rust.
// https://github.com/rust-lang/rust/pull/68717
castable_to!(Data => Greet, std::fmt::Debug);

fn main() {}
```

## `Arc` Support
`std::sync::Arc` is unique in that it implement `downcast` method only on `dyn Any + Send + Sync + 'static'.
To use with `Arc`, the following steps should be taken:

* Mark source traits with [`CastFromSync`] instead of [`CastFrom`]
* Add `[sync]` flag to `#[cast_to]` and `castable_to!` as follows:
  ```ignore
  #[cast_to([sync])]
  #[cast_to([sync] Trait1, Trait2)]
  castable_to!(Type => [sync] Trait, Trait2);
  ```

# How it works
First of all, [`CastFrom`] trait makes it possible to retrieve an object of [`std::any::Any`]
from an object for a sub-trait of [`CastFrom`]. 

And the macros provided by `intertrait` generates trampoline functions for downcasting a trait object
for [`std::any::Any`] back to its concrete type and then creating a trait object for the target trait from it.

Those trampoline functions are aggregated into a global registry
using [`linkme`](https://github.com/dtolnay/linkme/) crate, which involves no (generally discouraged)
life-before-main trick. The registry is keyed with a pair of [`TypeId`]s, which are those of the concrete type
backing a trait object for a sub-trait of [`CastFrom`] and the target trait (the actual implementation
is a bit different here, but conceptually so).

In the course, it doesn't rely on any unstable Rust implementation details such as the layout of trait objects
that may be changed in the future.

# Credits
`intertrait` has taken much of its core ideas from the great [`traitcast`](https://github.com/bch29/traitcast) crate.

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
[`CastFromSync`]: https://docs.rs/intertrait/*/intertrait/trait.CastFromSync.html