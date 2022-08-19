//! Simple allocator for embedded systems
//!
//! This crate provides a single type called [`Allocator`]. This type implements
//! the [`core::alloc::GlobalAlloc`]-trait, which is required to use the
//! [`alloc`-crate][alloc] on `#![no_std]`-targets. The allocator provided in
//! this crate is relatively simple, but reliable: its design is simple, so that
//! errors in the implementation are unlikely. Furthermore the crate is tested
//! by (unit) tests running under `miri`, so there shouldn't be any undefined
//! behavior.
//!
//! # Usage
//! The usage is simple: just copy and paste the following code snipped into
//! your binary crate and potentially adjust the number of bytes of the heap
//! (here 4K):
//! ```no_run
//! #[global_allocator]
//! static ALLOCATOR: emballoc::Allocator<4096> = emballoc::Allocator::new();
//!
//! extern crate alloc;
//! ```
//! Afterwards you don't need to interact with the crate or the variable
//! `ALLOCATOR` anymore. Now you can just `use alloc::vec::Vec` or even
//! `use alloc::collections::HashMap`, i.e. every fancy collection which is
//! normally provided by the `std`.
//!
//! The minimal buffer size is `8`, which would allow exactly one allocation of
//! size up to 4 at a time. Adjust the size as necessary, e.g. by doing a worst
//! case calculation and potentially adding some backup space of 10% (for
//! example).
//!
//! Note to users with things like `MPU`s, `MMU`s, etc.: your device might
//! support things like memory remapping or memory protection with setting
//! read/write/execution rights. This crate _doesn't use_ those features at all!
//! If that is desired, you should take the address of the buffer and use that
//! along with the known size `N` to protect the heap memory. To users with a
//! fully-working MMU: it is recommended, that you use an allocator, that
//! actually supports paging, etc. This crate might still be helpful, e.g.
//! before setting up the MMU.
//!
//! # Implementation
//! This algorithm does a linear scan for free blocks. The basic algorithm is as
//! follows:
//! 1.  We start with an empty buffer.
//!     ```text
//!     xxxx 0000 0000 0000 0000 0000 0000 0000
//!     ^--- ^---------------------------------
//!     FREE size = 28
//!     ```
//!     There is a single entry, which spans all the remaining buffer bytes
//!     (after the entry itself, which is always 4 bytes).
//! 2.  A block of 8 is allocated.
//!     ```text
//!     xxxx 0000 0000 yyyy 0000 0000 0000 0000
//!     ^--- ^-------- ^--- ^------------------
//!     USED size = 8  FREE size = 16
//!     ```
//!     Now the only free block (the FREE block of step 1) is split into two.
//!     There is now a used block with a total size of 12 bytes, 4 bytes for the
//!     header and 8 bytes for the content. The remaining buffer space is
//!     occupied by the FREE-element. Note, that the total number of "usable"
//!     space (the memory without the headers) shrunk from 28 to 24 (16 + 8)
//!     bytes, since there is now an additional header.
//! 3.  Another block of 4 is allocated.
//!     ```text
//!     xxxx 0000 0000 yyyy 0000 zzzz 0000 0000
//!     ^--- ^-------- ^--- ^--- ^--- ^--------
//!     USED size = 8  USED size FREE size = 8
//!     ```
//!     The same thing as in step 2 happens. Now there are two used blocks and
//!     a single free block with a size of 8.
//! 4.  A request for a block of 16 comes in. There is not enough free memory
//!     for that request. Therefore the allocation fails.
//! 5.  A block of 5 is allocated.
//!     ```text
//!     xxxx 0000 0000 yyyy 0000 zzzz 0000 0000
//!     ^--- ^-------- ^--- ^--- ^--- ^-----!!!
//!     USED size = 8  USED size USED size = 8
//!     ```
//!     There is not enough space at the end of the memory buffer, therefore the
//!     current entry is enlarged to fill the remaining space. This "wastes" 3
//!     bytes, but those would not be usable anyway.
//!
//!     To prevent alignment issues, the blocks are always rounded up to a
//!     multiple of 4 as well, which has the same result (this implies, that the
//!     aforementioned special handling of the remaining bytes is not necessary,
//!     care has to be taken to handle 0-sized "free" blocks correctly).
//! 6.  A request for a block of 1 comes in. There is no free memory at all and
//!     hence not enough free memory for that request. Therefore the allocation
//!     fails.
//! 7.  The third allocation (block size 5) is freed.
//!     ```text
//!     xxxx 0000 0000 yyyy 0000 zzzz 0000 0000
//!     ^--- ^-------- ^--- ^--- ^--- ^--------
//!     USED size = 8  USED size FREE size = 8
//!     ```
//!     The picture of step 3 is restored.
//! 8.  The first allocation (block size 8) is freed.
//!     ```text
//!     xxxx 0000 0000 yyyy 0000 zzzz 0000 0000
//!     ^--- ^-------- ^--- ^--- ^--- ^--------
//!     FREE size = 8  USED size FREE size = 8
//!     ```
//!     Now there are two free blocks and a usable block. Note, that there is
//!     fragmentation, so a request for 12 bytes could not be fulfilled, since
//!     there is no contiguous memory of that size.
//! 9.  Another block of 8 is allocated.
//!     ```text
//!     xxxx 0000 0000 yyyy 0000 zzzz 0000 0000
//!     ^--- ^-------- ^--- ^--- ^--- ^--------
//!     USED size = 8  USED size FREE size = 8
//!     ```
//!     Nothing special here, except that the allocator could choose between the
//!     two blocks of 8. Here the first one was chosen (arbitrarily).
//! 10. The second allocation (block size 4) is freed.
//!     ```text
//!     xxxx 0000 0000 yyyy 0000 0000 0000 0000
//!     ^--- ^-------- ^--- ^------------------
//!     USED size = 8  FREE size = 16
//!     ```
//!     The block is simply replaced by a FREE block, but there is a caveat: the
//!     two adjacent blocks have to be connected to a single big FREE-block in
//!     order to prevent more fragmentation. They are one continuous block with
//!     a single header.
//!
//!     This connection is easy, since the middle block of step 9 just has to
//!     look for the next header (the position of that block is known by its
//!     size) and check, whether it is free. If so, the new block gets adjusted
//!     to have a size of `self.size + 4 + other.size`. This effectively erases
//!     the right free block.
//! 11. A new block of 8 is allocated. Afterwards the first block is freed.
//!     ```text
//!     xxxx 0000 0000 yyyy 0000 0000 0000 0000
//!     ^--- ^-------- ^--- ^-------- ^--- ^---
//!     FREE size = 8  USED size = 8  FREE size
//!     ```
//!     This is just an intermediate step without any issues.
//! 12. The remaining used block is freed.
//!     ```text
//!     xxxx 0000 0000 yyyy 0000 0000 0000 0000
//!     ^--- ^-------- ^--- ^------------------
//!     FREE size = 8  FREE size = 16
//!     ```
//!     Now there are two(!) free blocks, since the concatenation described in
//!     step 10 does only happen to the right side of the freed block. Since the
//!     left block has an unknown size, it is not possible to find the header
//!     (except for linearly scanning the memory from the beginning). Therefore
//!     it is easier to just live with that fragmentation.
//!
//!     Something interesting here is, that one could check for such conditions
//!     from time to time and fix them during that scan. Doing it this way does
//!     not come with a constant time penalty when deallocating. Furthermore it
//!     lets the user decide, whether that feature is necessary or not.
//!
//! [alloc]: https://doc.rust-lang.org/alloc/index.html
#![no_std]

use core::alloc::{GlobalAlloc, Layout};

/// The memory allocator for embedded systems.
///
/// This is the core type of this crate: it is an allocator with a predefined
/// heap size. Therefore the heap memory usage is statically limited to an upper
/// value, which also helps to prevent issues with heap/stack-smashes, as the
/// heap is counted to the static memory (e.g. `.data`/`.bss`-sections). Such a
/// smash might still happen though, if the stack pointer grows into the heap,
/// but the heap cannot grow into the stack pointer.
///
/// Its usage is simple: just copy and paste the following in the binary crate
/// you're developing. The memory size of the heap is `4096` or 4K in this
/// example. Adjust that value to your needs.
/// ```no_run
/// #[global_allocator]
/// static ALLOCATOR: emballoc::Allocator<4096> = emballoc::Allocator::new();
/// ```
/// Also please refer to the [crate-level](crate)-documentation for
/// recommendations on the buffer size and general usage.
pub struct Allocator<const N: usize>(());
impl<const N: usize> Allocator<N> {
    /// Create a new [`Allocator`].
    ///
    /// This function is a `const fn`, therefore you can call it directly when
    /// creating the allocator.
    ///
    /// Please see the [crate-level](crate)-documentation for recommendations on
    /// the buffer size and general usage.
    ///
    /// # Panics
    /// This function will panic, if the supplied buffer size, i.e. `N` is less
    /// than `8` or not divisible by `4`.
    #[must_use = "assign the allocator to a static variable and apply the `#[global_allocator]`-attribute to make it the global allocator"]
    pub const fn new() -> Self {
        assert!(N >= 8, "too small heap memory: minimum size is 8");
        assert!(N % 4 == 0, "memory size has to be divisible by 4");
        Self(())
    }
}
unsafe impl<const N: usize> GlobalAlloc for Allocator<N> {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        todo!()
    }
}
