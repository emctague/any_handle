# any_handle

[![Crates.io](https://img.shields.io/crates/v/any_handle?style=for-the-badge)](https://crates.io/crates/any_handle) [![docs.rs](https://img.shields.io/docsrs/any_handle?style=for-the-badge)](https://docs.rs/any_handle) [![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/emctague/any_handle/rust.yml?style=for-the-badge)](https://github.com/emctague/any_handle) [![Crates.io](https://img.shields.io/crates/l/any_handle?style=for-the-badge)](https://opensource.org/license/mit/) 

`any_handle` provides a reference-counting smart pointer type, `AnyHandle<T>`,
which can store a value of a type `T`. The special `AnyHandle<dyn Any>` allows for
downcasting to any other `AnyHandle<T: Any>` type.

Internally, an `AnyHandle` is an `Arc<RwLock<Box<dyn Any>>>`, and matches the
reference-counting behaviour of `Arc` as well as the many-readers-or-one-writer
thread safety model of `RwLock`.

```rust
use any_handle::{AnyHandle, Any};
struct SomeStruct (i32);

fn main() {
    // Initialize a handle with an unknown type.
    // If you want to pass in a Box<dyn SomeOtherTrait>, instead of a concrete
    // type, you will have to use `#![feature(trait_upcasting)]`, unfortunately.
    let handle: AnyHandle<dyn Any> = AnyHandle::new(Box::new(SomeStruct(12)));
    // Now we can put it in some sort of generic container...
    
    // ...and when we retrieve it later:
    let mut handle: AnyHandle<SomeStruct> = handle.into()?;
    handle.write().do_mut_things_with();
    handle.read().do_things_with();
}
```
