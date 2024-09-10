use core::marker::PhantomData;

use void::Void;

use crate::{
    hal::digital::v2::{InputPin, OutputPin, PinState, StatefulOutputPin},
    pac::{gpio, GPIO as P0},
};

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

/// Alternate function mode (type state)
pub struct AlternateFunction<const PID: u8, const PUPD: u8>;

macro_rules! alternate_functions {
    (
            $($AF:ident: ($pid:literal, $pupd:literal)),+
    ) => {
        $(
            paste::paste! {
                pub type [<Af $AF>] = AlternateFunction<$pid, $pupd>;
            }
        )+
    };
}

alternate_functions! {
    Uart1Rx:    ( 1, 0b01),
    Uart1Tx:    ( 2, 0b01),
    Uart2Rx:    ( 3, 0b01),
    Uart2Tx:    ( 4, 0b01),
    SysClk:     ( 5, 0b01),
    LpClk:      ( 6, 0b01),
    I2cScl:     ( 9, 0b00),
    I2cSda:     (10, 0b00),
    Pwm5:       (11, 0b11),
    Pwm6:       (12, 0b11),
    Pwm7:       (13, 0b11),
    Adc:        (15, 0b00),
    Pwm0:       (16, 0b11),
    Pwm1:       (17, 0b11),
    BleDiag:    (18, 0b01),
    Uart1Ctsn:  (19, 0b01),
    Uart1Rtsn:  (20, 0b01),
    Pwm2:       (23, 0b11),
    Pwm3:       (24, 0b11),
    Pwm4:       (25, 0b11),
    SpiDi:      (26, 0b00),
    SpiDo:      (27, 0b11),
    SpiClk:     (28, 0b11),
    SpiEn1:      (29, 0b11),
    SpiEn2:      (30, 0b11)
}

// ===============================================================
// Implement Generic Pins for this port, which allows you to use
// other peripherals without having to be completely rust-generic
// across all of the possible pins
// ===============================================================
/// Generic $PX pin
pub struct Pin<MODE> {
    pin: u8,
    _mode: PhantomData<MODE>,
}

impl<MODE> Pin<MODE> {
    fn new(pin: u8) -> Self {
        Self {
            pin,
            _mode: PhantomData,
        }
    }

    #[inline]
    pub fn pin(&self) -> u8 {
        self.pin
    }

    fn block(&self) -> &gpio::RegisterBlock {
        unsafe { &*P0::ptr() }
    }

    // ToDo: Port this section!
    pub(crate) fn pin_mode(&self) -> &gpio::P0_MODE_REG {
        &self.block().p0_mode_reg[self.pin as usize]
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
            pin: self.pin,
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
            pin: self.pin,
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
            pin: self.pin,
        }
    }

    /// Convert the pin to be a push-pull output with normal drive.
    pub fn into_output(self, initial_output: PinState) -> Pin<Output> {
        let mut pin = Pin {
            _mode: PhantomData,
            pin: self.pin,
        };

        match initial_output {
            PinState::Low => pin.set_low().unwrap(),
            PinState::High => pin.set_high().unwrap(),
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

    pub fn into_alternate<const PID: u8, const PUPD: u8>(
        self,
    ) -> Pin<AlternateFunction<PID, PUPD>> {
        self.pin_mode()
            .modify(|_, w| unsafe { w.pupd().bits(PUPD).pid().bits(PID) });

        Pin {
            _mode: PhantomData,
            pin: self.pin,
        }
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
            pin: self.pin,
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
        $PX:ident, $pxsvd:ident, $px:ident, [
            $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty),)+
        ]
    ) => {
        /// GPIO
        pub mod $px {
            use super::{
                Pin,

                Floating,
                Disconnected,
                Input,
                PinState,
                Output,
                PullDown,
                PullUp,
                AlternateFunction,

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
                                w.pupd().bits(0b00)
                                .pid().bits(0)
                            }
                        });

                        $PXi {
                            _mode: PhantomData,
                        }
                    }
                    pub fn into_pulldown_input(self) -> $PXi<Input<PullDown>> {
                        unsafe { &(*$PX::ptr()).p0_mode_reg[$i] }.write(|w| {
                            unsafe {
                                w.pupd().bits(0b01)
                                .pid().bits(0)
                            }
                        });

                        $PXi {
                            _mode: PhantomData,
                        }
                    }
                    pub fn into_pullup_input(self) -> $PXi<Input<PullUp>> {
                        unsafe { &(*$PX::ptr()).p0_mode_reg[$i] }.write(|w| {
                            unsafe {
                                w.pupd().bits(0b10)
                                .pid().bits(0)
                            }
                        });

                        $PXi {
                            _mode: PhantomData,
                        }
                    }

                    /// Convert the pin to bepin a push-pull output with normal drive
                    pub fn into_output(self, initial_output: PinState)
                        -> $PXi<Output>
                    {
                        let mut pin = $PXi {
                            _mode: PhantomData,
                        };

                        match initial_output {
                            PinState::Low  => pin.set_low().unwrap(),
                            PinState::High => pin.set_high().unwrap(),
                        }

                        unsafe { &(*$PX::ptr()).p0_mode_reg[$i] }.write(|w| {
                            unsafe {
                                w.pupd().bits(0b11)
                                .pid().bits(0)
                            }
                        });

                        pin
                    }

                    pub fn into_alternate<const PID: u8, const PUPD: u8>(self) -> $PXi<AlternateFunction<PID, PUPD>> {
                        let pin = $PXi {
                            _mode: PhantomData,
                        };

                        unsafe { &(*$PX::ptr()).p0_mode_reg[$i] }.write(|w| {
                            unsafe {
                                w.pupd().bits(PUPD)
                                .pid().bits(PID)
                            }
                        });

                        pin
                    }

                    pub fn into_alternate_with_state<const PID: u8>(self, initial_output: PinState) -> $PXi<AlternateFunction<PID, 0b11>> {
                        let pin = $PXi {
                            _mode: PhantomData,
                        };

                        match initial_output {
                            PinState::Low  => {
                                unsafe { (*$PX::ptr()).p0_set_data_reg.write(|w| w.bits(1u16 << $i)); }
                            },
                            PinState::High => {
                                unsafe { (*$PX::ptr()).p0_reset_data_reg.write(|w| w.bits(1u16 << $i)); }
                            }
                        }

                        unsafe { &(*$PX::ptr()).p0_mode_reg[$i] }.write(|w| {
                            unsafe {
                                w.pupd().bits(0b11)
                                .pid().bits(PID)
                            }
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
                        Pin::new($i)
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
gpio!(P0, p0, p0, [
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
