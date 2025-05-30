// Ports and addresses of MMIO devices.
pub const UART_BASE_ADDR: usize = 0x10000000;
pub const SYSTEM_RESET_BASE_ADDR: usize = 0x100000;

// Memory layout
pub const KERNEL_BASE_ADDR: usize = 0x80000000;
pub const APP_BASE_ADDR: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;