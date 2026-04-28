# caravel-hal

A Hardware Abstraction Layer (HAL) for the
[Caravel](https://github.com/chipfoundry/caravel) &
[Caravan](https://github.com/chipfoundry/caravan) ChipIgnite management SoCs
from [ChipFoundry](https://chipfoundry.io/)

This crate provides safe, idiomatic Rust interfaces for interacting with
Caravel's hardware peripherals, including GPIO, UART, timers, interrupts, and
user I/O. It is designed to simplify embedded development on Caravel by
abstracting low-level register access and providing high-level APIs.

The UART driver implements the `Read` and `Write` traits from the
[embedded-io](https://docs.rs/embedded-io/latest/embedded_io/) crate.

The other drivers implement traits from the
[embedded-hal](https://docs.rs/embedded-hal/latest/embedded_hal/) crate.

## Features

- GPIO control (`embedded-hal` traits)
- UART communication (`embedded-io` traits)
- Timer management
- Interrupt dispatch
- User I/O pins

## Interrupts

The crate registers a handler for Machine External core interrupts, which
dispatches to user-defined handlers implemented using the
[riscv_rt::external_interrupt](https://docs.rs/riscv-rt/latest/riscv_rt/attr.external_interrupt.html)
attribute.

This behaviour can be disabled by disabling the default the `interrupts`
feature.

> [!IMPORTANT]
> "FAT" link-time optimisation breaks the interrupt dispatch. Set
> `lto = "thin"`, `lto = false` or `lto = "off"` in the build profile when using
> interrupts.
