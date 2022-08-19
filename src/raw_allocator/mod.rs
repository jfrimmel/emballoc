//! This module provides the raw allocator and its support types.
//!
//! A "raw allocator" is one, that simply gets request for a specific memory
//! size but does not need to worry about alignment.
mod buffer;
mod entry;
use entry::{Entry, State};

use core::mem::{self, MaybeUninit};

/// An error occurred when calling `free()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub struct RawAllocator<const N: usize> {
    buffer: buffer::Buffer<N>,
}
impl<const N: usize> RawAllocator<N> {
    /// Create a new [`RawAllocator`] with a given heap size.
    ///
    /// # Panics
    /// This function panics if the buffer size is less than `8` (the minimum
    /// useful allocation heap) or if it is not divisible by 4.
    pub const fn new() -> Self {
        assert!(N >= 8, "too small heap memory: minimum size is 8");
        assert!(N % 4 == 0, "memory size has to be divisible by 4");

        let buffer = buffer::Buffer::new();
        Self { buffer }
    }

    /// Allocate a new memory block of size `n`.
    ///
    /// This method is used for general allocation of multiple contiguous bytes.
    /// It searches for the smallest possible free entry and mark it as "used".
    /// As usual with [`RawAllocator`], this does not take alignment in account.
    ///
    /// If the allocation fails, `None` will be returned.
    pub fn alloc(&mut self, n: usize) -> Option<&mut [MaybeUninit<u8>]> {
        const HEADER_SIZE: usize = mem::size_of::<Entry>();

        // round up `n` to next multiple of `size_of::<Entry>()`
        let n = (n + HEADER_SIZE - 1) / HEADER_SIZE * HEADER_SIZE;

        let (offset, _) = self
            .buffer
            .entries()
            .map(|offset| (offset, self.buffer[offset]))
            .filter(|(_offset, entry)| entry.state() == State::Free)
            .filter(|(_offset, entry)| entry.size() >= n)
            .min_by_key(|(_offset, entry)| entry.size())?;

        // if the found block is large enough, split it into a used and a free
        let entry_size = self.buffer[offset].size();
        self.buffer[offset] = Entry::used(n);
        if let Some(following) = self.buffer.following_entry(offset) {
            following.write(Entry::free(entry_size - n - HEADER_SIZE));
        }
        Some(self.buffer.memory_of_mut(offset))
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
    pub fn free(&mut self, ptr: *mut u8) -> Result<(), FreeError> {
        let offset = self
            .buffer
            .entries()
            .find(|offset| {
                let size = self.buffer[*offset].size();
                let memory = self.buffer.memory_of(*offset);
                let ptr = ptr as *const _;
                let start = memory.as_ptr();
                let end = start.wrapping_add(size);

                start <= ptr && ptr < end
            })
            .ok_or(FreeError::AllocationNotFound)?;

        let entry = self.buffer[offset];
        if entry.state() == State::Free {
            return Err(FreeError::DoubleFreeDetected);
        }
        let additional_memory = self
            .buffer
            .following_entry(offset)
            .map(|entry| unsafe { entry.assume_init_ref() })
            .filter(|entry| entry.state() == State::Free)
            .map_or(0, |entry| entry.size() + mem::size_of::<Entry>());
        Ok(self.buffer[offset] = Entry::free(entry.size() + additional_memory))
    }
}

#[cfg(test)]
mod tests {
    use super::{Entry, FreeError, RawAllocator};

    #[test]
    fn successful_single_allocation() {
        let mut allocator = RawAllocator::<32>::new();
        allocator.alloc(4).unwrap();

        let mut iter = allocator.buffer.entries();
        assert_eq!(allocator.buffer[iter.next().unwrap()], Entry::used(4));
        assert_eq!(allocator.buffer[iter.next().unwrap()], Entry::free(20));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn unsuccessful_single_allocation() {
        let mut allocator = RawAllocator::<32>::new();
        assert!(allocator.alloc(36).is_none());
    }

    #[test]
    fn successful_multiple_allocation() {
        let mut allocator = RawAllocator::<32>::new();
        allocator.alloc(12).unwrap();
        allocator.alloc(12).unwrap();
        // allocator is now full
    }

    #[test]
    fn unsuccessful_multiple_allocation() {
        let mut allocator = RawAllocator::<32>::new();
        allocator.alloc(12).unwrap();
        assert!(allocator.alloc(13).is_none());
    }

    #[test]
    fn simple_free() {
        let mut allocator = RawAllocator::<8>::new();
        let memory = allocator.alloc(4).unwrap();
        let ptr = memory.as_mut_ptr().cast();

        // free the memory without concatenation
        allocator.free(ptr).unwrap();

        let offset = allocator.buffer.entries().next().unwrap();
        assert_eq!(allocator.buffer[offset], Entry::free(4));
    }

    #[test]
    fn double_free() {
        let mut allocator = RawAllocator::<32>::new();
        let memory = allocator.alloc(4).unwrap();
        let ptr = memory.as_mut_ptr().cast();
        allocator.alloc(4).unwrap();

        // free the memory without concatenation
        allocator.free(ptr).unwrap();
        assert_eq!(
            allocator.free(ptr).unwrap_err(),
            FreeError::DoubleFreeDetected
        );
    }

    #[test]
    fn invalid_free() {
        use core::ptr;

        let mut allocator = RawAllocator::<32>::new();
        allocator.alloc(4).unwrap();

        // free the memory without concatenation
        let mut x = 0_u32;
        assert_eq!(
            allocator.free(ptr::addr_of_mut!(x).cast()),
            Err(FreeError::AllocationNotFound)
        );
    }

    #[test]
    fn free_with_defrag() {
        let mut allocator = RawAllocator::<32>::new();
        let memory = allocator.alloc(4).unwrap();
        let ptr = memory.as_mut_ptr().cast();

        // free the memory without concatenation
        allocator.free(ptr).unwrap();

        let offset = allocator.buffer.entries().next().unwrap();
        assert_eq!(allocator.buffer[offset], Entry::free(28));
    }

    #[test]
    fn free_at_end() {
        let mut allocator = RawAllocator::<32>::new();
        allocator.alloc(20).unwrap();
        let memory = allocator.alloc(4).unwrap();
        let ptr = memory.as_mut_ptr().cast();

        // free the memory without concatenation
        allocator.free(ptr).unwrap();

        let offset = allocator.buffer.entries().nth(1).unwrap();
        assert_eq!(allocator.buffer[offset], Entry::free(4));
    }

    #[test]
    fn free_impossible_defrag() {
        let mut allocator = RawAllocator::<16>::new();
        let ptr1 = allocator.alloc(4).unwrap().as_mut_ptr();
        let ptr2 = allocator.alloc(4).unwrap().as_mut_ptr();
        allocator.free(ptr1.cast()).unwrap();

        // now we have a free block, followed by a used block which in turn gets
        // freed up. Therefore there are two contiguous free blocks, but those
        // aren't concatenated, since the old free block is to the left (instead
        // of to the right).
        allocator.free(ptr2.cast()).unwrap();

        // therefore there must be two free blocks
        let mut iter = allocator
            .buffer
            .entries()
            .map(|offset| allocator.buffer[offset]);
        assert_eq!(iter.next(), Some(Entry::free(4)));
        assert_eq!(iter.next(), Some(Entry::free(4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn entries() {
        let mut allocator = RawAllocator::<256>::new();
        allocator.alloc(8).unwrap();
        allocator.alloc(56).unwrap();

        let mut iter = allocator
            .buffer
            .entries()
            .map(|offset| allocator.buffer[offset]);
        assert_eq!(iter.next(), Some(Entry::used(8)));
        assert_eq!(iter.next(), Some(Entry::used(56)));
        assert_eq!(iter.next(), Some(Entry::free(180)));
        assert_eq!(iter.next(), None);
    }
}
