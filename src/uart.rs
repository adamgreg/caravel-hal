use caravel_pac::{UartEnabledOutRegister, UartRegisters};
use embedded_io::{ErrorType, Read, Write};

pub struct Uart {
    regs: &'static UartRegisters,
}

unsafe impl Send for Uart {}

impl Uart {
    pub fn new(enable: &UartEnabledOutRegister) -> Self {
        let this = Self {
            regs: UartRegisters::new(),
        };
        // Enable the UART
        unsafe {
            enable.value.write(1); // Enable bit
        }
        this
    }

    /// Non-blocking check: returns true if data is available to read
    #[inline(always)]
    pub fn rx_ready(&self) -> bool {
        self.regs.rxempty.read() == 0 // Ready when NOT empty
    }

    /// Non-blocking check: returns true if space is available to write
    #[inline(always)]
    pub fn tx_ready(&self) -> bool {
        self.regs.txfull.read() == 0
    }

    /// Enable/disable the TX event
    #[inline(always)]
    pub fn tx_event_enable(&self, enable: bool) {
        unsafe {
            self.regs.ev_enable.modify(|x| x.with_tx(enable));
        }
    }

    /// Enable/disable the RX event
    #[inline(always)]
    pub fn rx_event_enable(&self, enable: bool) {
        unsafe {
            self.regs.ev_enable.modify(|x| x.with_rx(enable));
        }
    }

    /// Check for a pending RX event
    #[inline(always)]
    pub fn rx_event_pending(&self) -> bool {
        self.regs.ev_pending.read().rx()
    }

    /// Clear the RX event flag
    #[inline(always)]
    pub fn clear_rx_event(&self) {
        unsafe {
            self.regs.ev_pending.modify(|x| x.with_rx(true));
        }
    }

    /// Check for a pending TX event
    #[inline(always)]
    pub fn tx_event_pending(&self) -> bool {
        self.regs.ev_pending.read().tx()
    }

    /// Clear the TX event flag
    #[inline(always)]
    pub fn clear_tx_event(&self) {
        unsafe {
            self.regs.ev_pending.modify(|x| x.with_tx(true));
        }
    }
}

// ---------- embedded-io blocking impls ----------

impl ErrorType for Uart {
    type Error = core::convert::Infallible;
}

impl Read for Uart {
    /// Reads *at least 1* byte, up to `buf.len()`, blocking until 1 is available.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        // Block until at least one byte is available in the RX FIFO
        while !self.rx_ready() {}

        // Read all available bytes (at least 1, up to buf.len())
        let mut n = 0;

        loop {
            // Read the byte
            buf[n] = self.regs.rxtx.read() as u8;

            // Clear the RX event pending flag
            unsafe {
                self.regs.ev_pending.modify(|x| x.with_rx(true));
            }

            // Break the loop and return if no more data is available or buffer is full
            n += 1;
            if !self.rx_ready() || n == buf.len() {
                break;
            }
        }

        Ok(n)
    }
}

impl Write for Uart {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        // Block until we can write at least one byte
        while !self.tx_ready() {}

        // Write as many bytes as possible without blocking
        let mut n = 0;
        for &byte in buf {
            if !self.tx_ready() {
                break;
            }
            unsafe {
                self.regs.rxtx.write(byte as u32);
            }
            n += 1;
        }
        Ok(n)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // Wait until TX is ready (not full = transmitter has space/is idle)
        while !self.tx_ready() {}
        Ok(())
    }
}
