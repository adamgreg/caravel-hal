use caravel_pac::MprjWbIenaOutRegister;

pub struct Wishbone {
    reg: &'static MprjWbIenaOutRegister,
}

unsafe impl Send for Wishbone {}

impl Wishbone {
    /// Constructs the Wishbone bus driver.
    #[inline]
    pub const fn new() -> Self {
        Self {
            reg: MprjWbIenaOutRegister::new(),
        }
    }

    /// Enable the Wishbone bus
    pub fn enable(&self) {
        unsafe {
            self.reg.value.write(1);
        }
    }

    /// Disable the Wishbone bus
    pub fn disable(&self) {
        unsafe {
            self.reg.value.write(0);
        }
    }
}
