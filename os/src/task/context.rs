#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12]
}

impl TaskContext {
    pub fn zero_init() -> Self {
        TaskContext {
            ra: 0,
            sp: 0,
            s: [0; 12]
        }
    }
    
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        unsafe extern "C" {
            safe fn __restore();
        }
        Self {
            ra: __restore as usize,
            sp: kstack_ptr,
            s: [0; 12]
        }
    }
}