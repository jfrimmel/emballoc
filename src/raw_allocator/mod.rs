//! This module provides the raw allocator and its support types.
//!
//! A "raw allocator" is one, that simply gets request for a specific memory
//! size but does not need to worry about alignment.
use core::mem::MaybeUninit;

/// An error occurred when calling `free()`.
pub enum FreeError {
    /// There is a double-free detected. An already freed-up-block is freed up
    /// again.
    DoubleFreeDetected,
    /// An invalid pointer was freed up (either a pointer outside of the heap
    /// memory or a pointer to a header).
    AllocationNotFound,
}

/// A raw memory allocator for contiguous slices of bytes without any alignment.
///
/// This allocator is an intermediate one, which does not need to handle the
/// alignment of a [`Layout`](core::alloc::Layout). This abstracts the parts
/// "allocating of memory" and "getting a pointer with proper alignment".
///
/// Note, that the allocated memory is always aligned to `4`.
pub struct RawAllocator<const N: usize>(());
impl<const N: usize> RawAllocator<N> {
    /// Create a new [`RawAllocator`] with a given heap size.
    ///
    /// # Panics
    /// This function panics if the buffer size is less than `8` (the minimum
    /// useful allocation heap) or if it is not divisible by 4.
    pub const fn new() -> Self {
        assert!(N >= 8, "too small heap memory: minimum size is 8");
        assert!(N % 4 == 0, "memory size has to be divisible by 4");

        Self(())
    }

    /// Allocate a new memory block of size `n`.
    ///
    /// This method is used for general allocation of multiple contiguous bytes.
    /// It searches for the smallest possible free entry and mark it as "used".
    /// As usual with [`RawAllocator`], this does not take alignment in account.
    ///
    /// If the allocation fails, `None` will be returned.
    pub fn alloc(&mut self, _n: usize) -> Option<&mut [MaybeUninit<u8>]> {
        todo!()
    }

    /// Free a pointer inside a used memory block.
    ///
    /// This method is used to release a memory block allocated with this raw
    /// allocator. If a entry to the given pointer is found, the corresponding
    /// memory block is marked as free. If no entry is found, than an error is
    /// reported (as allocators are not allowed to unwind).
    ///
    /// # Algorithm
    /// Freeing a pointer is done in the following way: all the entries are
    /// scanned linearly. The pointer is compared against each block. If the
    /// pointer points to the memory of an entry, than that entry is selected.
    /// If no such entry is found, than the user tried to free an allocation,
    /// that was not allocated with this allocator (or the allocator messed up
    /// internally). [`FreeError::AllocationNotFound`] is reported.
    ///
    /// The selected block is tested for its state. If it is marked as "used",
    /// than everything is fine. If it is already marked as "free", than
    /// [`FreeError::DoubleFreeDetected`] is returned. If the block following
    /// the just freed up one is also free, the two blocks are concatenated to a
    /// single one (to prevent fragmentation).
    pub fn free(&mut self, _ptr: *mut u8) -> Result<(), FreeError> {
        todo!()
    }
}
