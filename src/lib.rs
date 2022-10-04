#![no_std]

pub mod adc;
pub mod crg_aon;
pub mod crg_top;
pub mod gpio;
pub mod i2c;
pub mod nvic;
pub mod sys_wdog;
pub mod timer;
pub mod wkup;

pub use cortex_m as cm;
pub use da14531 as pac;
pub use embedded_hal as hal;

mod sealed {
    pub trait Sealed {}
}

pub(crate) use sealed::Sealed;
