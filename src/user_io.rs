use embedded_hal::digital::{ErrorType, InputPin, OutputPin};

use caravel_pac::{UserProjectRegisters, bitfields::UserIoXferBits};

pub use caravel_pac::bitfields::UserIOBits;

/// Driver for a single Caravel user I/O pin
pub struct UserIoPin {
    regs: &'static UserProjectRegisters,
    /// The pin number (0 to 37)
    pub n: usize,
}

unsafe impl Send for UserIoPin {}

impl UserIoPin {
    /// Modify the pin's configuration using a closure.
    ///
    /// The closure is passed a `UserIOBits`
    /// [bitfield-struct](https://docs.rs/bitfield-struct/latest/bitfield_struct/)
    /// containing the current value, which it can modify and return.
    /// The updated value is then written back to the register.
    #[inline]
    pub fn configure(&mut self, f: impl FnOnce(UserIOBits) -> UserIOBits) {
        unsafe {
            self.regs.io[self.n].modify(f);
        }
    }

    /// Toggle the output state of the pin. Requires input enabled.
    #[inline]
    pub fn toggle(&mut self) -> Result<(), core::convert::Infallible> {
        if self.is_high()? {
            self.set_low()
        } else {
            self.set_high()
        }
    }
}

impl ErrorType for UserIoPin {
    type Error = core::convert::Infallible;
}

impl OutputPin for UserIoPin {
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        if self.n < 32 {
            unsafe {
                self.regs.datal.modify(|x| x & !(1 << self.n));
            }
        } else {
            unsafe {
                self.regs.datah.modify(|x| x & !(1 << (self.n - 32)));
            }
        }
        Ok(())
    }

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        if self.n < 32 {
            unsafe {
                self.regs.datal.modify(|x| x | (1 << self.n));
            }
        } else {
            unsafe {
                self.regs.datah.modify(|x| x | (1 << (self.n - 32)));
            }
        }
        Ok(())
    }
}

impl InputPin for UserIoPin {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(if self.n < 32 {
            (self.regs.datal.read() >> self.n) & 1 != 0
        } else {
            (self.regs.datah.read() >> (self.n - 32)) & 1 != 0
        })
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

/// Builder for configuring Caravel user I/O pins
pub struct UserIoBuilder {
    regs: &'static UserProjectRegisters,
    config: [UserIOBits; 38],
}

impl UserIoBuilder {
    const fn new() -> Self {
        Self {
            regs: UserProjectRegisters::new(),
            config: const {
                let mut arr = [UserIOBits::MGMT_STD_INPUT_NOPULL; 38];
                // Set custom values for first 2 pins
                arr[0] = UserIOBits::MGMT_STD_ANALOG; // Debug pin
                arr[1] = UserIOBits::MGMT_STD_OUTPUT; // Housekeeping SPI SDO pin
                arr
            },
        }
    }

    /// Configure a specific pin
    pub const fn pin(mut self, n: usize, bits: UserIOBits) -> Self {
        self.config[n] = bits;
        self
    }

    /// Build and initialize the UserIo, performing the transfer
    pub fn xfer(self) -> UserIo {
        unsafe {
            // Write all pin configurations
            for (i, bits) in self.config.iter().enumerate() {
                self.regs.io[i].write(*bits);
            }
        }

        // Create the UserIo instance, perform the transfer, and return
        let mut user_io = UserIo { regs: self.regs };
        user_io.transfer();
        user_io
    }
}

/// Container for the full set of Caravel user I/O pins
pub struct UserIo {
    regs: &'static UserProjectRegisters,
}

unsafe impl Send for UserIo {}

impl UserIo {
    /// Create a builder for configuring user I/O pins.
    /// This is the way to create a `UserIo` instance.
    /// The configuration for individual pins can be set on the builder,
    /// and then the configuration is applied and the `UserIo` instance is created
    /// by calling `xfer()`.
    ///
    /// Example:
    /// ```no_run
    /// use caravel_hal::UserIo;
    ///
    /// let user_io = UserIo::configure()
    ///     .pin(1, UserIo::USER_STD_OUTPUT)
    ///     .pin(5, UserIo::USER_STD_INPUT_NOPULL)
    ///     .xfer();
    /// ```
    pub const fn configure() -> UserIoBuilder {
        UserIoBuilder::new()
    }

    /// Compile-time pin access
    pub fn pin<const N: usize>(&self) -> UserIoPin {
        assert!(N < 38, "User I/O pin number must be 0-37");
        UserIoPin {
            regs: self.regs,
            n: N,
        }
    }

    /// Runtime pin access
    #[inline]
    pub fn pin_checked(&mut self, n: usize) -> Option<UserIoPin> {
        (n < 38).then_some(UserIoPin { regs: self.regs, n })
    }

    /// Transfer the full user I/O configuration to the actual I/O pads
    pub fn transfer(&mut self) {
        unsafe {
            self.regs
                .xfer
                .write(UserIoXferBits::new().with_xfer_busy(true));
        }
        // Wait for transfer to complete
        while self.regs.xfer.read().xfer_busy() {}
    }
}

// Make standard UserIOBits configurations available as associated constants for convenience
impl UserIo {
    pub const MGMT_STD_INPUT_NOPULL: UserIOBits = UserIOBits::MGMT_STD_INPUT_NOPULL;
    pub const MGMT_STD_INPUT_PULLDOWN: UserIOBits = UserIOBits::MGMT_STD_INPUT_PULLDOWN;
    pub const MGMT_STD_INPUT_PULLUP: UserIOBits = UserIOBits::MGMT_STD_INPUT_PULLUP;
    pub const MGMT_STD_OUTPUT: UserIOBits = UserIOBits::MGMT_STD_OUTPUT;
    pub const MGMT_STD_BIDIRECTIONAL: UserIOBits = UserIOBits::MGMT_STD_BIDIRECTIONAL;
    pub const MGMT_STD_OUT_MONITORED: UserIOBits = UserIOBits::MGMT_STD_OUT_MONITORED;
    pub const MGMT_STD_ANALOG: UserIOBits = UserIOBits::MGMT_STD_ANALOG;

    // User interface standard modes
    pub const USER_STD_INPUT_NOPULL: UserIOBits = UserIOBits::USER_STD_INPUT_NOPULL;
    pub const USER_STD_INPUT_PULLDOWN: UserIOBits = UserIOBits::USER_STD_INPUT_PULLDOWN;
    pub const USER_STD_INPUT_PULLUP: UserIOBits = UserIOBits::USER_STD_INPUT_PULLUP;
    pub const USER_STD_OUTPUT: UserIOBits = UserIOBits::USER_STD_OUTPUT;
    pub const USER_STD_BIDIRECTIONAL: UserIOBits = UserIOBits::USER_STD_BIDIRECTIONAL;
    pub const USER_STD_OUT_MONITORED: UserIOBits = UserIOBits::USER_STD_OUT_MONITORED;
    pub const USER_STD_ANALOG: UserIOBits = UserIOBits::USER_STD_ANALOG;
}
