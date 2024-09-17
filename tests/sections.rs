//! This test ensures, that the allocator heap is not placed in `.data`.
//!
//! The `.data`-section typically contains the non-zero-initialized global
//! variables, so your `static X: u32 = 42` will show up there. Crucially, this
//! is also the home of partly-initialized memory (i.e. if not all bytes are
//! zeroed). This, however, needs not just the actually used RAM but also flash
//! (on the most micro-controllers and embedded devices): the initialization
//! data for the variables in `.data` (hence the name). So: every variable in
//! `.data` also shows up in the non-volatile flash. This is fine and expected.
//!
//! The aforementioned behavior is bad for the allocator: if the allocator is
//! located in the `.data`-section, the whole initial heap is also stored in the
//! non-volatile flash, despite the fact, that _all but the first four bytes are
//! uninitialized_!
//!
//! Therefore this test makes sure, that an global allocator is not placed in
//! the `.data`-section. This ensures, that issues like [#30] won't pop up
//! again.
//!
//! [#30]: https://github.com/jfrimmel/emballoc/issues/30

use std::alloc::{GlobalAlloc, Layout};
use std::ptr;

static ALLOCATOR: emballoc::Allocator<{ 128 * 1024 * 1024 }> = emballoc::Allocator::new();

#[cfg(all(target_arch = "x86_64", target_os = "linux"))] // this is only tested on Linux
#[test]
fn ensure_that_allocator_memory_is_not_initialized() {
    // Just use the allocator in order to make sure that it will actually remain
    // in the binary.
    // SAFETY: we just use the allocator as intended.
    unsafe {
        let layout = Layout::new::<u64>();
        let ptr = ALLOCATOR.alloc(layout);
        ALLOCATOR.dealloc(ptr, layout);
    }

    let memory_map = MemoryMap::new();
    let bss_start = memory_map.bss_start;
    let data_end = memory_map.data_end;
    assert_eq!(bss_start, data_end, "test assumes bss directly after data");

    let addr_allocator = ptr::addr_of!(ALLOCATOR) as usize;
    assert!(addr_allocator >= bss_start, "allocator is placed in .data");
}

/// The (at runtime) reconstructed memory map containing addresses of sections.
struct MemoryMap {
    /// The end of the `.data`-section.
    data_end: usize,
    /// The start address of the `.bss`-section.
    bss_start: usize,
}
impl MemoryMap {
    pub fn new() -> Self {
        // The symbols defined in the (default) linker script
        extern "C" {
            static __bss_start: usize;
            static _edata: usize;
        }

        Self {
            data_end: unsafe { ptr::addr_of!(__bss_start) } as usize,
            bss_start: unsafe { ptr::addr_of!(_edata) } as usize,
        }
    }
}
