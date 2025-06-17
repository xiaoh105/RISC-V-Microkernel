use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::cmp::{max, min};
use core::mem::size_of;
use core::ptr::NonNull;
use crate::{green_msg, yellow_msg};
use crate::utils::spinlock::SpinLock;

pub struct BuddyAllocator<const ORDER: usize> {
    head: [usize; ORDER],
    user: usize,
    allocated: usize,
    total: usize
}

pub struct LockedAllocator<const ORDER: usize> {
    allocator: UnsafeCell<BuddyAllocator<ORDER>>,
    lock: UnsafeCell<SpinLock>
}

fn align_up_to(val: usize, alignment: usize) -> usize {
    assert!(alignment.is_power_of_two());
    (val + alignment - 1) & (!alignment + 1)
}

fn align_to(val: usize, alignment: usize) -> usize {
    assert!(alignment.is_power_of_two());
    val & (!alignment + 1)
}

fn prev_power_of_two(val: usize) -> usize {
    if val == 0 {
        0
    } else {
        1 << (usize::BITS - 1 - val.leading_zeros())
    }
}

impl<const ORDER: usize> BuddyAllocator<ORDER> {
    pub const fn empty() -> Self {
        BuddyAllocator {
            head: [0; ORDER],
            user: 0,
            allocated: 0,
            total: 0
        }
    }
    
    pub fn report(&self) {
        green_msg!("Buddy Allocator: {}/{} bytes used(with {}/{} bytes allocated).", 
            self.user, self.total, self.allocated, self.total);
    }
    
    unsafe fn insert_block(&mut self, addr: usize, order: usize) {
        if self.head[order] == 0 {
            self.head[order] = addr;
            unsafe {
                *(addr as *mut usize) = 0;
            }
        } else {
            unsafe {
                *(addr as *mut usize) = self.head[order];
                self.head[order] = addr;
            }
        }
    }
    
    unsafe fn pop_front(&mut self, order: usize) -> Option<usize> {
        if self.head[order] == 0 {
            None
        } else {
            let ret = self.head[order];
            unsafe { self.head[order] = *(self.head[order] as *mut usize); }
            Some(ret)
        }
    }
    
    pub unsafe fn init(&mut self, mut start: usize, size: usize) {
        let mut end = start + size;
        start = align_up_to(start, size_of::<usize>());
        end = align_to(end, align_of::<usize>());
        assert!(start <= end);
        if start == end {
            yellow_msg!("Warning: No allocatable memory after alignment");
        }
        let mut current = start;
        while current + size_of::<usize>()  <= end {
            let mut alignment = min(current & (!current + 1), prev_power_of_two(end - current));
            let mut order = alignment.trailing_zeros() as usize;
            if order >= ORDER {
                yellow_msg!("Warning: Your given order {} is lower than the largest possible alignment ({} bytes)",
                    ORDER, alignment);
                yellow_msg!("This may restrict the maximum size of space you can allocate at a time.");
            }
            while order >= ORDER {
                order -= 1;
                alignment >>= 1;
            }
            unsafe { self.insert_block(current, order); }
            current += alignment;
            self.total += alignment;
        }
    }
    
    pub unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        let size = max(size_of::<usize>(), max(layout.size().next_power_of_two(), layout.align()));
        assert!(size.is_power_of_two());
        if layout.size() * 4 <= layout.align() {
            yellow_msg!("Current heap allocation may suffer from severe internal fragmentation when \
                alignment({} bytes) is much larger than target size({} bytes).", layout.align(), layout.size());
        }
        let order = size.trailing_zeros() as usize;
        if order >= ORDER {
            panic!("Allocation size({} bytes) too large. Please increase your ORDER.", size);
        }
        for i in order..ORDER {
            if let Some(addr) = unsafe { self.pop_front(i) } {
                for j in order..i {
                    unsafe { self.insert_block(addr + (1 << j), j); }
                }
                self.user += layout.size();
                self.allocated += size;
                
                if let Some(ptr) = NonNull::new(addr as *mut u8) {
                    return Ok(ptr);
                } else {
                    panic!("Memory in kernel buddy allocator should not be 0!");
                }
            }
        }
        self.report();
        Err(())
    }
    
    pub unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let size = max(size_of::<usize>(), max(layout.size().next_power_of_two(), layout.align()));
        let order = size.trailing_zeros() as usize;
        let mut addr = ptr.addr().get();
        'outer: for i in order..ORDER {
            if i == ORDER - 1 {
                unsafe { self.insert_block(addr, i); }
                break;
            }
            let buddy = addr ^ (1 << i);
            if self.head[i] == buddy {
                self.head[i] = unsafe { *(self.head[i] as *const usize) };
                addr = min(addr, buddy);
                continue;
            }
            let mut prev = self.head[i];
            let mut cur = 
                if self.head[i] != 0 {
                    unsafe { *(self.head[i] as *const usize) }
                } else { 
                    0 
                };
            while cur != 0 {
                if cur == buddy {
                    unsafe {
                        *(prev as *mut usize) = *(cur as *const usize);
                    }
                    addr = min(addr, buddy);
                    continue 'outer;
                }
                prev = cur;
                cur = unsafe { *(cur as *const usize) };
            }
            unsafe { self.insert_block(addr, i); }
            break;
        }
        self.allocated -= size;
        self.user -= layout.size();
    }
}

impl<const ORDER: usize> LockedAllocator<ORDER> {
    pub const fn empty() -> Self {
        Self {
            allocator: UnsafeCell::new(BuddyAllocator::empty()),
            lock: UnsafeCell::new(SpinLock::new())
        }
    }
    pub unsafe fn report(&self) {
        unsafe {
            (*(self.lock.get())).lock();
            (*(self.allocator.get())).report();
            (*(self.lock.get())).unlock();
        }
    }
    pub unsafe fn init(&self, start: usize, size: usize) {
        unsafe {
            (*(self.lock.get())).lock();
            (*(self.allocator.get())).init(start, size);
            (*(self.lock.get())).unlock();
        }
    }
}

unsafe impl<const ORDER: usize> GlobalAlloc for LockedAllocator<ORDER> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            (*(self.lock.get())).lock();
            let res = (*(self.allocator.get())).alloc(layout);
            (*(self.lock.get())).unlock();
            res.ok().unwrap().as_ptr()
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            (*(self.lock.get())).lock();
            (*(self.allocator.get())).dealloc(NonNull::<u8>::new(ptr).unwrap(), layout);
            (*(self.lock.get())).unlock();
        }
    }
}

unsafe impl<const ORDER: usize> Sync for LockedAllocator<ORDER> {}