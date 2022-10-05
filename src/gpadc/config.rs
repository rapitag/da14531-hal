use crate::{
    gpio::{self, AfAdc},
    hal::adc::Channel,
    pac::GPADC,
};

pub struct AdcInputTemp;

pub struct AdcInputVbatHigh;
pub struct AdcInputVbatLow;

pub struct AdcInputVddd;

pub trait AdcInputPositive
where
    Self: crate::hal::adc::Channel<GPADC>,
{
}
pub trait AdcInputNegative
where
    Self: crate::hal::adc::Channel<GPADC>,
{
}

macro_rules! adc_pin_entry {
    (PN: $pin:ty => $chan:expr) => {
        adc_pin_entry!(P: $pin => $chan);

        impl AdcInputNegative for $pin
        where
            Self: crate::hal::adc::Channel<crate::pac::GPADC, ID = u8> {}
    };
    (P: $pin:ty => $chan:expr) => {
        impl crate::hal::adc::Channel<crate::pac::GPADC> for $pin {
            type ID = u8;
            fn channel() -> u8 {
                $chan
            }
        }

        impl AdcInputPositive for $pin
        where
            Self: crate::hal::adc::Channel<crate::pac::GPADC, ID = u8> {}
    };
}

macro_rules! adc_pins {
    ($($mode:ident: $pin:ty => $chan:expr),+ $(,)*) => {
        $(
            adc_pin_entry!($mode: $pin => $chan);
        )+
    };
}

adc_pins!(
    PN: gpio::p0::P0_01<AfAdc> => 0,
    PN: gpio::p0::P0_02<AfAdc> => 1,
    PN: gpio::p0::P0_06<AfAdc> => 2,
    PN: gpio::p0::P0_07<AfAdc> => 3,
    P: AdcInputTemp => 4,
    P: AdcInputVbatHigh => 5,
    P: AdcInputVbatLow => 6,
    P: AdcInputVddd => 7
);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputMode {
    Differential,
    SingleEnded,
}

impl Default for InputMode {
    fn default() -> Self {
        Self::SingleEnded
    }
}

impl Into<bool> for InputMode {
    fn into(self) -> bool {
        match self {
            InputMode::Differential => false,
            InputMode::SingleEnded => true,
        }
    }
}

//     #[derive(Clone, Copy, PartialEq, Eq, Debug)]
//     #[repr(u8)]
//     pub enum ChopperMode {
//         Off = 0,
//         On,
//     }

//     #[derive(Clone, Copy, PartialEq, Eq, Debug)]
//     #[repr(u8)]
//     pub enum Mute {
//         Off = 0,
//         On,
//     }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Continuous {
    Single,
    Continuous,
}

impl Into<bool> for Continuous {
    fn into(self) -> bool {
        match self {
            Continuous::Single => false,
            Continuous::Continuous => true,
        }
    }
}

impl Default for Continuous {
    fn default() -> Self {
        Self::Single
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Attenuation {
    None = 0,
    X2,
    X3,
    X4,
}

impl Default for Attenuation {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct AdcConfig {
    pub(crate) mode: InputMode,
    // pub(crate) chopper: ChopperMode,
    // pub(crate) mute: Mute,
    pub(crate) attenuation: Attenuation,
    pub(crate) continuous: Continuous,
    pub(crate) channel_sel_pos: u8,
    pub(crate) channel_sel_neg: u8,
    pub(crate) enable_die_temp: bool, // pub(crate) vddd: u32
}

impl Default for AdcConfig {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            attenuation: Default::default(),
            continuous: Default::default(),
            channel_sel_pos: 0,
            channel_sel_neg: 0,
            enable_die_temp: false,
        }
    }
}

impl AdcConfig {
    pub fn set_channel_pos<P: Channel<GPADC, ID = u8> + AdcInputPositive>(
        mut self,
        _pin: P,
    ) -> Self {
        let channel = P::channel() as u8;
        self.channel_sel_pos = channel;

        // Enable internal temp sensor (for channel 4)
        if channel == 4 {
            self.enable_die_temp = true;
        }

        self
    }
    pub fn set_channel_neg<N: Channel<GPADC, ID = u8> + AdcInputNegative>(
        mut self,
        _pin: N,
    ) -> Self {
        self.mode = InputMode::Differential;
        self.channel_sel_neg |= N::channel() as u8;

        self
    }

    pub fn set_attenuation(mut self, attenuation: Attenuation) -> Self {
        self.attenuation = attenuation;
        self
    }
}
