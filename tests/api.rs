#[test]
fn is_usable_in_const_contexts() {
    const _ALLOCATOR1: emballoc::Allocator<32> = emballoc::Allocator::new();
    static _ALLOCATOR2: emballoc::Allocator<32> = emballoc::Allocator::new();
}

#[test]
fn supports_global_alloc() {
    fn assert<T: core::alloc::GlobalAlloc>(_: T) {}
    assert(emballoc::Allocator::<64>::new())
}

#[test]
#[should_panic(expected = "too small heap memory")]
fn min_heap_size_of_at_least_8() {
    let _allocator = emballoc::Allocator::<4>::new(); // panic here
}

#[test]
#[should_panic(expected = "divisible by 4")]
fn heap_size_must_be_a_multiple_of_4() {
    let _allocator = emballoc::Allocator::<31>::new(); // panic here
}
