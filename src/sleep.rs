use crate::{
    cm::peripheral::SCB,
    interrupt::InterruptController,
    pac::{CRG_AON, CRG_TOP},
    watchdog::SystemWatchdog,
};

#[derive(PartialEq)]
#[repr(u8)]
pub enum RemapAddr {
    ToRom = 0,
    ToOtp = 1,
    ToRam1 = 2,
    ToRam3 = 3,
}

pub struct SleepController {
    crg_aon: CRG_AON,
}

impl SleepController {
    pub fn new(crg_aon: CRG_AON) -> Self {
        Self { crg_aon }
    }

    // see: arch_set_deep_sleep
    // pub fn enter_deep_sleep(
    //     &mut self,
    //     wdg: &mut SystemWatchdog,
    //     interrupt_controller: &mut InterruptController,
    //     scb: &mut SCB,
    //     ram1_on: bool,
    //     ram2_on: bool,
    //     ram3_on: bool,
    //     pad_latch_en: bool,
    //     led: &mut Pin<Output>,
    // ) {
    //     let crg_top = unsafe { &*CRG_TOP::ptr() };
    //     // Stop watchdog timer
    //     wdg.freeze();
    //     // Disable Interrupts
    //     crate::cm::interrupt::disable();
    //     // Disable radio
    //     crg_top
    //         .pmu_ctrl_reg
    //         .modify(|_, w| w.radio_sleep().set_bit());
    //     crg_top.clk_radio_reg.modify(|_, w| {
    //         // Disable the BLE core clocks
    //         w.ble_enable().clear_bit();
    //         // Apply HW reset to BLE_Timers
    //         w.ble_lp_reset().set_bit();
    //         w
    //     });
    //     // Clear quadec pending interrupt - mask quadec interrupt
    //     self.quadec
    //         .qdec_ctrl_reg
    //         .modify(|_, w| w.qdec_irq_status().set_bit());
    //     // Close peripheral clock
    //     crg_top
    //         .clk_per_reg
    //         .modify(|_, w| w.quad_enable().clear_bit());
    //     // Clear all pending interrupts
    //     interrupt_controller.clear_pending_interrupts();
    //     // Debugger must be disabled before entering a sleep mode. Wait until debugger has been disabled.
    //     while crg_top.sys_stat_reg.read().dbg_is_up().bit() {}
    //     crg_top
    //         .sys_ctrl_reg
    //         .modify(|_, w| unsafe { w.debugger_enable().bits(0) });
    //     // Set deep sleep mode
    //     self.crg_aon
    //         .hibern_ctrl_reg
    //         .modify(|_, w| w.hibernation_enable().clear_bit());
    //     scb.set_sleepdeep();
    //     // Configure the state of RAM blocks during the deep sleep mode
    //     crg_top.ram_pwr_ctrl_reg.modify(|_, w| {
    //         unsafe {
    //             w.ram1_pwr_ctrl().bits((!ram1_on) as u8);
    //             w.ram2_pwr_ctrl().bits((!ram2_on) as u8);
    //             w.ram3_pwr_ctrl().bits((!ram3_on) as u8);
    //         }
    //         w
    //     });
    //     // Perform HW reset on wake-up
    //     crg_top
    //         .pmu_ctrl_reg
    //         .modify(|_, w| w.reset_on_wakeup().set_bit());
    //     // Enable/Disable latching of pads state during sleep
    //     self.crg_aon
    //         .pad_latch_reg
    //         .modify(|_, w| w.pad_latch_en().bit(pad_latch_en));
    //     self.crg_aon.power_aon_ctrl_reg.modify(|_, w| {
    //         // Set clamp output (0xF = 603mV)
    //         unsafe { w.ldo_ret_trim().bits(0x0f) };
    //         // Disable the testmode comparator
    //         w.force_running_comp_dis().set_bit();
    //         w
    //     });
    //     // Perform the following steps when in boost (or bypass) mode
    //     if crg_top.ana_status_reg.read().boost_selected().bit() {
    //         // Always LDO_LOW Off
    //         crg_top
    //             .power_ctrl_reg
    //             .modify(|_, w| unsafe { w.ldo_low_ctrl_reg().bits(1) });
    //         // Force connection between VBAT_HIGH and VBAT_LOW
    //         self.crg_aon
    //             .power_aon_ctrl_reg
    //             .modify(|_, w| unsafe { w.vbat_hl_connect_res_ctrl().bits(2) });
    //     }
    //     // Perform the following steps, when in buck mode
    //     else {
    //         // High-current mode in active, Low-current mode in sleep
    //         crg_top
    //             .power_ctrl_reg
    //             .modify(|_, w| unsafe { w.ldo_low_ctrl_reg().bits(3) });
    //         // Set automatic connection control between VBAT_HIGH and VBAT_LOW
    //         self.crg_aon
    //             .power_aon_ctrl_reg
    //             .modify(|_, w| unsafe { w.vbat_hl_connect_res_ctrl().bits(3) })
    //     }
    //     self.crg_aon.power_aon_ctrl_reg.modify(|_, w| {
    //         // Disable the POR triggered from VBAT_HIGH voltage level sensing
    //         w.por_vbat_high_rst_mask().set_bit();
    //         // Enable the POR triggered from VBAT_LOW voltage level sensing
    //         w.por_vbat_low_rst_mask().clear_bit();
    //         w
    //     });
    //     // Set for proper RCX operation
    //     self.crg_aon
    //         .gp_data_reg
    //         .modify(|_, w| unsafe { w.ana_spare().bits(2) });
    //     crate::cm::asm::nop();
    //     crate::cm::asm::nop();
    //     crate::cm::asm::nop();
    //     // Enter Deep Sleep Mode
    //     crate::cm::asm::wfi();
    // }

    // see: arch_set_hibernation
    pub fn enter_hibernation(
        &mut self,
        wdg: &mut SystemWatchdog,
        interrupt_controller: &mut InterruptController,
        scb: &mut SCB,
        wakeup_pin: u8,
        ram1_on: bool,
        ram2_on: bool,
        ram3_on: bool,
        remap_addr0: RemapAddr,
        pad_latch_en: bool,
    ) {
        assert!(wakeup_pin > 0 && wakeup_pin <= 5);
        // assert!((remap_addr0 == RemapAddr::ToRam1) != ram1_on);
        // assert!((remap_addr0 == RemapAddr::ToRam3) != ram3_on);

        let crg_top = unsafe { &*CRG_TOP::ptr() };

        // Stop watchdog timer
        wdg.freeze();

        // Disable Interrupts
        crate::cm::interrupt::disable();

        // Clear all pending interrupts
        interrupt_controller.clear_pending_interrupts();

        // // Store the debugger configuration
        // booter_val.dbg_cfg = GetBits16(SYS_CTRL_REG, DEBUGGER_ENABLE);

        // Debugger must be disabled before entering a sleep mode. Wait until debugger has been disabled.
        while crg_top.sys_stat_reg.read().dbg_is_up().bit() {}

        crg_top
            .sys_ctrl_reg
            .modify(|_, w| unsafe { w.debugger_enable().bits(0) });

        // Set the wake up pins
        self.crg_aon
            .hibern_ctrl_reg
            .modify(|_, w| unsafe { w.hibern_wkup_mask().bits(1 << (wakeup_pin - 1)) });

        // Check the output of the clockless wakeup XOR tree to determine the wake up polarity
        if !crg_top.ana_status_reg.read().clkless_wakeup_stat().bit() {
            self.crg_aon.hibern_ctrl_reg.modify(|r, w| {
                w.hibern_wkup_polarity()
                    .bit(!r.hibern_wkup_polarity().bit())
            });
        }

        while !crg_top.ana_status_reg.read().clkless_wakeup_stat().bit() {}

        // Set hibernation sleep mode
        self.crg_aon
            .hibern_ctrl_reg
            .modify(|_, w| w.hibernation_enable().set_bit());
        scb.set_sleepdeep();

        // Configure the state of RAM blocks during hibernation mode
        crg_top.ram_pwr_ctrl_reg.modify(|_, w| {
            unsafe {
                w.ram1_pwr_ctrl().bits((!ram1_on) as u8);
                w.ram2_pwr_ctrl().bits((!ram2_on) as u8);
                w.ram3_pwr_ctrl().bits((!ram3_on) as u8);
            }
            w
        });

        // Remap address 0
        crg_top
            .sys_ctrl_reg
            .modify(|_, w| unsafe { w.remap_adr0().bits(remap_addr0 as u8) });

        // Enable/Disable latching of pads state during sleep
        self.crg_aon
            .pad_latch_reg
            .modify(|_, w| w.pad_latch_en().bit(pad_latch_en));

        // Disable the TLS (Transparent Light Sleep) core feature
        self.crg_aon
            .ram_lpmx_reg
            .modify(|_, w| unsafe { w.ramx_lpmx().bits(7) });

        self.crg_aon.power_aon_ctrl_reg.modify(|_, w| {
            // Set required LDO_RET_TRIM value (for -40 - +40°C)
            unsafe {
                w.ldo_ret_trim().bits(if !ram1_on && !ram2_on && !ram3_on {
                    0x0e
                } else {
                    0x0d
                });
            }

            // Disable the testmode comparator
            w.force_running_comp_dis().set_bit();

            w
        });

        // Perform the following steps when in boost (or bypass) mode
        if crg_top.ana_status_reg.read().boost_selected().bit() {
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
