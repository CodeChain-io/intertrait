# InterTrait
This library provides direct casting among trait objects implemented by a type.

In Rust, an object of a sub-trait of `std::any::Any` can be downcast to a concrete value at runtime if the type of the value is known. But no casting between two trait objects are possible (even no coercion from a trait object to that of its super-trait yet).

# How it works
`intertrait` crate generates trampoline functions for downcasting a trait object of `std::any::Any` back to its concrete value and then creating another trait object for the target trait, and let them leveraged with convenience. In the course, it doesn't rely on any unstable behavior such as the layout of trait objects that may be changed in the future.


