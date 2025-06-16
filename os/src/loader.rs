use crate::config::*;

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