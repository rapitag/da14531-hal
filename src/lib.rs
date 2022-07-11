#![no_std]

pub mod clock;
pub mod crg_aon;
pub mod gpio;
pub mod i2c;
pub mod interrupt;
pub mod sleep;
pub mod timer;
pub mod wakeup;
pub mod watchdog;

pub use cortex_m as cm;
pub use da14531 as pac;
pub use embedded_hal as hal;
