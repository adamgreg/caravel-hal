#![no_std]

mod gpio;
mod interrupts;
mod timer;
mod uart;
mod user_io;

// Re-export user_register_block macro from PAC crate, for convenience
pub use caravel_pac::user_register_block;

pub use gpio::Gpio;
pub use timer::Timer0;
pub use uart::Uart;
pub use user_io::{UserIOBits, UserIo};

pub use interrupts::{CaravelInterrupt, enable_caravel_interrupts};
