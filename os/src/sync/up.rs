use core::cell::{RefCell, RefMut};

pub struct UPSafeCell<T> {
    cell: RefCell<T>
}

unsafe impl<T> Sync for UPSafeCell<T> {}

/// A RefCell that is safe in uni-core processors.
impl<T> UPSafeCell<T> {
    pub unsafe fn new(value: T) -> Self {
        Self {
            cell: RefCell::new(value)
        }
    }
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.cell.borrow_mut()
    }
}