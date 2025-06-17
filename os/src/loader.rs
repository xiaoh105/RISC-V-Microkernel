use alloc::vec::Vec;
use lazy_static::lazy_static;
use crate::{blue_msg, println};

lazy_static! {
    static ref APP_NAMES: Vec<&'static str> =  {
        let num_app = get_num_app();
        unsafe extern "C" {
            safe fn _app_names();
        }
        let mut start = _app_names as usize as *const u8;
        let mut ret = Vec::new();
        unsafe {
            for _ in 0..num_app {
                let mut end = start;
                while end.read_volatile() != '\0' as u8 {
                    end = end.add(1);
                }
                let slice = core::slice::from_raw_parts(start, end.offset_from(start) as usize);
                let str = core::str::from_utf8(slice).unwrap();
                ret.push(str);
                start = end.add(1);
            }
        }
        ret
    };
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

pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
    let num_app = get_num_app();
    (0..num_app)
        .find(|&i| APP_NAMES[i] == name)
        .map(|i| get_app_data(i))
}

pub fn list_apps() {
    blue_msg!("[kernel] ----- APPS -----");
    for app in APP_NAMES.iter() {
        println!("{}", app);
    }
    blue_msg!("[kernel] ----------------");
}