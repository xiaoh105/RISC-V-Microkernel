use core::alloc::Layout;
use core::ptr::addr_of_mut;
use crate::utils::allocator::LockedAllocator;
use crate::config::KERNEL_HEAP_SIZE;

extern crate alloc;

#[global_allocator]
static HEAP_ALLOCATOR: LockedAllocator<32> = LockedAllocator::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error: layout = {:?}", layout);
}

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub unsafe fn init_heap() {
    unsafe { HEAP_ALLOCATOR.init(addr_of_mut!(HEAP_SPACE) as usize, KERNEL_HEAP_SIZE); }
}

pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    unsafe extern "C" {
        fn sbss();
        fn ebss();
    }
    let bss_range = sbss as usize..ebss as usize;
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for i in 0..500 {
        assert_eq!(v[i], i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
}