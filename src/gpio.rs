use core::marker::PhantomData;

/// Disconnected pin in input mode (type state, reset value).
pub struct Disconnected;

/// Input mode (type state).
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Floating input (type state).
pub struct Floating;
/// Pulled down input (type state).
pub struct PullDown;
/// Pulled up input (type state).
pub struct PullUp;

/// Output mode (type state).
pub struct Output;

/// Represents a digital input or output level.
#[derive(Debug, Eq, PartialEq)]
pub enum Level {
    Low,
    High,
}

/// A GPIO port with up to 16 pins.
#[derive(Debug, Eq, PartialEq)]
pub enum Port {
    /// Port 0
    Port0,
}

// ===============================================================
// Implement Generic Pins for this port, which allows you to use
// other peripherals without having to be completely rust-generic
// across all of the possible pins
// ===============================================================
/// Generic $PX pin
pub struct Pin<MODE> {
    /// 00AB BBBB
    /// A: Port
    /// B: Pin
    pin_port: u8,
    _mode: PhantomData<MODE>,
}

use crate::pac::{gpio, GPIO as P0};

use crate::hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};
use void::Void;

impl<MODE> Pin<MODE> {
    fn new(port: Port, pin: u8) -> Self {
        let port_bits = match port {
            Port::Port0 => 0x00,
        };
        Self {
            pin_port: pin | port_bits,
            _mode: PhantomData,
        }
    }

    pub unsafe fn from_psel_bits(psel_bits: u32) -> Self {
        Self {
            pin_port: psel_bits as u8,
            _mode: PhantomData,
        }
    }

    #[inline]
    pub fn pin(&self) -> u8 {
        {
            self.pin_port
        }
    }

    #[inline]
    pub fn port(&self) -> Port {
        Port::Port0
    }

    #[inline]
    pub fn psel_bits(&self) -> u32 {
        self.pin_port as u32
    }

    fn block(&self) -> &gpio::RegisterBlock {
        let ptr = match self.port() {
            Port::Port0 => P0::ptr(),
        };

        unsafe { &*ptr }
    }

    // ToDo: Port this section!
    pub(crate) fn pin_mode(&self) -> &gpio::P0_MODE_REG {
        &self.block().p0_mode_reg[self.pin_port as usize]
    }

    /// Convert the pin to be a floating input
    pub fn into_floating_input(self) -> Pin<Input<Floating>> {
        self.pin_mode().write(|w| {
            unsafe {
                w.pupd().bits(0b00);
                w.pid().bits(0);
            }
            w
        });

        Pin {
            _mode: PhantomData,
            pin_port: self.pin_port,
        }
    }
    pub fn into_pullup_input(self) -> Pin<Input<PullUp>> {
        self.pin_mode().write(|w| {
            unsafe {
                w.pupd().bits(0b01);
                w.pid().bits(0);
            }
            w
        });

        Pin {
            _mode: PhantomData,
            pin_port: self.pin_port,
        }
    }
    pub fn into_pulldown_input(self) -> Pin<Input<PullDown>> {
        self.pin_mode().write(|w| {
            unsafe {
                w.pupd().bits(0b10);
                w.pid().bits(0);
            }
            w
        });

        Pin {
            _mode: PhantomData,
            pin_port: self.pin_port,
        }
    }

    /// Convert the pin to be a push-pull output with normal drive.
    pub fn into_output(self, initial_output: Level) -> Pin<Output> {
        let mut pin = Pin {
            _mode: PhantomData,
            pin_port: self.pin_port,
        };

        match initial_output {
            Level::Low => pin.set_low().unwrap(),
            Level::High => pin.set_high().unwrap(),
        }

        self.pin_mode().write(|w| {
            unsafe {
                w.pupd().bits(0b11);
                w.pid().bits(0);
            }
            w
        });

        pin
    }

    /// Disconnects the pin.
    ///
    /// In disconnected mode the pin cannot be used as input or output.
    /// It is primarily useful to reduce power usage.
    pub fn into_disconnected(self) -> Pin<Disconnected> {
        // Reset value is disconnected.
        self.pin_mode().reset();

        Pin {
            _mode: PhantomData,
            pin_port: self.pin_port,
        }
    }
}

impl<MODE> InputPin for Pin<Input<MODE>> {
    type Error = Void;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.block().p0_data_reg.read().bits() & (1 << self.pin()) == 0)
    }
}

impl OutputPin for Pin<Output> {
    type Error = Void;

    /// Set the output as high.
    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.block()
                .p0_set_data_reg
                .write(|w| w.bits(1u16 << self.pin()));
        }
        Ok(())
    }

    /// Set the output as low.
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.block()
                .p0_reset_data_reg
                .write(|w| w.bits(1u16 << self.pin()));
        }
        Ok(())
    }
}

impl StatefulOutputPin for Pin<Output> {
    /// Is the output pin set as high?
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.is_set_low().map(|v| !v)
    }

    /// Is the output pin set as low?
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        // NOTE(unsafe) atomic read with no side effects - TODO(AJM) verify?
        // TODO - I wish I could do something like `.pins$i()`...
        Ok(self.block().p0_data_reg.read().bits() & (1 << self.pin()) == 0)
    }
}

macro_rules! gpio {
    (
        $PX:ident, $pxsvd:ident, $px:ident, $port_value:expr, [
            $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty),)+
        ]
    ) => {
        /// GPIO
        pub mod $px {
            use super::{
                Pin,
                Port,

                Floating,
                Disconnected,
                Input,
                Level,
                Output,
                PullDown,
                PullUp,

                PhantomData,
                $PX
            };

            use crate::hal::digital::v2::{OutputPin, StatefulOutputPin, InputPin};
            use void::Void;


            // ===============================================================
            // This chunk allows you to obtain an da14531 gpio from the
            // upstream gpio definitions by defining a trait
            // ===============================================================
            /// GPIO parts
            pub struct Parts {
                $(
                    /// Pin
                    pub $pxi: $PXi<$MODE>,
                )+
            }

            impl Parts {
                pub fn new(_gpio: $PX) -> Self {
                    Self {
                        $(
                            $pxi: $PXi {
                                _mode: PhantomData,
                            },
                        )+
                    }
                }
            }

            // ===============================================================
            // Implement each of the typed pins usable through the da14531-hal
            // defined interface
            // ===============================================================
            $(
                pub struct $PXi<MODE> {
                    _mode: PhantomData<MODE>,
                }


                impl<MODE> $PXi<MODE> {
                    /// Convert the pin to be a floating input
                    pub fn into_floating_input(self) -> $PXi<Input<Floating>> {
                        unsafe { &(*$PX::ptr()).p0_mode_reg[$i] }.write(|w| {
                            unsafe {
                                w.pupd().bits(0b00);
                                w.pid().bits(0);
                            }
                            w
                        });

                        $PXi {
                            _mode: PhantomData,
                        }
                    }
                    pub fn into_pulldown_input(self) -> $PXi<Input<PullDown>> {
                        unsafe { &(*$PX::ptr()).p0_mode_reg[$i] }.write(|w| {
                            unsafe {
                                w.pupd().bits(0b01);
                                w.pid().bits(0);
                            }
                            w
                        });

                        $PXi {
                            _mode: PhantomData,
                        }
                    }
                    pub fn into_pullup_input(self) -> $PXi<Input<PullUp>> {
                        unsafe { &(*$PX::ptr()).p0_mode_reg[$i] }.write(|w| {
                            unsafe {
                                w.pupd().bits(0b10);
                                w.pid().bits(0);
                            }
                            w
                        });

                        $PXi {
                            _mode: PhantomData,
                        }
                    }

                    /// Convert the pin to bepin a push-pull output with normal drive
                    pub fn into_output(self, initial_output: Level)
                        -> $PXi<Output>
                    {
                        let mut pin = $PXi {
                            _mode: PhantomData,
                        };

                        match initial_output {
                            Level::Low  => pin.set_low().unwrap(),
                            Level::High => pin.set_high().unwrap(),
                        }

                        unsafe { &(*$PX::ptr()).p0_mode_reg[$i] }.write(|w| {
                            unsafe {
                                w.pupd().bits(0b11);
                                w.pid().bits(0);
                            }
                            w
                        });

                        pin
                    }


                    /// Disconnects the pin.
                    ///
                    /// In disconnected mode the pin cannot be used as input or output.
                    /// It is primarily useful to reduce power usage.
                    pub fn into_disconnected(self) -> $PXi<Disconnected> {
                        // Reset value is disconnected.
                        unsafe { &(*$PX::ptr()).p0_mode_reg[$i] }.reset();

                        $PXi {
                            _mode: PhantomData,
                        }
                    }

                    /// Degrade to a generic pin struct, which can be used with peripherals
                    pub fn degrade(self) -> Pin<MODE> {
                        Pin::new($port_value, $i)
                    }
                }

                impl<MODE> InputPin for $PXi<Input<MODE>> {
                    type Error = Void;

                    fn is_high(&self) -> Result<bool, Self::Error> {
                        self.is_low().map(|v| !v)
                    }

                    fn is_low(&self) -> Result<bool, Self::Error> {
                        Ok(unsafe { ((*$PX::ptr()).p0_data_reg.read().bits() & (1 << $i)) == 0 })
                    }
                }

                impl<MODE> From<$PXi<MODE>> for Pin<MODE> {
                    fn from(value: $PXi<MODE>) -> Self {
                        value.degrade()
                    }
                }

                impl OutputPin for $PXi<Output> {
                    type Error = Void;

                    /// Set the output as high
                    fn set_high(&mut self) -> Result<(), Self::Error> {
                        unsafe { (*$PX::ptr()).p0_set_data_reg.write(|w| w.bits(1u16 << $i)); }
                        Ok(())
                    }

                    /// Set the output as low
                    fn set_low(&mut self) -> Result<(), Self::Error> {
                        unsafe { (*$PX::ptr()).p0_reset_data_reg.write(|w| w.bits(1u16 << $i)); }
                        Ok(())
                    }
                }

                impl StatefulOutputPin for $PXi<Output> {
                    /// Is the output pin set as high?
                    fn is_set_high(&self) -> Result<bool, Self::Error> {
                        self.is_set_low().map(|v| !v)
                    }

                    /// Is the output pin set as low?
                    fn is_set_low(&self) -> Result<bool, Self::Error> {
                        // NOTE(unsafe) atomic read with no side effects - TODO(AJM) verify?
                        // TODO - I wish I could do something like `.pins$i()`...
                        Ok(unsafe { ((*$PX::ptr()).p0_data_reg.read().bits() & (1 << $i)) == 0 })
                    }
                }
            )+
        }
    }
}

// ===========================================================================
// Definition of all the items used by the macros above.
// ===========================================================================
gpio!(P0, p0, p0, Port::Port0, [
    P0_00: (p0_00,  0, Disconnected),
    P0_01: (p0_01,  1, Disconnected),
    P0_02: (p0_02,  2, Disconnected),
    P0_03: (p0_03,  3, Disconnected),
    P0_04: (p0_04,  4, Disconnected),
    P0_05: (p0_05,  5, Disconnected),
    P0_06: (p0_06,  6, Disconnected),
    P0_07: (p0_07,  7, Disconnected),
    P0_08: (p0_08,  8, Disconnected),
    P0_09: (p0_09,  9, Disconnected),
    P0_10: (p0_10, 10, Disconnected),
    P0_11: (p0_11, 11, Disconnected),
]);
