#![no_std]

mod gpio;
mod housekeeping_spi;
mod timer0;
mod uart;
mod user_io;

#[cfg(feature = "interrupts")]
mod interrupts;

// Re-export user_register_block macro from PAC crate, for convenience
pub use caravel_pac::user_register_block;

pub use gpio::Gpio;
pub use housekeeping_spi::HousekeepingSpi;
pub use timer0::Timer0;
pub use uart::Uart;
pub use user_io::{UserIOBits, UserIo};

#[cfg(feature = "interrupts")]
pub use interrupts::{CaravelInterrupt, enable_caravel_interrupts};
