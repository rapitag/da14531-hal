// use crate::{
//     gpio::{self, AfAdc},
//     pac::GPADC,
// };

// #[derive(Clone, Copy)]
// #[repr(u8)]
// pub enum InputPositive {
//     Adc0 = 0,
//     Adc1,
//     Adc2,
//     Adc3,
//     Temp,
//     VBatHigh,
//     VBatLow,
//     Vddd,
// }

// #[derive(Clone, Copy)]
// #[repr(u8)]
// pub enum InputNegative {
//     Adc0 = 0,
//     Adc1,
//     Adc2,
//     Adc3,
// }

// pub struct AdcInputTemp;

// pub struct AdcInputVbatHigh;
// pub struct AdcInputVbatLow;

// pub struct AdcInputVddd;

// pub trait AdcInputPositive<ADC>
// where
//     Self: embedded_hal::adc::Channel<ADC>,
// {
// }
// pub trait AdcInputNegative<ADC>
// where
//     Self: embedded_hal::adc::Channel<ADC>,
// {
// }

// macro_rules! adc_pin_entry {
//     (PN: $pin:ty => ($adc:ident, $chan:expr)) => {
//         impl embedded_hal::adc::Channel<crate::pac::$adc> for $pin {
//             type ID = u8;
//             fn channel() -> u8 {
//                 $chan
//             }
//         }

//         impl AdcInputPositive<crate::pac::$adc> for $pin {}
//         impl AdcInputNegative<crate::pac::$adc> for $pin {}
//     };
//     (P: $pin:ty => ($adc:ident, $chan:expr)) => {
//         impl embedded_hal::adc::Channel<crate::pac::$adc> for $pin {
//             type ID = u8;
//             fn channel() -> u8 {
//                 $chan
//             }
//         }

//         impl AdcInputPositive<crate::pac::$adc> for $pin {}
//     };
// }

// macro_rules! adc_pins {
//     ($($mode:ident: $pin:ty => ($adc:ident, $chan:expr)),+ $(,)*) => {
//         $(
//             adc_pin_entry!($mode: $pin => ($adc, $chan));
//         )+
//     };
// }

// pub mod config {

//     #[derive(Clone, Copy, PartialEq, Eq, Debug)]
//     #[repr(u8)]
//     pub enum InputMode {
//         Differential = 0,
//         SingleEnded,
//     }

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

//     #[derive(Clone, Copy, PartialEq, Eq, Debug)]
//     #[repr(u8)]
//     pub enum Continuous {
//         Single = 0,
//         Continuous,
//     }

//     #[derive(Clone, Copy, PartialEq, Eq, Debug)]
//     #[repr(u8)]
//     pub enum InputAttenuation {
//         None = 0,
//         X2,
//         X3,
//         X4,
//     }

//     #[derive(Copy, Clone, PartialEq, Eq, Debug)]
//     pub struct AdcConfig {
//         pub(crate) mode: InputMode,
//         pub(crate) chopper: ChopperMode,
//         pub(crate) mute: Mute,
//         pub(crate) attenuation: InputAttenuation,
//         pub(crate) continuous: Continuous,
//         pub(crate) vddd: u32
//     }
// }

// pub struct Adc {
//     gpadc: GPADC,
// }

// impl Adc {
//     pub fn new(gpadc: GPADC) -> Self {
//         Self { gpadc }
//     }

//     pub fn start(&mut self, trim_val: u16) {
//         self.stop();

//         self.gpadc.gp_adc_trim_reg.modify(|_, w| unsafe {
//             w.bits(trim_val);
//             w.gp_adc_ldo_level().bits(0x4)
//         });

//         // Disable ADC IRQ
//         // self.gpadc.gp_adc_ctrl_reg.modify().gp_adc_mint()

//         self.gpadc
//             .gp_adc_ctrl_reg
//             .modify(|_, w| w.gp_adc_en().set_bit());
//     }

//     pub fn configure_adc(
//         &mut self,
//         input_mode: config::InputMode,
//         input_positive: InputPositive,
//         input_negative: Option<InputNegative>,
//         continuous: bool,
//         interval: u8,
//         temp_cal_val: u16,
//     ) {
//         self.set_input_mode(input_mode);

//         self.gpadc.gp_adc_sel_reg.modify(|_, w| unsafe {
//             w.gp_adc_sel_p().bits(input_positive as u8);
//             if let InputMode::Differential = input_mode {
//                 if let Some(input_negative) = input_negative {
//                     w.gp_adc_sel_n().bits(input_negative as u8);
//                 } else {
//                     panic!("input_negative needs to be set in differential mode!");
//                 }
//             }
//             w
//         });

//         self.gpadc
//             .gp_adc_ctrl_reg
//             .modify(|_, w| w.gp_adc_cont().bit(continuous));

//         self.gpadc
//             .gp_adc_ctrl3_reg
//             .modify(|_, w| unsafe { w.gp_adc_interval().bits(interval) });

//         if let InputPositive::Temp = input_positive {
//             let cal_val = if temp_cal_val != 0 {
//                 temp_cal_val
//             } else {
//                 // OTP calibration value is not present - use typical value
//                 // instead (= 473*64)
//                 30272
//             };
//             self.temp_sensor_enable();
//             self.gpadc
//                 .gp_adc_ctrl_reg
//                 .modify(|_, w| w.gp_adc_chop().set_bit());
//             self.set_input_mode(InputMode::SingleEnded);

//             self.gpadc.gp_adc_ctrl2_reg.modify(|_, w| {
//                 w.gp_adc_i20u().set_bit();
//                 unsafe { w.gp_adc_store_del().bits(0) }
//             });

//             self.set_sample_time(15);
//             self.set_oversampling(6);
//         } else {
//         }
//     }

//     fn temp_sensor_enable(&mut self) {
//         self.gpadc
//             .gp_adc_ctrl_reg
//             .modify(|_, w| w.die_temp_en().set_bit());

//         // Guideline from the Analog IC Team: Wait for 25us to let the temperature
//         // sensor settle just after enabling it
//         // 25us/16MHz = 400 cylces
//         crate::cm::asm::delay(400);
//     }

//     fn set_input_mode(&mut self, input_mode: InputMode) {
//         self.gpadc
//             .gp_adc_ctrl_reg
//             .modify(|_, w| w.gp_adc_se().bit(input_mode as u8 != 0));
//     }

//     fn set_sample_time(&mut self, sample_time: u8) {
//         assert!(sample_time <= 15);

//         self.gpadc
//             .gp_adc_ctrl2_reg
//             .modify(|_, w| unsafe { w.gp_adc_smpl_time().bits(sample_time) })
//     }

//     fn set_oversampling(&mut self, mode: u8) {
//         assert!(mode <= 7);

//         self.gpadc
//             .gp_adc_ctrl2_reg
//             .modify(|_, w| unsafe { w.gp_adc_conv_nrs().bits(mode) })
//     }

//     pub fn stop(&mut self) {
//         self.gpadc.gp_adc_ctrl_reg.reset();
//         self.gpadc.gp_adc_ctrl2_reg.reset();
//         self.gpadc.gp_adc_ctrl3_reg.reset();
//     }
// }

// adc_pins!(
//     PN: gpio::p0::P0_01<AfAdc> => (GPADC, 0),
//     PN: gpio::p0::P0_02<AfAdc> => (GPADC, 1),
//     PN: gpio::p0::P0_06<AfAdc> => (GPADC, 2),
//     PN: gpio::p0::P0_07<AfAdc> => (GPADC, 3),
//     P: AdcInputTemp => (GPADC, 4),
//     P: AdcInputVbatHigh => (GPADC, 5),
//     P: AdcInputVbatLow => (GPADC, 6),
//     P: AdcInputVddd => (GPADC, 7)
// );
