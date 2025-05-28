use core::fmt;
use core::fmt::Write;
use crate::drivers::uart::UartPort;

pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for c in s.bytes() {
            UartPort.send(c);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        print(format_args!($fmt $(, $($arg)+)?));
    }
}


#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::stdout::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! green_msg {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::stdout::print(format_args!(concat!("\x1B[32m", $fmt, "\x1B[0m\n") $(, $($arg)+)?));
    }
}