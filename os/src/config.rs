// Ports and addresses of MMIO devices.
pub const UART_BASE_ADDR: usize = 0x1000_0000;
pub const SYSTEM_RESET_BASE_ADDR: usize = 0x10_0000;
pub const TIMER_ADDR: usize = 0x0200_bff8;
pub const TIMER_CMP_BASE_ADDR: usize = 0x0200_4000;

// Memory layout
pub const KERNEL_BASE_ADDR: usize = 0x8000_0000;
pub const APP_BASE_ADDR: usize = 0x8040_0000;
pub const APP_SIZE_LIMIT: usize = 0x2_0000;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE : usize = 0x30_0000;
pub const MEMORY_END : usize = 0x8080_0000;

// Environment config
pub const MAX_APP_NUM: usize = 16;
pub const CPUS: usize = 1;

// Timer config
pub const TIMER_FREQ: usize = 12_500_000;
pub const SCHED_PERIOD: usize = 125_000;