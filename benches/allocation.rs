#![feature(test)]
extern crate test;
use test::Bencher;

use std::alloc::{GlobalAlloc as _, Layout};

mod repeated_allocation_deallocation {
    use super::*;

    /// Run a benchmark, which repeatedly allocates and deallocates the same
    /// block. The benchmark will allocate the given amount of blocks beforehand
    /// (before running the actual benchmark) in order to fill up the heap with
    /// unrelated allocations.
    ///
    /// # Panics
    /// This will panic, if the requested pre-allocations will fill up the whole
    /// heap (so the actual benchmark cannot allocate blocks anymore).
    fn benchmark_with_preallocation(b: &mut Bencher, pre_allocations: usize) {
        let allocator = emballoc::Allocator::<8192>::new();
        // pre-allocate much memory to see the real impact of the linear search
        for _ in 0..pre_allocations {
            unsafe { allocator.alloc(Layout::new::<u8>()) };
        }

        let layout = Layout::new::<u8>();

        // make sure, that there is enough room for the next allocation
        let ptr = unsafe { allocator.alloc(layout) };
        assert_ne!(ptr, std::ptr::null_mut::<u8>());
        unsafe { allocator.dealloc(ptr, layout) };

        // run actual benchmark: allocate & deallocate the same block repeatedly
        b.iter(|| {
            let ptr = unsafe { allocator.alloc(layout) };
            let ptr = test::black_box(ptr);
            unsafe { allocator.dealloc(ptr, layout) };
        });
    }

    #[bench]
    fn no_memory_usage(b: &mut Bencher) {
        benchmark_with_preallocation(b, 0);
    }

    #[bench]
    fn low_memory_usage(b: &mut Bencher) {
        benchmark_with_preallocation(b, 8);
    }

    #[bench]
    fn medium_memory_usage(b: &mut Bencher) {
        benchmark_with_preallocation(b, 510);
    }

    #[bench]
    fn high_memory_usage(b: &mut Bencher) {
        benchmark_with_preallocation(b, 1020);
    }
}
