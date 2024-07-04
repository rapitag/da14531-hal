use crate::pac::GPADC;

pub mod config;

use config::{AdcConfig, Averaging, SampleTime};

/// Extension trait that constrains the `SYS_WDOG` peripheral
pub trait GpAdcExt {
    /// Constrains the `SYS_WDOG` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> GpAdc;
}

impl GpAdcExt for GPADC {
    fn constrain(self) -> GpAdc {
        GpAdc { gpadc: self }
    }
}

pub struct GpAdc {
    gpadc: GPADC,
}

impl GpAdc {
    pub fn calibrate_offset(&self, adc_trim_val: u16) {
        // Configure for calibration
        self.init(
            AdcConfig::default()
                .set_adc_trim_val(adc_trim_val)
                .set_sample_time(SampleTime::Cycles1X8)
                .set_averaging(Averaging::SamplesX128),
        );
        self.enable_20u_sink();

        // Try once to calibrate correctly

        self.gpadc
            .gp_adc_offp_reg
            .modify(|_, w| unsafe { w.gp_adc_offp().bits(0x200) });
        self.gpadc
            .gp_adc_offn_reg
            .modify(|_, w| unsafe { w.gp_adc_offn().bits(0x200) });
        self.gpadc
            .gp_adc_ctrl_reg
            .modify(|_, w| w.gp_adc_mute().set_bit().gp_adc_sign().clear_bit());

        self.start_conversion();
        self.wait_for_conversion();
        let adc_val = self.current_sample();
        let adc_off_p = (adc_val >> 6) - 0x200;

        self.gpadc
            .gp_adc_ctrl_reg
            .modify(|_, w| w.gp_adc_sign().set_bit());

        self.start_conversion();
        self.wait_for_conversion();
        let adc_val = self.current_sample();
        let adc_off_n = (adc_val >> 6) - 0x200;

        self.gpadc
            .gp_adc_offp_reg
            .modify(|_, w| unsafe { w.gp_adc_offp().bits(0x200 - 2 * adc_off_p) });

        self.gpadc
            .gp_adc_offn_reg
            .modify(|_, w| unsafe { w.gp_adc_offn().bits(0x200 - 2 * adc_off_n) });

        self.gpadc
            .gp_adc_ctrl_reg
            .modify(|_, w| w.gp_adc_sign().clear_bit());

        // Verify the calibration result

        self.start_conversion();
        self.wait_for_conversion();
        let adc_val = self.current_sample() >> 6;

        let diff = if adc_val > 0x200 {
            adc_val - 0x200
        } else {
            0x200 - adc_val
        };

        if diff >= 0x8 {
            // Calibration failed
        }
    }

    pub fn init(&self, adc_config: AdcConfig) {
        self.reset();

        // Set GP_ADC_LDO_LEVEL to the preferred level of 925mV
        self.gpadc
            .gp_adc_trim_reg
            .modify(|_, w| unsafe { w.bits(adc_config.adc_trim_val).gp_adc_ldo_level().bits(0x4) });

        self.configure(adc_config);

        self.enable();
    }

    pub fn configure(&self, adc_config: AdcConfig) {
        self.gpadc.gp_adc_ctrl_reg.modify(|_, w| {
            w.gp_adc_se()
                .bit(adc_config.mode.into())
                .gp_adc_cont()
                .bit(adc_config.continuous.into())
                .die_temp_en()
                .bit(adc_config.enable_die_temp)
                .gp_adc_chop()
                .bit(adc_config.chopper.into())
        });

        self.gpadc.gp_adc_sel_reg.modify(|_, w| unsafe {
            w.gp_adc_sel_p()
                .bits(adc_config.channel_sel_pos)
                .gp_adc_sel_n()
                .bits(adc_config.channel_sel_neg)
        });

        if adc_config.enable_die_temp {
            // Guideline from the Analog IC Team: Wait for 25us to let the temperature
            // sensor settle just after enabling it
            // 25us*16MHz = 400 cylces
            crate::cm::asm::delay(400);
        }

        self.gpadc.gp_adc_ctrl2_reg.modify(|_, w| unsafe {
            w.gp_adc_offs_sh_en()
                .bit(adc_config.shifter.into())
                .gp_adc_attn()
                .bits(adc_config.attenuation as u8)
                .gp_adc_conv_nrs()
                .bits(adc_config.averaging as u8)
                .gp_adc_smpl_time()
                .bits(adc_config.sample_time as u8)
        });
    }

    /// Enable ADC peripheral
    pub fn enable(&self) {
        self.gpadc
            .gp_adc_ctrl_reg
            .modify(|_, w| w.gp_adc_en().set_bit());

        let delay_cycles = 4 * self.gpadc.gp_adc_ctrl3_reg.read().gp_adc_en_del().bits() as u32;

        crate::cm::asm::delay(delay_cycles);
    }

    /// Disable ADC peripheral
    pub fn disable(&self) {
        self.gpadc
            .gp_adc_ctrl_reg
            .modify(|_, w| w.gp_adc_en().clear_bit());
    }

    /// Reset ADC peripheral
    pub fn reset(&self) {
        self.gpadc.gp_adc_ctrl_reg.reset();
        self.gpadc.gp_adc_ctrl2_reg.reset();
        self.gpadc.gp_adc_ctrl3_reg.reset();
        self.gpadc.gp_adc_sel_reg.reset();
        self.gpadc.gp_adc_trim_reg.reset();
    }

    /// Enable 2u constant current sink for LDO
    pub fn enable_20u_sink(&self) {
        self.gpadc
            .gp_adc_ctrl2_reg
            .modify(|_, w| w.gp_adc_i20u().set_bit());
    }

    /// Enable 2u constant current sink for LDO
    pub fn disable_20u_sink(&self) {
        self.gpadc
            .gp_adc_ctrl2_reg
            .modify(|_, w| w.gp_adc_i20u().clear_bit());
    }

    /// Enable 2u constant current sink for LDO
    pub fn enable_shifter(&self) {
        self.gpadc
            .gp_adc_ctrl2_reg
            .modify(|_, w| w.gp_adc_offs_sh_en().set_bit());
    }

    /// Enable 2u constant current sink for LDO
    pub fn disable_shifter(&self) {
        self.gpadc
            .gp_adc_ctrl2_reg
            .modify(|_, w| w.gp_adc_offs_sh_en().clear_bit());
    }

    /// Start ADC conversion
    pub fn start_conversion(&self) {
        self.gpadc
            .gp_adc_ctrl_reg
            .modify(|_, w| w.gp_adc_start().set_bit());
    }

    /// Wait for conversion to finish (in manual mode only)
    pub fn wait_for_conversion(&self) {
        while self
            .gpadc
            .gp_adc_ctrl_reg
            .read()
            .gp_adc_start()
            .bit_is_set()
        {}

        self.gpadc
            .gp_adc_clear_int_reg
            .write(|w| unsafe { w.gp_adc_clr_int().bits(1) })
    }

    /// Read current sample value from register
    pub fn current_sample(&self) -> u16 {
        self.gpadc.gp_adc_result_reg.read().gp_adc_val().bits()
    }

    pub fn correction_apply(&self, gain_error: i16, offset: i16, sample: u16) -> u16 {
        let res = (u16::MAX as f32 * sample as f32) / (u16::MAX as f32 + gain_error as f32);
        let res = res - (u16::MAX as f32 * offset as f32) / (u16::MAX as f32 + gain_error as f32);

        // Boundary check for lower limit
        if res > 2.0 * u16::MAX as f32 {
            return 0;
        }

        // Boundary check for upper limit
        if res > u16::MAX as f32 {
            return u16::MAX;
        }

        res as u16
    }

    pub fn has_shifter(&self) -> bool {
        self.gpadc.gp_adc_ctrl2_reg.read().gp_adc_offs_sh_en().bit()
    }

    pub fn convert_to_voltage(&self, sample: u16) -> f32 {
        // TODO: This need to be fixed
        // 1bit = 1.526e-5 (1/0xffff)
        // 0xffff => get_max_value() (calculated based on attenuation)
        
        // let attn = self.gpadc.gp_adc_ctrl2_reg.read().gp_adc_attn().bits() + 1;
        let factor = 3.6 / 0xffff as f32;

        sample as f32 * factor
    }
}
