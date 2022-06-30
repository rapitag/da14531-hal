#![no_std]

pub mod crg_aon;
pub mod gpio;
pub mod i2c;
pub mod prelude;
pub mod watchdog;

pub use da14531 as pac;
pub use embedded_hal as hal;

mod sealed {
    pub trait Sealed {}
}

pub(crate) use sealed::Sealed;

fn stripped_type_name<T>() -> &'static str {
    let s = core::any::type_name::<T>();
    let p = s.split("::");
    p.last().unwrap()
}
