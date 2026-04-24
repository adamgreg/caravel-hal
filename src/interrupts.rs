// Re-export CaravelInterrupt for ease-of-use
pub use caravel_pac::CaravelInterrupt;

/// Default interrupt dispatcher for Caravel external interrupts
pub fn default_handle_machine_external_interrupt() {
    unsafe extern "C" {
        fn _dispatch_external_interrupt(code: usize);

    }
    let pending = vexriscv::register::vmip::read();
    if pending == 0 {
        return;
    }
    let next_pending = pending.trailing_zeros() as usize;
    unsafe {
        _dispatch_external_interrupt(next_pending);
    }
}

/// Enable Caravel external interrupt sources
pub fn enable_caravel_interrupts() {
    critical_section::with(|_| unsafe {
        // Enable Caravel ("machine external") interrupts
        riscv::interrupt::enable_interrupt(riscv::interrupt::Interrupt::MachineExternal);

        // Enable interrupts globally
        riscv::interrupt::enable();
    });
}

/// Register external interrupt dispatcher
#[cfg(not(feature = "no-interrupts"))]
#[riscv_rt::core_interrupt(riscv::interrupt::Interrupt::MachineExternal)]
fn _handle_machine_external_interrupt() {
    default_handle_machine_external_interrupt();
}
