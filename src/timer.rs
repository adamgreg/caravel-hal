use embedded_hal::delay::DelayNs;

use caravel_pac::{CaravelInterrupt, Timer0Registers};

pub struct Timer0 {
    regs: &'static Timer0Registers,
    /// Precomputed factor: ticks per nanosecond * 2^32 for fixed-point maths
    ns_to_ticks_factor: u64,
}

unsafe impl Send for Timer0 {}

impl Timer0 {
    /// Constructs the Timer0 driver.
    /// `ticks_per_s` should be the clock speed of the core, in Hertz
    #[inline]
    #[allow(clippy::new_without_default)]
    pub const fn new(ticks_per_s: u32) -> Self {
        // Precompute fixed-point conversion factor
        Self {
            regs: Timer0Registers::new(),
            ns_to_ticks_factor: ((ticks_per_s as u64) << 32) / 1_000_000_000,
        }
    }

    /// Enable the Timer0 interrupt source
    /// Define the ISR using the `riscv_rt::external_interrupt` attribute
    #[inline(always)]
    pub fn enable_interrupt(&mut self) {
        unsafe {
            riscv::interrupt::enable_interrupt(riscv::interrupt::Interrupt::MachineExternal);
            self.regs.ev_enable.modify(|x| x.with_zero(true));
        }
        CaravelInterrupt::Timer0.enable();
    }

    /// Disable the Timer0 interrupt source
    #[inline(always)]
    pub fn disable_interrupt(&mut self) {
        unsafe {
            self.regs.ev_enable.modify(|x| x.with_zero(false));
        }
        CaravelInterrupt::Timer0.disable();
    }

    /// Set the timer to fire periodically
    #[inline(always)]
    pub fn set_periodic(&mut self, period_ns: u32) {
        // Fast 32-bit fixed-point multiply: (ns * factor) >> 32
        let ticks = ((period_ns as u64 * self.ns_to_ticks_factor) >> 32) as u32;

        unsafe {
            // Configure timer for periodic operation
            self.regs.en.write(0);
            self.regs.load.write(ticks);
            self.regs.reload.write(ticks);
            self.regs.en.write(1);
        }
    }

    /// Disable the timer immediately
    #[inline(always)]
    pub fn disable(&mut self) {
        unsafe {
            self.regs.en.write(0);
        }
    }

    /// Check for a pending event
    #[inline(always)]
    pub fn event_pending(&self) -> bool {
        self.regs.ev_pending.read().zero()
    }

    /// Clear the pending event flag
    #[inline(always)]
    pub fn clear_event(&self) {
        unsafe {
            self.regs.ev_pending.modify(|x| x.with_zero(true));
        }
    }
}

impl DelayNs for Timer0 {
    #[inline]
    fn delay_ns(&mut self, ns: u32) {
        // Fast 32-bit fixed-point multiply: (ns * factor) >> 32
        let ticks = ((ns as u64 * self.ns_to_ticks_factor) >> 32) as u32;

        unsafe {
            // Configure timer for a single-shot countdown
            self.regs.en.write(0);
            self.regs.load.write(ticks);
            self.regs.en.write(1);

            // Loop, waiting for value to reach zero
            self.regs.update_value.write(1);
            while self.regs.value.read() > 0 {
                self.regs.update_value.write(1);
            }
        }
    }
}
