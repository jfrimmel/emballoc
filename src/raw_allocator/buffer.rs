use super::entry::Entry;

use core::mem::{self, MaybeUninit};

/// An offset into the [`Buffer`], that is validated and known to be safe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedOffset(usize);

/// The buffer memory backing the heap.
#[repr(align(4))]
pub struct Buffer<const N: usize>([MaybeUninit<u8>; N]);
impl<const N: usize> Buffer<N> {
    /// Create a new buffer.
    ///
    /// This buffer will be uninitialized except for the first few bytes, which
    /// contain the first header. This header is a free [`Entry`] with the size
    /// of the remaining buffer.
    ///
    /// # Panics
    /// This function panics if the buffer is less than 4 bytes in size, i.e. if
    /// `N < 4`.
    pub const fn new() -> Self {
        assert!(N >= 4, "buffer too small, use N >= 4");
        let remaining_size = N - mem::size_of::<Entry>();
        let initial_entry = Entry::free(remaining_size).as_raw();

        // this is necessary, since there mut be always a valid first entry
        let mut buffer = [MaybeUninit::uninit(); N];
        buffer[0] = MaybeUninit::new(initial_entry[0]);
        buffer[1] = MaybeUninit::new(initial_entry[1]);
        buffer[2] = MaybeUninit::new(initial_entry[2]);
        buffer[3] = MaybeUninit::new(initial_entry[3]);
        Self(buffer)
    }

    /// Obtain a reference to an [`Entry`] inside of the buffer.
    ///
    /// The returned memory will point inside the buffer itself and thus
    /// modifying the reference will modify the buffer contents. This is a safe
    /// operation, since the calling requirements (see below) are checked at
    /// runtime. For safety-reasons this function does not return the [`Entry`]
    /// directly, but instead uses a [`MaybeUninit<Entry>`] instead. Without
    /// this, the function would be unsafe, since the caller would need to
    /// guarantee, that the memory read is actually filled with a valid and
    /// initialized `Entry`. By using the `MaybeUninit`-variant, the caller has
    /// to use the `unsafe`-block when actually reading and assuming, that it is
    /// initialized.
    ///
    /// # Panics
    /// This function panics if the offset is not a multiple of 4 or the offset
    /// plus the 4 bytes after it would read past the end of the buffer.
    fn at(&self, offset: usize) -> &MaybeUninit<Entry> {
        assert!(offset % mem::align_of::<Entry>() == 0);
        assert!(offset + mem::size_of::<Entry>() <= self.0.len());

        // SAFETY: this operation is unsafe for multiple reasons: the alignment
        // has to be satisfied and the entry read must be in bound of the buffer
        // memory.
        // 1. the bounds of the memory is checked by the assert above: the
        //    current offset plus the number of bytes read for an `Entry` is
        //    inside the buffer. Therefore this safety requirement is always
        //    fulfilled.
        // 2. the proper alignment is ensured by first checking, whether the
        //    offset is a multiple of the alignment of `Entry`. This makes sure,
        //    that we are aligned within the buffer. Another important aspect is
        //    that the buffer itself is aligned. This is achieved using a
        //    `#[repr(align(4))]`-attribute on the buffer itself. Therefore the
        //    alignment safety requirement is fulfilled as well.
        //
        // Note, that the memory, that is pointed to, might not contain a valid
        // `Entry`. This is fine, since the function returns a `MaybeUninit`
        // version of an `Entry`. Therefore the caller has to ensure, that the
        // thing written or read is valid.
        unsafe {
            let memory = &self.0[offset..offset + 4];
            let memory = memory.as_ptr();
            #[allow(clippy::cast_ptr_alignment)] // alignment is asserted above
            &*(memory
                .cast::<[MaybeUninit<u8>; 4]>()
                .cast::<MaybeUninit<Entry>>())
        }
    }

    /// Obtain a mutable reference to an [`Entry`] inside of the buffer.
    ///
    /// Please see [`at()`](Self::at) for details.
    ///
    /// # Panics
    /// This function panics if the offset is not a multiple of 4 or the offset
    /// plus the 4 bytes after it would read past the end of the buffer.
    fn at_mut(&mut self, offset: usize) -> &mut MaybeUninit<Entry> {
        assert!(offset % mem::align_of::<Entry>() == 0);
        assert!(offset + mem::size_of::<Entry>() <= self.0.len());

        // SAFETY: same as `at()`
        unsafe {
            let memory = &mut self.0[offset..offset + 4];
            let memory = memory.as_mut_ptr();
            #[allow(clippy::cast_ptr_alignment)] // alignment is asserted above
            &mut *(memory
                .cast::<[MaybeUninit<u8>; 4]>()
                .cast::<MaybeUninit<Entry>>())
        }
    }

    /// Iterate over all entries and obtain the [`ValidatedOffset`]s.
    pub fn entries(&self) -> EntryIter<N> {
        EntryIter::new(self)
    }

    /// Request the memory of an entry at a [`ValidatedOffset`].
    ///
    /// This operation is safe, since the offset is validated. It returns the
    /// slice of the memory of the given entry.
    pub fn memory_of(&self, offset: ValidatedOffset) -> &[MaybeUninit<u8>] {
        let offset = offset.0;
        let entry = unsafe { self.at(offset).assume_init_ref() };
        let size = entry.size();

        let offset = offset + mem::size_of::<Entry>();
        &self.0[offset..offset + size]
    }

    /// Request the mutable memory of an entry at a [`ValidatedOffset`].
    ///
    /// This operation is safe, since the offset is validated. It returns the
    /// slice of the memory of the given entry.
    pub fn memory_of_mut(&mut self, offset: ValidatedOffset) -> &mut [MaybeUninit<u8>] {
        let offset = offset.0;
        let entry = unsafe { self.at(offset).assume_init_ref() };
        let size = entry.size();

        let offset = offset + mem::size_of::<Entry>();
        &mut self.0[offset..offset + size]
    }

    /// Query the following entry, if there is a following entry.
    ///
    /// This function takes a [`ValidatedOffset`] of one entry and tries to
    /// obtain a mutable reference to the entry after it. If there is no entry
    /// after it (because the given one is the last in the buffer) then `None`
    /// is returned.
    pub fn following_entry(&mut self, offset: ValidatedOffset) -> Option<&mut MaybeUninit<Entry>> {
        let offset = offset.0;
        let entry = unsafe { self.at(offset).assume_init_ref() };
        let size = entry.size();

        let offset = offset + size + mem::size_of::<Entry>();
        (offset < N).then(|| self.at_mut(offset))
    }
}
impl<const N: usize> core::ops::Index<ValidatedOffset> for Buffer<N> {
    type Output = Entry;

    fn index(&self, index: ValidatedOffset) -> &Self::Output {
        // SAFETY: the `ValidatedOffset` marks the read valid (safety invariant
        // of that type)
        unsafe { self.at(index.0).assume_init_ref() }
    }
}
impl<const N: usize> core::ops::IndexMut<ValidatedOffset> for Buffer<N> {
    fn index_mut(&mut self, index: ValidatedOffset) -> &mut Self::Output {
        // SAFETY: the `ValidatedOffset` marks the read valid (safety invariant
        // of that type)
        unsafe { self.at_mut(index.0).assume_init_mut() }
    }
}

pub struct EntryIter<'buffer, const N: usize> {
    buffer: &'buffer Buffer<N>,
    offset: usize,
}
impl<'buffer, const N: usize> EntryIter<'buffer, N> {
    /// Create an entry iterator over the given [`Buffer`].
    pub const fn new(buffer: &'buffer Buffer<N>) -> Self {
        Self { buffer, offset: 0 }
    }
}
impl<'buffer, const N: usize> Iterator for EntryIter<'buffer, N> {
    type Item = ValidatedOffset;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + mem::size_of::<Entry>() < N {
            let offset = self.offset;
            // SAFETY: the buffer invariant (valid entries) have to be upheld
            let entry = unsafe { self.buffer.at(offset).assume_init_ref() };
            self.offset += entry.size() + mem::size_of::<Entry>();
            Some(ValidatedOffset(offset))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Buffer, Entry, ValidatedOffset};

    #[test]
    fn empty_allocator() {
        let buffer = Buffer::<32>::new();
        let expected = Entry::free(32 - 4);
        let actual = unsafe { buffer.at(0).assume_init() };
        assert_eq!(expected, actual);
    }

    #[test]
    fn entry_iter() {
        let buffer = Buffer::<32>::new();
        let mut iter = buffer.entries();
        assert_eq!(iter.next(), Some(ValidatedOffset(0)));
        assert_eq!(iter.next(), None);

        let mut buffer = Buffer::<32>::new();
        buffer.at_mut(0).write(Entry::free(4));
        buffer.at_mut(8).write(Entry::used(4));
        buffer.at_mut(16).write(Entry::free(12));
        let mut iter = buffer.entries();
        assert_eq!(iter.next(), Some(ValidatedOffset(0)));
        assert_eq!(iter.next(), Some(ValidatedOffset(8)));
        assert_eq!(iter.next(), Some(ValidatedOffset(16)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn indexing() {
        let mut buffer = Buffer::<32>::new();
        buffer.at_mut(8).write(Entry::used(4));

        assert_eq!(buffer[ValidatedOffset(8)], Entry::used(4));
        buffer[ValidatedOffset(8)] = Entry::free(12);
        assert_eq!(buffer[ValidatedOffset(8)], Entry::free(12));
    }

    #[test]
    fn following_entry() {
        let mut buffer = Buffer::<20>::new();
        buffer.at_mut(0).write(Entry::used(4));
        buffer.at_mut(8).write(Entry::used(8));

        let entry = unsafe {
            buffer
                .following_entry(ValidatedOffset(0))
                .unwrap()
                .assume_init()
        };
        assert_eq!(entry, Entry::used(8));
        assert!(buffer.following_entry(ValidatedOffset(8)).is_none());
    }

    #[test]
    fn memory_of() {
        use core::ptr;

        let mut buffer = Buffer::<20>::new();
        buffer.at_mut(0).write(Entry::used(4));

        let expected = &buffer.0[4..8];
        let actual = buffer.memory_of(ValidatedOffset(0));
        assert_eq!(ptr::addr_of!(expected[0]), ptr::addr_of!(actual[0]));
    }
}
