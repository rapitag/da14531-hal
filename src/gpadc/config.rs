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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Chopper {
    Off,
    On,
}

impl Default for Chopper {
    fn default() -> Self {
        Self::Off
    }
}

impl Into<bool> for Chopper {
    fn into(self) -> bool {
        match self {
            Chopper::Off => false,
            Chopper::On => true,
        }
    }
}

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Averaging {
    SamplesX1 = 0,
    SamplesX2,
    SamplesX4,
    SamplesX8,
    SamplesX16,
    SamplesX32,
    SamplesX128,
}

impl Default for Averaging {
    fn default() -> Self {
        Self::SamplesX1
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum SampleTime {
    Cycles1X8 = 0,
    Cycles2X8,
    Cycles3X8,
    Cycles4X8,
    Cycles5X8,
    Cycles6X8,
    Cycles7X8,
    Cycles8X8,
    Cycles9X8,
    Cycles10X8,
    Cycles11X8,
    Cycles12X8,
    Cycles13X8,
    Cycles14X8,
    Cycles15X8,
}

impl Default for SampleTime {
    fn default() -> Self {
        Self::Cycles1X8
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Shifter {
    Off,
    On,
}

impl Default for Shifter {
    fn default() -> Self {
        Self::Off
    }
}

impl Into<bool> for Shifter {
    fn into(self) -> bool {
        match self {
            Shifter::Off => false,
            Shifter::On => true,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct AdcConfig {
    pub(crate) mode: InputMode,
    pub(crate) chopper: Chopper,
    pub(crate) sample_time: SampleTime,
    pub(crate) averaging: Averaging,
    pub(crate) shifter: Shifter,
    // pub(crate) mute: Mute,
    pub(crate) attenuation: Attenuation,
    pub(crate) continuous: Continuous,
    pub(crate) channel_sel_pos: u8,
    pub(crate) channel_sel_neg: u8,
    pub(crate) enable_die_temp: bool, // pub(crate) vddd: u32
    pub(crate) adc_trim_val: u16,
}

impl Default for AdcConfig {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            chopper: Default::default(),
            sample_time: Default::default(),
            averaging: Default::default(),
            attenuation: Default::default(),
            continuous: Default::default(),
            shifter: Default::default(),
            channel_sel_pos: 0,
            channel_sel_neg: 0,
            enable_die_temp: false,
            adc_trim_val: 0,
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

    pub fn set_chopper_mode(mut self, chopper: Chopper) -> Self {
        self.chopper = chopper;
        self
    }

    pub fn set_sample_time(mut self, sample_time: SampleTime) -> Self {
        self.sample_time = sample_time;
        self
    }

    pub fn set_averaging(mut self, averaging: Averaging) -> Self {
        self.averaging = averaging;
        self
    }

    pub fn set_shifter(mut self, shifter: Shifter) -> Self {
        self.shifter = shifter;
        self
    }

    pub fn set_adc_trim_val(mut self, val: u16) -> Self {
        self.adc_trim_val = val;
        self
    }
}
