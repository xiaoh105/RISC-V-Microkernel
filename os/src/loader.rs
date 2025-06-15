use core::arch::asm;

use crate::config::*;
use crate::trap::context::TrapContext;

#[derive(Clone, Copy)]
#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE]
}

#[derive(Clone, Copy)]
#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE]
}

static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack { data: [0; KERNEL_STACK_SIZE] }; MAX_APP_NUM];
static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack { data: [0; USER_STACK_SIZE] };  MAX_APP_NUM];

fn get_base_i(id: usize) -> usize {
    APP_BASE_ADDR + id * APP_SIZE_LIMIT
}

pub fn get_num_app() -> usize {
    unsafe extern "C" {
        safe fn _num_app();
    }
    unsafe {
        (_num_app as usize as *const usize).read_volatile()
    }
}

pub fn get_app_data(id: usize) -> &'static [u8] {
    unsafe extern "C" {
        safe fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe {
        core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
    };
    assert!(id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[id] as *const u8, 
            app_start[id + 1] -  app_start[id]
        )
    }
}

pub fn load_apps() {
    unsafe extern "C" {
        safe fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe {
        core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
    };
    for i in 0..num_app {
        let base_i = get_base_i(i);
        (base_i..base_i + APP_SIZE_LIMIT).for_each(|addr| unsafe {
            (addr as *mut u8).write_volatile(0);
        });
        let src = unsafe {
            core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i])
        };
        let dst = unsafe {
            core::slice::from_raw_parts_mut(base_i as *mut u8, src.len())
        };
        dst.copy_from_slice(src);
    }
    unsafe {
        asm!("fence.i");
    }
}

pub fn init_app_cx(id: usize) -> usize {
    KERNEL_STACK[id].push_context(
        TrapContext::app_init_context(get_base_i(id), USER_STACK[id].get_sp())
    )
}

impl KernelStack {
    fn get_sp(&self) -> usize { self.data.as_ptr() as usize + KERNEL_STACK_SIZE }
    pub fn push_context(&self, cx: TrapContext) -> usize {
        let ptr = (self.get_sp() - size_of::<usize>()) as *mut TrapContext;
        unsafe {
            *ptr = cx;
            ptr as usize
        }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize { self.data.as_ptr() as usize + USER_STACK_SIZE }
}