use core::marker::PhantomData;

mod convert;
use convert::PinMode;
mod partially_erased;
pub use partially_erased::{PEPin, PartiallyErasedPin};
mod erased;
pub use erased::{EPin, ErasedPin};

pub use embedded_hal::digital::v2::PinState;

use core::fmt;

/// A filler pin type
#[derive(Debug)]
pub struct NoPin;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The parts to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self) -> Self::Parts;
}

/// Id, port and mode for any pin
pub trait PinExt {
    /// Current pin mode
    type Mode;
    /// Pin number
    fn pin_id(&self) -> u8;
    /// Port number starting from 0
    fn port_id(&self) -> u8;
}

/// Some alternate mode (type state)
pub struct Alternate<const A: u8, Otype = PushPull>(PhantomData<Otype>);

/// Input mode (type state)
pub struct Input;

/// Pull setting for an input.
#[derive(Debug, Eq, PartialEq)]
pub enum Pull {
    /// Floating
    None = 0,
    /// Pulled up
    Up = 1,
    /// Pulled down
    Down = 2,
}

// /// Open drain input or output (type state)
// pub struct OpenDrain;

/// Output mode (type state)
pub struct Output<MODE = PushPull> {
    _mode: PhantomData<MODE>,
}

/// Push pull output (type state)
pub struct PushPull;

/// Analog mode (type state)
pub struct Analog;

/// JTAG/SWD mote (type state)
pub type Debugger = Alternate<0, PushPull>;

mod marker {
    /// Marker trait that show if `ExtiPin` can be implemented
    pub trait Interruptable {}
    /// Marker trait for readable pin modes
    pub trait Readable {}
    /// Marker trait for slew rate configurable pin modes
    pub trait OutputSpeed {}
    /// Marker trait for active pin modes
    pub trait Active {}
    /// Marker trait for all pin modes except alternate
    pub trait NotAlt {}
    /// Marker trait for pins with alternate function `A` mapping
    pub trait IntoAf<const A: u8> {}
}

impl<MODE> marker::Interruptable for Output<MODE> {}
impl marker::Interruptable for Input {}
impl marker::Readable for Input {}
// impl marker::Readable for Output<OpenDrain> {}
impl marker::Active for Input {}
impl<Otype> marker::OutputSpeed for Output<Otype> {}
impl<const A: u8, Otype> marker::OutputSpeed for Alternate<A, Otype> {}
impl<Otype> marker::Active for Output<Otype> {}
impl<const A: u8, Otype> marker::Active for Alternate<A, Otype> {}
impl marker::NotAlt for Input {}
impl<Otype> marker::NotAlt for Output<Otype> {}
impl marker::NotAlt for Analog {}

/// GPIO interrupt trigger edge selection
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Edge {
    /// Rising edge of voltage
    Rising,
    /// Falling edge of voltage
    Falling,
    /// Rising and falling edge of voltage
    RisingFalling,
}

macro_rules! af {
    ($($i:literal: $AFi:ident),+) => {
        $(
            #[doc = concat!("Alternate function ", $i, " (type state)" )]
            pub type $AFi<Otype = PushPull> = Alternate<$i, Otype>;
        )+
    };
}

af!(
    0: AF0,
    1: AF1,
    2: AF2,
    3: AF3,
    4: AF4,
    5: AF5,
    6: AF6,
    7: AF7,
    8: AF8,
    9: AF9,
    10: AF10,
    11: AF11,
    12: AF12,
    13: AF13,
    14: AF14,
    15: AF15
);

/// Generic pin type
///
/// - `MODE` is one of the pin modes (see [Modes](crate::gpio#modes) section).
/// - `N` is pin number: from `0` to `15`.
pub struct Pin<const N: u8, MODE = Input> {
    _mode: PhantomData<MODE>,
}
impl<const N: u8, MODE> Pin<N, MODE> {
    const fn new() -> Self {
        Self { _mode: PhantomData }
    }
}

impl<const N: u8, MODE> fmt::Debug for Pin<N, MODE> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!(
            "P0_{}<{}>",
            N,
            crate::stripped_type_name::<MODE>()
        ))
    }
}

impl<const N: u8, MODE> PinExt for Pin<N, MODE> {
    type Mode = MODE;

    #[inline(always)]
    fn pin_id(&self) -> u8 {
        N
    }
    #[inline(always)]
    fn port_id(&self) -> u8 {
        0
    }
}

impl<const N: u8, MODE> Pin<N, MODE>
where
    MODE: marker::Active,
{
    /// Set the internal pull-up and pull-down resistor
    pub fn set_internal_resistor(&mut self, resistor: Pull) {
        let value = resistor as u8;
        unsafe {
            match N {
                0 => {
                    (*Gpio::ptr())
                        .p00_mode_reg
                        .modify(|r, w| w.pupd().bits(r.pupd().bits() | value));
                }
                1 => {
                    (*Gpio::ptr())
                        .p01_mode_reg
                        .modify(|r, w| w.pupd().bits(r.pupd().bits() | value));
                }
                2 => {
                    (*Gpio::ptr())
                        .p02_mode_reg
                        .modify(|r, w| w.pupd().bits(r.pupd().bits() | value));
                }
                3 => {
                    (*Gpio::ptr())
                        .p03_mode_reg
                        .modify(|r, w| w.pupd().bits(r.pupd().bits() | value));
                }
                4 => {
                    (*Gpio::ptr())
                        .p04_mode_reg
                        .modify(|r, w| w.pupd().bits(r.pupd().bits() | value));
                }
                5 => {
                    (*Gpio::ptr())
                        .p05_mode_reg
                        .modify(|r, w| w.pupd().bits(r.pupd().bits() | value));
                }
                6 => {
                    (*Gpio::ptr())
                        .p06_mode_reg
                        .modify(|r, w| w.pupd().bits(r.pupd().bits() | value));
                }
                7 => {
                    (*Gpio::ptr())
                        .p07_mode_reg
                        .modify(|r, w| w.pupd().bits(r.pupd().bits() | value));
                }
                8 => {
                    (*Gpio::ptr())
                        .p08_mode_reg
                        .modify(|r, w| w.pupd().bits(r.pupd().bits() | value));
                }
                9 => {
                    (*Gpio::ptr())
                        .p09_mode_reg
                        .modify(|r, w| w.pupd().bits(r.pupd().bits() | value));
                }
                10 => {
                    (*Gpio::ptr())
                        .p010_mode_reg
                        .modify(|r, w| w.pupd().bits(r.pupd().bits() | value));
                }
                11 => {
                    (*Gpio::ptr())
                        .p011_mode_reg
                        .modify(|r, w| w.pupd().bits(r.pupd().bits() | value));
                }
                _ => {}
            };
        }
    }

    /// Set the internal pull-up and pull-down resistor
    pub fn internal_resistor(mut self, resistor: Pull) -> Self {
        self.set_internal_resistor(resistor);
        self
    }

    /// Enables / disables the internal pull up
    pub fn internal_pull_up(self, on: bool) -> Self {
        if on {
            self.internal_resistor(Pull::Up)
        } else {
            self.internal_resistor(Pull::None)
        }
    }

    /// Enables / disables the internal pull down
    pub fn internal_pull_down(self, on: bool) -> Self {
        if on {
            self.internal_resistor(Pull::Down)
        } else {
            self.internal_resistor(Pull::None)
        }
    }
}

impl<const N: u8, MODE> Pin<N, MODE> {
    /// Erases the pin number from the type
    ///
    /// This is useful when you want to collect the pins into an array where you
    /// need all the elements to have the same type
    pub fn erase_number(self) -> PartiallyErasedPin<MODE> {
        PartiallyErasedPin::new(N)
    }

    /// Erases the pin number and the port from the type
    ///
    /// This is useful when you want to collect the pins into an array where you
    /// need all the elements to have the same type
    pub fn erase(self) -> ErasedPin<MODE> {
        ErasedPin::new(0, N)
    }
}

impl<const N: u8, MODE> Pin<N, MODE> {
    /// Set the output of the pin regardless of its mode.
    /// Primarily used to set the output value of the pin
    /// before changing its mode to an output to avoid
    /// a short spike of an incorrect value
    #[inline(always)]
    fn _set_state(&mut self, state: PinState) {
        match state {
            PinState::High => self._set_high(),
            PinState::Low => self._set_low(),
        }
    }
    #[inline(always)]
    fn _set_high(&mut self) {
        // NOTE(unsafe) atomic write to a stateless register
        unsafe {
            (*Gpio::ptr())
                .p0_set_data_reg
                .write(|w| w.p0_set().bits(1 << N))
        }
    }
    #[inline(always)]
    fn _set_low(&mut self) {
        // NOTE(unsafe) atomic write to a stateless register
        unsafe {
            (*Gpio::ptr())
                .p0_reset_data_reg
                .write(|w| w.p0_reset().bits(1 << N))
        }
    }
    #[inline(always)]
    fn _is_set_low(&self) -> bool {
        // NOTE(unsafe) atomic read with no side effects
        unsafe { (*Gpio::ptr()).p0_data_reg.read().bits() & (1 << N) == 0 }
    }
    #[inline(always)]
    fn _is_low(&self) -> bool {
        // NOTE(unsafe) atomic read with no side effects
        unsafe { (*Gpio::ptr()).p0_data_reg.read().bits() & (1 << N) == 0 }
    }
}

impl<const N: u8, MODE> Pin<N, Output<MODE>> {
    /// Drives the pin high
    #[inline(always)]
    pub fn set_high(&mut self) {
        self._set_high()
    }

    /// Drives the pin low
    #[inline(always)]
    pub fn set_low(&mut self) {
        self._set_low()
    }

    /// Is the pin in drive high or low mode?
    #[inline(always)]
    pub fn get_state(&self) -> PinState {
        if self.is_set_low() {
            PinState::Low
        } else {
            PinState::High
        }
    }

    /// Drives the pin high or low depending on the provided value
    #[inline(always)]
    pub fn set_state(&mut self, state: PinState) {
        match state {
            PinState::Low => self.set_low(),
            PinState::High => self.set_high(),
        }
    }

    /// Is the pin in drive high mode?
    #[inline(always)]
    pub fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    /// Is the pin in drive low mode?
    #[inline(always)]
    pub fn is_set_low(&self) -> bool {
        self._is_set_low()
    }

    /// Toggle pin output
    #[inline(always)]
    pub fn toggle(&mut self) {
        if self.is_set_low() {
            self.set_high()
        } else {
            self.set_low()
        }
    }
}

impl<const N: u8, MODE> Pin<N, MODE>
where
    MODE: marker::Readable,
{
    /// Is the input pin high?
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        !self.is_low()
    }

    /// Is the input pin low?
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        self._is_low()
    }
}

macro_rules! gpio {
    ($GPIOX:ident, $gpiox:ident, $PEPin:ident, $port_id:expr, $PXn:ident, [
        $($PXi:ident: ($pxi:ident, $i:expr, [$($A:literal),*] $(, $MODE:ty)?),)+
    ]) => {
        /// GPIO
        pub mod $gpiox {
            use crate::pac::{$GPIOX};
            // use crate::pac::{$GPIOX, RCC};
            // use crate::rcc::{Enable, Reset};

            /// GPIO parts
            pub struct Parts {
                $(
                    /// Pin
                    pub $pxi: $PXi $(<$MODE>)?,
                )+
            }

            impl super::GpioExt for $GPIOX {
                type Parts = Parts;

                fn split(self) -> Parts {
                    // TODO: Fix this!
                    // unsafe {
                        // NOTE(unsafe) this reference will only be used for atomic writes with no side effects.
                        // let rcc = &(*RCC::ptr());

                        // // Enable clock.
                        // $GPIOX::enable(rcc);
                        // $GPIOX::reset(rcc);
                    // }
                    Parts {
                        $(
                            $pxi: $PXi::new(),
                        )+
                    }
                }
            }

            #[doc="Common type for "]
            #[doc=stringify!($GPIOX)]
            #[doc=" related pins"]
            pub type $PXn<MODE> = super::PartiallyErasedPin<MODE>;

            $(
                #[doc=stringify!($PXi)]
                #[doc=" pin"]
                pub type $PXi<MODE = super::Input> = super::Pin<$i, MODE>;

                $(
                    impl<MODE> super::marker::IntoAf<$A> for $PXi<MODE> { }
                )*
            )+

        }

        pub use $gpiox::{ $($PXi,)+ };
    }
}

gpio!(GPIO, gpio, P0, '0', PAn, [
    P0_0: (p0_0, 0, [1, 2, 7]),
    P0_1: (p0_1, 1, [1, 2, 7]),
    P0_2: (p0_2, 2, [1, 2, 3, 7]),
    P0_3: (p0_3, 3, [1, 2, 3, 7]),
    P0_4: (p0_4, 4, [5, 6, 7]),
    P0_5: (p0_5, 5, [1, 5]),
    P0_6: (p0_6, 6, [1, 2, 5]),
    P0_7: (p0_7, 7, [1, 2, 5]),
    P0_8: (p0_8, 8, [0, 1, 4, 7, 10]),
    P0_9: (p0_9, 9, [1, 4, 7]),
    P0_10: (p0_10, 10, [1, 7, 10]),
    P0_11: (p0_11, 11, [1, 7, 8, 10]),
]);

struct Gpio;
impl Gpio {
    const fn ptr() -> *const crate::pac::gpio::RegisterBlock {
        crate::pac::GPIO::ptr()
    }
}
