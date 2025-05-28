use core::ptr::NonNull;
use volatile::VolatilePtr;
use volatile::access::*;
use bitflags::bitflags;
use crate::config::UART_BASE_ADDR;

macro_rules! wait_for {
    ($cond: expr) => {
        while !$cond {
            core::hint::spin_loop();
        }
    };
}

pub struct ReadPort;

pub struct WritePort;

pub struct UartPort;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct InterruptEnable: u8 {
        const RBR_AVAILABLE = 1 << 0;
        const THR_EMPTY = 1 << 1;
        const RX_LSR_CHANGE = 1 << 2;
        const MSR_CHANGE = 1 << 3;
    }

    #[derive(Clone, Copy)]
    pub struct InterruptIdentify: u8 {}

    #[derive(Clone, Copy)]
    pub struct FifoControl: u8 {
        const ENABLE_FIFO = 1 << 0;
        const CLEAR_RX_FIFO = 1 << 1;
        const CLEAR_TX_FIFO = 1 << 2;
        const DMA_MODE = 1 << 3;
        const BUF_1B = (0b00) << 4;
        const BUF_4B = (0b01) << 4;
        const BUF_8B = (0b10) << 4;
        const BUF_14B = (0b11) << 4;
    }

    #[derive(Clone, Copy)]
    pub struct LineControl: u8 {
        const DATA_5b = (0b00) << 0;
        const DATA_6b = (0b01) << 0;
        const DATA_7b = (0b10) << 0;
        const DATA_8b = (0b11) << 0;
        const STOP_2b = 1 << 2;
        const ODD_PARITY = (0b001) << 3;
        const EVEN_PARITY = (0b011) << 3;
        const HIGH_PARITY = (0b101) << 3;
        const LOW_PARITY = (0b111) << 3;
        const ENABLE_BREAK_SIG = 1 << 6;
        const ENABLE_DALAB = 1 << 7;
    }

    #[derive(Clone, Copy)]
    pub struct ModemControl: u8 {
        const DATA_TERMINAL_READY = 1 << 0;
        const REQUEST_TO_SEND = 1 << 1;
        /// Used as the controller for interrupt process.
        const AUX_OUTPUT_2 = 1 << 3;
        const LOOPBACK_MODE = 1 << 4;
    }

    #[derive(Clone, Copy)]
    pub struct LineStatus: u8 {
        const DATA_AVAILABLE = 1 << 0;
        const OVERRUN_ERR = 1 << 1;
        const PARITY_ERR = 1 << 2;
        const FRAMING_ERR = 1 << 3;
        const BREAK_SIG = 1 << 4;
        const TX_EMPTY = 1 << 5;
        const TX_EMPTY_LINE_IDLE = 1 << 6;
        const FIFO_ERR_DATA = 1 << 7;
    }

    #[derive(Clone, Copy)]
    pub struct ModemStatus: u8 {}
}

impl ReadPort {
    const RBR: VolatilePtr<'static, u8, ReadOnly> = unsafe {
        VolatilePtr::new_read_only(NonNull::new_unchecked(UART_BASE_ADDR as *mut _))
    };
    const IER: VolatilePtr<'static, InterruptEnable, ReadWrite> = unsafe {
        VolatilePtr::new_restricted(ReadWrite,
                                    NonNull::new_unchecked((UART_BASE_ADDR + 1) as *mut _))
    };
    const IIR: VolatilePtr<'static, InterruptIdentify, ReadOnly> = unsafe {
        VolatilePtr::new_read_only(NonNull::new_unchecked((UART_BASE_ADDR + 2) as *mut _))
    };
    const LCR: VolatilePtr<'static, LineControl, ReadWrite> = unsafe {
        VolatilePtr::new_restricted(ReadWrite,
                                    NonNull::new_unchecked((UART_BASE_ADDR + 3) as *mut _))
    };
    const MCR: VolatilePtr<'static, ModemControl, ReadWrite> = unsafe {
        VolatilePtr::new_restricted(ReadWrite, 
                                    NonNull::new_unchecked((UART_BASE_ADDR + 4) as *mut _))
    };
    const LSR: VolatilePtr<'static, LineStatus, ReadOnly> = unsafe {
        VolatilePtr::new_read_only(NonNull::new_unchecked((UART_BASE_ADDR + 5) as *mut _))
    };
    const MSR: VolatilePtr<'static, ModemStatus, ReadOnly> = unsafe {
        VolatilePtr::new_read_only(NonNull::new_unchecked((UART_BASE_ADDR + 6) as *mut _))
    };
}

impl WritePort {
    const THR: VolatilePtr<'static, u8, WriteOnly> = unsafe {
        VolatilePtr::new_restricted(WriteOnly, NonNull::new_unchecked(UART_BASE_ADDR as *mut _))
    };
    const IER: VolatilePtr<'static, InterruptEnable, ReadWrite> = unsafe {
        VolatilePtr::new_restricted(ReadWrite, 
                                    NonNull::new_unchecked((UART_BASE_ADDR + 1) as *mut _))
    };
    const FCR: VolatilePtr<'static, FifoControl, WriteOnly> = unsafe {
        VolatilePtr::new_restricted(WriteOnly, 
                                    NonNull::new_unchecked((UART_BASE_ADDR + 2) as *mut _))
    };
    const LCR: VolatilePtr<'static, LineControl, ReadWrite> = unsafe {
        VolatilePtr::new_restricted(ReadWrite,
                                    NonNull::new_unchecked((UART_BASE_ADDR + 3) as *mut _))
    };
    const MCR: VolatilePtr<'static, ModemControl, ReadWrite> = unsafe {
        VolatilePtr::new_restricted(ReadWrite,
                                    NonNull::new_unchecked((UART_BASE_ADDR + 4) as *mut _))
    };
    const LSR: VolatilePtr<'static, LineStatus, ReadOnly> = unsafe {
        VolatilePtr::new_read_only(NonNull::new_unchecked((UART_BASE_ADDR + 5) as *mut _))
    };
    const MSR: VolatilePtr<'static, ModemStatus, ReadOnly> = unsafe {
        VolatilePtr::new_read_only(NonNull::new_unchecked((UART_BASE_ADDR + 6) as *mut _))
    };
}

impl UartPort {
    pub fn init(&self) {
        // Disable all sorts of interrupts
        ReadPort::IER.write(InterruptEnable::empty());

        // Enable DLAB
        ReadPort::LCR.write(LineControl::ENABLE_DALAB);

        // Set baud rate = 38.4K, divisor = 3
        WritePort::THR.write(0x03); // DLL = 0x03.
        WritePort::IER.write(InterruptEnable::empty()); // DLM = 0x00.

        // Disable DLAB, and set data length to 8 bits without check
        ReadPort::LCR.write(LineControl::DATA_8b);

        // Enable FIFO, clear all queues and set interrupt watermark to 14 bytes
        WritePort::FCR.write(FifoControl::ENABLE_FIFO | FifoControl::BUF_14B |
            FifoControl::CLEAR_RX_FIFO | FifoControl::CLEAR_TX_FIFO);

        // Set terminal ready and enable auxiliary output 2 (i.e. interrupt enable)
        ReadPort::MCR.write(ModemControl::DATA_TERMINAL_READY | ModemControl::AUX_OUTPUT_2);

        // Enable interrupts at receiver end
        ReadPort::IER.write(InterruptEnable::RBR_AVAILABLE);
    }

    fn wait_and_send(&self, data: u8) {
        wait_for!((WritePort::LSR.read() | LineStatus::TX_EMPTY).bits() != 0);
        WritePort::THR.write(data);
    }

    pub fn send(&self, data: u8) {
        if data == 0x08 || data == 0x7f {
            self.wait_and_send(0x08);
            self.wait_and_send(b' ');
            self.wait_and_send(0x08);
        } else {
            self.wait_and_send(data);
        }
    }
}