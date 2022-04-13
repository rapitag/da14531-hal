use super::*;

// impl<const N: u8, const A: u8> Pin<N, Alternate<A, PushPull>> {
//     /// Turns pin alternate configuration pin into open drain
//     pub fn set_open_drain(self) -> Pin<N, Alternate<A, OpenDrain>> {
//         self.into_mode()
//     }
// }

impl<const N: u8, MODE: PinMode + marker::NotAlt, const A: u8, Otype> From<Pin<N, MODE>>
    for Pin<N, Alternate<A, Otype>>
where
    Alternate<A, Otype>: PinMode,
    Self: marker::IntoAf<A>,
{
    #[inline(always)]
    fn from(f: Pin<N, MODE>) -> Self {
        f.into_mode()
    }
}

// impl<const N: u8, const A: u8, const B: u8> From<Pin<N, Alternate<B, PushPull>>>
//     for Pin<N, Alternate<A, OpenDrain>>
// where
//     Self: marker::IntoAf<A>,
// {
//     #[inline(always)]
//     fn from(f: Pin<N, Alternate<B, PushPull>>) -> Self {
//         f.into_mode()
//     }
// }

impl<const N: u8, Otype> From<Pin<N, Output<Otype>>> for Pin<N, Input>
where
    Output<Otype>: PinMode,
{
    #[inline(always)]
    fn from(f: Pin<N, Output<Otype>>) -> Self {
        f.into_mode()
    }
}

impl<const N: u8> From<Pin<N, Analog>> for Pin<N, Input> {
    #[inline(always)]
    fn from(f: Pin<N, Analog>) -> Self {
        f.into_mode()
    }
}

impl<const N: u8, const A: u8, Otype, MODE> From<Pin<N, Alternate<A, Otype>>> for Pin<N, MODE>
where
    Alternate<A, Otype>: PinMode,
    MODE: PinMode + marker::NotAlt,
{
    #[inline(always)]
    fn from(f: Pin<N, Alternate<A, Otype>>) -> Self {
        f.into_mode()
    }
}

impl<const N: u8, Otype> From<Pin<N, Input>> for Pin<N, Output<Otype>>
where
    Output<Otype>: PinMode,
{
    #[inline(always)]
    fn from(f: Pin<N, Input>) -> Self {
        f.into_mode()
    }
}

impl<const N: u8, Otype> From<Pin<N, Analog>> for Pin<N, Output<Otype>>
where
    Output<Otype>: PinMode,
{
    #[inline(always)]
    fn from(f: Pin<N, Analog>) -> Self {
        f.into_mode()
    }
}

// impl<const N: u8> From<Pin<N, Output<PushPull>>> for Pin<N, Output<OpenDrain>> {
//     #[inline(always)]
//     fn from(f: Pin<N, Output<PushPull>>) -> Self {
//         f.into_mode()
//     }
// }

// impl<const N: u8> From<Pin<N, Output<OpenDrain>>> for Pin<N, Output<PushPull>> {
//     #[inline(always)]
//     fn from(f: Pin<N, Output<OpenDrain>>) -> Self {
//         f.into_mode()
//     }
// }

impl<const N: u8> From<Pin<N, Input>> for Pin<N, Analog> {
    #[inline(always)]
    fn from(f: Pin<N, Input>) -> Self {
        f.into_mode()
    }
}

impl<const N: u8, Otype> From<Pin<N, Output<Otype>>> for Pin<N, Analog>
where
    Output<Otype>: PinMode,
{
    #[inline(always)]
    fn from(f: Pin<N, Output<Otype>>) -> Self {
        f.into_mode()
    }
}

impl<const N: u8, MODE: PinMode> Pin<N, MODE> {
    // /// Configures the pin to operate alternate mode
    // pub fn into_alternate<const A: u8>(self) -> Pin<N, Alternate<A, PushPull>>
    // where
    //     Self: marker::IntoAf<A>,
    // {
    //     self.into_mode()
    // }

    // /// Configures the pin to operate in alternate open drain mode
    // #[allow(path_statements)]
    // pub fn into_alternate_open_drain<const A: u8>(self) -> Pin<N, Alternate<A, OpenDrain>>
    // where
    //     Self: marker::IntoAf<A>,
    // {
    //     self.into_mode()
    // }

    /// Configures the pin to operate as a input pin
    pub fn into_input(self) -> Pin<N, Input> {
        self.into_mode()
    }

    /// Configures the pin to operate as a floating input pin
    pub fn into_floating_input(self) -> Pin<N, Input> {
        self.into_mode().internal_resistor(Pull::None)
    }

    /// Configures the pin to operate as a pulled down input pin
    pub fn into_pull_down_input(self) -> Pin<N, Input> {
        self.into_mode().internal_resistor(Pull::Down)
    }

    /// Configures the pin to operate as a pulled up input pin
    pub fn into_pull_up_input(self) -> Pin<N, Input> {
        self.into_mode().internal_resistor(Pull::Up)
    }

    /// Configures the pin to operate as a pulled up input pin
    pub fn into_output(self) -> Pin<N, Output> {
        self.into_mode().internal_resistor(Pull::None)
    }

    // /// Configures the pin to operate as an open drain output pin
    // /// Initial state will be low.
    // pub fn into_open_drain_output(self) -> Pin<N, Output<OpenDrain>> {
    //     self.into_mode()
    // }

    // /// Configures the pin to operate as an open-drain output pin.
    // /// `initial_state` specifies whether the pin should be initially high or low.
    // pub fn into_open_drain_output_in_state(
    //     mut self,
    //     initial_state: PinState,
    // ) -> Pin<N, Output<OpenDrain>> {
    //     self._set_state(initial_state);
    //     self.into_mode()
    // }

    // /// Configures the pin to operate as an push pull output pin
    // /// Initial state will be low.
    // pub fn into_push_pull_output(mut self) -> Pin<N, Output<PushPull>> {
    //     self._set_low();
    //     self.into_mode()
    // }

    // /// Configures the pin to operate as an push-pull output pin.
    // /// `initial_state` specifies whether the pin should be initially high or low.
    // pub fn into_push_pull_output_in_state(
    //     mut self,
    //     initial_state: PinState,
    // ) -> Pin<N, Output<PushPull>> {
    //     self._set_state(initial_state);
    //     self.into_mode()
    // }

    // /// Configures the pin to operate as an analog input pin
    // pub fn into_analog(self) -> Pin<N, Analog> {
    //     self.into_mode()
    // }

    // TODO: Fix
    // /// Configures the pin as a pin that can change between input
    // /// and output without changing the type. It starts out
    // /// as a floating input
    // pub fn into_dynamic(self) -> DynamicPin<N> {
    //     self.into_floating_input();
    //     DynamicPin::new(Dynamic::InputFloating)
    // }

    /// Puts `self` into mode `M`.
    ///
    /// This violates the type state constraints from `MODE`, so callers must
    /// ensure they use this properly.
    #[inline(always)]
    pub(super) fn mode<M: PinMode>(&mut self) {
        // let offset = 2 * N;
        // unsafe {
        // if MODE::OTYPER != M::OTYPER {
        //     if let Some(otyper) = M::OTYPER {
        //         (*Gpio::ptr())
        //             .otyper
        //             .modify(|r, w| w.bits(r.bits() & !(0b1 << N) | (otyper << N)));
        //     }
        // }

        // if MODE::AFR != M::AFR {
        //     if let Some(afr) = M::AFR {
        //         if N < 8 {
        //             let offset2 = 4 * { N };
        //             (*Gpio::ptr()).afrl.modify(|r, w| {
        //                 w.bits((r.bits() & !(0b1111 << offset2)) | (afr << offset2))
        //             });
        //         } else {
        //             let offset2 = 4 * { N - 8 };
        //             (*Gpio::ptr()).afrh.modify(|r, w| {
        //                 w.bits((r.bits() & !(0b1111 << offset2)) | (afr << offset2))
        //             });
        //         }
        //     }
        // }

        // if MODE::MODER != M::MODER {
        //     (*Gpio::ptr())
        //         .moder
        //         .modify(|r, w| w.bits((r.bits() & !(0b11 << offset)) | (M::MODER << offset)));
        // }

        //         (*Gpio::ptr())
        //             .
        //             .modify(|r, w| w.bits((r.bits() & !(0b11 << offset)) | (M::MODER << offset)));
        //     }
        // }

        if MODE::PUPD != M::PUPD {
            let value = M::PUPD as u8;
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
    }

    #[inline(always)]
    /// Converts pin into specified mode
    pub fn into_mode<M: PinMode>(mut self) -> Pin<N, M> {
        self.mode::<M>();
        Pin::new()
    }
}

impl<const N: u8, MODE> Pin<N, MODE>
where
    MODE: PinMode,
{
    fn with_mode<M, F, R>(&mut self, f: F) -> R
    where
        M: PinMode,
        F: FnOnce(&mut Pin<N, M>) -> R,
    {
        self.mode::<M>();

        // This will reset the pin back to the original mode when dropped.
        // (so either when `with_mode` returns or when `f` unwinds)
        let _resetti = ResetMode { pin: self };

        let mut witness = Pin::new();

        f(&mut witness)
    }

    /// Temporarily configures this pin as a input.
    ///
    /// The closure `f` is called with the reconfigured pin. After it returns,
    /// the pin will be configured back.
    pub fn with_input<R>(&mut self, f: impl FnOnce(&mut Pin<N, Input>) -> R) -> R {
        self.with_mode(f)
    }

    /// Temporarily configures this pin as an analog pin.
    ///
    /// The closure `f` is called with the reconfigured pin. After it returns,
    /// the pin will be configured back.
    pub fn with_analog<R>(&mut self, f: impl FnOnce(&mut Pin<N, Analog>) -> R) -> R {
        self.with_mode(f)
    }

    /// Temporarily configures this pin as an open drain output.
    ///
    /// The closure `f` is called with the reconfigured pin. After it returns,
    /// the pin will be configured back.
    /// The value of the pin after conversion is undefined. If you
    /// want to control it, use `with_open_drain_output_in_state`
    // pub fn with_open_drain_output<R>(
    //     &mut self,
    //     f: impl FnOnce(&mut Pin<N, Output<OpenDrain>>) -> R,
    // ) -> R {
    //     self.with_mode(f)
    // }

    /// Temporarily configures this pin as an open drain output .
    ///
    /// The closure `f` is called with the reconfigured pin. After it returns,
    /// the pin will be configured back.
    /// Note that the new state is set slightly before conversion
    /// happens. This can cause a short output glitch if switching
    /// between output modes
    // pub fn with_open_drain_output_in_state<R>(
    //     &mut self,
    //     state: PinState,
    //     f: impl FnOnce(&mut Pin<N, Output<OpenDrain>>) -> R,
    // ) -> R {
    //     self._set_state(state);
    //     self.with_mode(f)
    // }

    /// Temporarily configures this pin as a push-pull output.
    ///
    /// The closure `f` is called with the reconfigured pin. After it returns,
    /// the pin will be configured back.
    /// The value of the pin after conversion is undefined. If you
    /// want to control it, use `with_push_pull_output_in_state`
    pub fn with_push_pull_output<R>(
        &mut self,
        f: impl FnOnce(&mut Pin<N, Output<PushPull>>) -> R,
    ) -> R {
        self.with_mode(f)
    }

    /// Temporarily configures this pin as a push-pull output.
    ///
    /// The closure `f` is called with the reconfigured pin. After it returns,
    /// the pin will be configured back.
    /// Note that the new state is set slightly before conversion
    /// happens. This can cause a short output glitch if switching
    /// between output modes
    pub fn with_push_pull_output_in_state<R>(
        &mut self,
        state: PinState,
        f: impl FnOnce(&mut Pin<N, Output<PushPull>>) -> R,
    ) -> R {
        self._set_state(state);
        self.with_mode(f)
    }
}

struct ResetMode<'a, const N: u8, ORIG: PinMode> {
    pin: &'a mut Pin<N, ORIG>,
}

// TODO: Fix!
// impl<'a, const N: u8, ORIG: PinMode> Drop for ResetMode<'a, N, ORIG> {
//     fn drop(&mut self) {
//         self.pin.mode::<ORIG>();
//     }
// }

/// Marker trait for valid pin modes (type state).
///
/// It can not be implemented by outside types.
pub trait PinMode: crate::Sealed {
    // These constants are used to implement the pin configuration code.
    // They are not part of public API.

    #[doc(hidden)]
    const PUPD: u32 = u32::MAX;
    // #[doc(hidden)]
    // const MODER: u32 = u32::MAX;
    // #[doc(hidden)]
    // const OTYPER: Option<u32> = None;
    // #[doc(hidden)]
    // const AFR: Option<u32> = None;
}

impl crate::Sealed for Input {}
impl PinMode for Input {
    const PUPD: u32 = 0b00;
}

impl crate::Sealed for Analog {}
impl PinMode for Analog {
    const PUPD: u32 = 0b11;
}

// impl<Otype> crate::Sealed for Output<Otype> {}
// impl PinMode for Output<OpenDrain> {
//     const PUPD: u32 = 0b01;
//     const OTYPER: Option<u32> = Some(0b1);
// }

impl crate::Sealed for Output<PushPull> {}
impl PinMode for Output<PushPull> {
    const PUPD: u32 = 0b11;
    // const OTYPER: Option<u32> = Some(0b0);
}

// impl<const A: u8, Otype> crate::Sealed for Alternate<A, Otype> {}
// impl<const A: u8> PinMode for Alternate<A, OpenDrain> {
//     const MODER: u32 = 0b10;
//     const OTYPER: Option<u32> = Some(0b1);
//     const AFR: Option<u32> = Some(A as _);
// }

// impl<const A: u8> PinMode for Alternate<A, PushPull> {
//     const MODER: u32 = 0b10;
//     const OTYPER: Option<u32> = Some(0b0);
//     const AFR: Option<u32> = Some(A as _);
// }
