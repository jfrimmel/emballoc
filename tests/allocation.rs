#![no_std]

const HEAP_SIZE: usize = 4 * 1024 * 1024;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<HEAP_SIZE> = emballoc::Allocator::new();

extern crate alloc;

#[test]
fn vec() {
    let mut v = alloc::vec![1, 2, 3];
    v.push(4);

    assert_eq!((1..=4).collect::<alloc::vec::Vec<_>>(), v);
}

#[test]
fn map_and_formatting() {
    let mut map = alloc::collections::BTreeMap::new();
    map.insert(10, "Hello");
    map.insert(11, "world");
    map.insert(20, "Hallo");
    map.insert(21, "Welt");
    map.insert(-1, "english");
    map.insert(-2, "german");

    let english = alloc::format!("[{}]: {}, {}!", map[&-1], map[&10], map[&11]);
    let german = alloc::format!("[{}]: {}, {}!", map[&-2], map[&20], map[&21]);
    assert_eq!(english, "[english]: Hello, world!");
    assert_eq!(german, "[german]: Hallo, Welt!");
}
