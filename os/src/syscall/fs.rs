use crate::io::stdout::getchar;
use crate::mem::page_table::translated_byte_buffer;
use crate::print;
use crate::task::processor::current_user_token;

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            assert_eq!(len, 1, "Only support len = 1 in sys_read");
            let c = getchar();
            let mut buffers = translated_byte_buffer(current_user_token(), buf, len);
            unsafe {
                buffers[0].as_mut_ptr().write_volatile(c);
            }
            1
        }
        _ => {
            panic!("Unsupported fd = {} in sys_read!", fd)
        }
    }
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}