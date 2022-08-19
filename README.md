# Embedded memory allocator

This repository provides the [`emballoc`](https://crates.io/crates/emballoc) crate: a simple memory allocator developed for usage in small embedded systems.
It is one possible way to support dynamic memory on targets without the standard library, i.e. ones with `#![no_std]`.
This is achieved by providing a type [`Allocation`](https://docs.rs/emballoc/*/emballoc/struct.Allocator.html) which can be registered as the global allocator for the binary.
See the usage description below.

An allocator is a rather critical part of a software project:
when using dynamic memory many operations implicitly can or will allocate, sometimes unexpectedly.
Therefore a misbehaving allocator can "randomly" crash the program in ver obscure ways.
As such an allocator has to be well-tested and battle-proven.
Furthermore it has to be _simple_: the simpler the algorithm is, the more likely is a correct implementation.

Refer to the [crate-documentation](https://docs.rs/emballoc/) for details on the algorithm and usage hints.

# Usage

Copy the following snippet to your `Cargo.toml` to pull the crate in as one of your dependencies.

```toml
[dependencies.emballoc]
version = "*" # replace with current version from crates.io
```

After that the usage is very simple: just copy the following code to the binary crate of the project.
Substitute the `4096` with the desired heap size.

```rust
#[global_allocator]
static ALLOCATOR: emballoc::Allocator<4096> = emballoc::Allocator::new();

extern crate alloc;
```

Now the crate can use the `std` collections such as `Vec<T>`, `HashMap<K, V>`, etc. together with important types like `Box<T>` and `Rc<T>`.
Note, that things in the `std`-prelude (e.g. `Vec<T>`, `Box<T>`, ...) have to be imported explicitly.
