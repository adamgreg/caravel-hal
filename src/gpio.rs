use embedded_hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin};

use caravel_pac::GpioRegisters;

/// Driver for the Caravel management GPIO pin
pub struct Gpio {
    regs: &'static GpioRegisters,
}

unsafe impl Send for Gpio {}

impl Gpio {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            regs: GpioRegisters::new(),
        }
    }

    /// Enable or disable the output driver
    #[inline]
    pub fn set_output_enable(&mut self, enable: bool) {
        unsafe {
            let input_enabled = self.regs.ien.read() != 0;

            self.regs.mode1.write(enable as u32);

            // mode0 (DM[0]) must be 1 for input enable if output disabled
            self.regs.mode0.write((!enable && input_enabled) as u32);

            self.regs.oe.write(enable as u32);
        }
    }

    /// Enable or disable the input buffer
    #[inline]
    pub fn set_input_enable(&mut self, enable: bool) {
        unsafe {
            // mode0 (DM[0]) must be 1 for input enable if output disabled
            let output_enabled = self.regs.oe.read() != 0;
            self.regs.mode0.write((enable && !output_enabled) as u32);

            self.regs.ien.write(enable as u32);
        }
    }
}

impl ErrorType for Gpio {
    type Error = core::convert::Infallible;
}

impl OutputPin for Gpio {
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.regs.output.write(0);
        }
        Ok(())
    }

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.regs.output.write(1);
        }
        Ok(())
    }
}

impl StatefulOutputPin for Gpio {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.regs.output.read() != 0)
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.regs.output.read() == 0)
    }
}

impl InputPin for Gpio {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.regs.input.read() != 0)
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.regs.input.read() == 0)
    }
}
