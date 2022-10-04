use crate::pac::CRG_AON;

pub trait CrgAonExt {
    fn constrain(self) -> CrgAon;
}

impl CrgAonExt for CRG_AON {
    fn constrain(self) -> CrgAon {
        CrgAon { crg_aon: self }
    }
}

pub struct CrgAon {
    crg_aon: CRG_AON,
}

impl CrgAon {
    pub fn set_pad_latch_en(&mut self, state: bool) {
        self.crg_aon
            .pad_latch_reg
            .write(|w| w.pad_latch_en().bit(state));
    }
}

pub mod sleep {
    use crate::{crg_top::CrgTop, nvic::Nvic, pac::SCB, sys_wdog::SysWdog};

    use super::CrgAon;

    #[derive(PartialEq, Copy, Clone)]
    #[repr(u8)]
    pub enum RemapAddr {
        ToRom = 0,
        ToOtp = 1,
        ToRam1 = 2,
        ToRam3 = 3,
    }

    impl Default for RemapAddr {
        fn default() -> Self {
            Self::ToRom
        }
    }

    #[derive(Clone, Default)]
    pub struct SleepConfig {
        pin_mask: u8,
        ram1_on: bool,
        ram2_on: bool,
        ram3_on: bool,
        remap_addr: RemapAddr,
        pad_latch_en: bool,
    }

    impl SleepConfig {
        pub fn new(
            pin_mask: u8,
            ram1_on: bool,
            ram2_on: bool,
            ram3_on: bool,
            remap_addr: RemapAddr,
            pad_latch_en: bool,
        ) -> Self {
            Self {
                pin_mask,
                ram1_on,
                ram2_on,
                ram3_on,
                remap_addr,
                pad_latch_en,
            }
        }

        pub fn enable_pin<WP: WakeupPin>(mut self, _pin: WP) -> Self {
            self.pin_mask |= WP::mask();
            self
        }

        pub fn set_ram_power(mut self, ram1_on: bool, ram2_on: bool, ram3_on: bool) -> Self {
            self.ram1_on = ram1_on;
            self.ram2_on = ram2_on;
            self.ram3_on = ram3_on;
            self
        }

        pub fn set_remap_addr(mut self, remap_addr: RemapAddr) -> Self {
            self.remap_addr = remap_addr;
            self
        }

        pub fn set_pad_latch_en(mut self, pad_latch_en: bool) -> Self {
            self.pad_latch_en = pad_latch_en;
            self
        }
    }

    impl CrgAon {
        pub fn init_sleep(
            &mut self,
            nvic: &mut Nvic,
            crg_top: &mut CrgTop,
            sys_wdog: &mut SysWdog,
            scb: &mut SCB,
            sleep_config: &SleepConfig,
        ) {
            // assert!((remap_addr0 == RemapAddr::ToRam1) != ram1_on);
            // assert!((remap_addr0 == RemapAddr::ToRam3) != ram3_on);

            // Stop watchdog timer
            sys_wdog.freeze();

            // Disable Interrupts
            crate::cm::interrupt::disable();

            // Clear all pending interrupts
            nvic.clear_pending_interrupts();

            // // Store the debugger configuration
            // booter_val.dbg_cfg = GetBits16(SYS_CTRL_REG, DEBUGGER_ENABLE);

            // Debugger must be disabled before entering a sleep mode. Wait until debugger has been disabled.
            while crg_top.is_dbg_up() {}

            crg_top.disable_dbg();

            // Set the wake up pins
            self.crg_aon
                .hibern_ctrl_reg
                .modify(|_, w| unsafe { w.hibern_wkup_mask().bits(sleep_config.pin_mask) });

            // Check the output of the clockless wakeup XOR tree to determine the wake up polarity
            if !crg_top.clkless_wakeup_stat() {
                self.crg_aon.hibern_ctrl_reg.modify(|r, w| {
                    w.hibern_wkup_polarity()
                        .bit(!r.hibern_wkup_polarity().bit())
                });
            }

            while !crg_top.clkless_wakeup_stat() {}

            // Set hibernation sleep mode
            self.crg_aon
                .hibern_ctrl_reg
                .modify(|_, w| w.hibernation_enable().set_bit());
            scb.set_sleepdeep();

            // Configure the state of RAM blocks during hibernation mode
            crg_top.set_ram_pwr_ctrl(
                (!sleep_config.ram1_on) as u8,
                (!sleep_config.ram2_on) as u8,
                (!sleep_config.ram3_on) as u8,
            );

            // Remap address
            crg_top.set_remap_addr(sleep_config.remap_addr as u8);

            // Enable/Disable latching of pads state during sleep
            self.crg_aon
                .pad_latch_reg
                .modify(|_, w| w.pad_latch_en().bit(sleep_config.pad_latch_en));

            // Disable the TLS (Transparent Light Sleep) core feature
            self.crg_aon
                .ram_lpmx_reg
                .modify(|_, w| unsafe { w.ramx_lpmx().bits(7) });

            self.crg_aon.power_aon_ctrl_reg.modify(|_, w| {
                // Set required LDO_RET_TRIM value (for -40 - +40°C)
                unsafe {
                    w.ldo_ret_trim().bits(
                        if !sleep_config.ram1_on && !sleep_config.ram2_on && !sleep_config.ram3_on {
                            0x0e
                        } else {
                            0x0d
                        },
                    );
                }

                // Disable the testmode comparator
                w.force_running_comp_dis().set_bit();

                w
            });

            // Perform the following steps when in boost (or bypass) mode
            if crg_top.boost_selected() {
                self.crg_aon.power_aon_ctrl_reg.modify(|_, w| {
                    // Force connection between VBAT_HIGH and VBAT_LOW
                    unsafe {
                        w.vbat_hl_connect_res_ctrl().bits(2);
                    }

                    // Do not charge VBAT_HIGH in boost mode
                    w.charge_vbat_disable().set_bit();

                    w
                });
            }
            // Perform the following steps, when in buck mode
            else {
                // Set automatic connection control between VBAT_HIGH and VBAT_LOW
                self.crg_aon
                    .power_aon_ctrl_reg
                    .modify(|_, w| unsafe { w.vbat_hl_connect_res_ctrl().bits(3) })
            }

            self.crg_aon.power_aon_ctrl_reg.modify(|_, w| {
                // Disable the POR triggered from VBAT_HIGH voltage level sensing
                w.por_vbat_high_rst_mask().set_bit();

                // Enable the POR triggered from VBAT_LOW voltage level sensing
                w.por_vbat_low_rst_mask().clear_bit();
                w
            });

            // Set for proper RCX operation
            self.crg_aon
                .gp_data_reg
                .modify(|_, w| unsafe { w.ana_spare().bits(2) });

            crate::cm::asm::nop();
            crate::cm::asm::nop();
            crate::cm::asm::nop();

            // Enter Deep Sleep Mode
            crate::cm::asm::wfi();
        }
    }

    pub trait WakeupPin {
        fn mask() -> u8;
    }

    macro_rules! wakeup_pins {
        ($($PIN:ident => $bit:literal,)+) => {
            $(
                impl WakeupPin for crate::gpio::p0::$PIN<crate::gpio::Input<crate::gpio::PullUp>> {
                    fn mask() -> u8 {
                        1 << $bit
                    }
                }
                impl WakeupPin for crate::gpio::p0::$PIN<crate::gpio::Input<crate::gpio::PullDown>> {
                    fn mask() -> u8 {
                        1 << $bit
                    }
                }
                impl WakeupPin for crate::gpio::p0::$PIN<crate::gpio::Input<crate::gpio::Floating>> {
                    fn mask() -> u8 {
                        1 << $bit
                    }
                }
            )+
        };
    }

    wakeup_pins!(
        P0_01 => 1,
        P0_02 => 2,
        P0_03 => 3,
        P0_04 => 4,
        P0_05 => 5,

    );
}
