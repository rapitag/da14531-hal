use crate::{
    cm::peripheral::SCB,
    interrupt::InterruptController,
    pac::{CRG_AON, CRG_TOP},
    watchdog::SystemWatchdog,
};

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum RemapAddr {
    ToRom = 0,
    ToOtp = 1,
    ToRam1 = 2,
    ToRam3 = 3,
}

pub struct SleepController {
    crg_aon: CRG_AON,
    pin_mask: u8,
    ram1_on: bool,
    ram2_on: bool,
    ram3_on: bool,
    remap_addr0: RemapAddr,
    pad_latch_en: bool,
}

impl SleepController {
    pub fn new(
        crg_aon: CRG_AON,
        pin_mask: u8,
        ram1_on: bool,
        ram2_on: bool,
        ram3_on: bool,
        remap_addr0: RemapAddr,
        pad_latch_en: bool,
    ) -> Self {
        assert!(pin_mask & 0b11100000 == 0);

        Self {
            crg_aon,
            pin_mask: pin_mask & 0b11111,
            ram1_on,
            ram2_on,
            ram3_on,
            remap_addr0,
            pad_latch_en,
        }
    }

    // see: arch_set_hibernation
    pub fn enter_hibernation(
        &mut self,
        wdg: &mut SystemWatchdog,
        interrupt_controller: &mut InterruptController,
        scb: &mut SCB,
    ) {
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
            .modify(|_, w| unsafe { w.hibern_wkup_mask().bits(self.pin_mask) });

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
                w.ram1_pwr_ctrl().bits((!self.ram1_on) as u8);
                w.ram2_pwr_ctrl().bits((!self.ram2_on) as u8);
                w.ram3_pwr_ctrl().bits((!self.ram3_on) as u8);
            }
            w
        });

        // Remap address 0
        crg_top
            .sys_ctrl_reg
            .modify(|_, w| unsafe { w.remap_adr0().bits(self.remap_addr0 as u8) });

        // Enable/Disable latching of pads state during sleep
        self.crg_aon
            .pad_latch_reg
            .modify(|_, w| w.pad_latch_en().bit(self.pad_latch_en));

        // Disable the TLS (Transparent Light Sleep) core feature
        self.crg_aon
            .ram_lpmx_reg
            .modify(|_, w| unsafe { w.ramx_lpmx().bits(7) });

        self.crg_aon.power_aon_ctrl_reg.modify(|_, w| {
            // Set required LDO_RET_TRIM value (for -40 - +40°C)
            unsafe {
                w.ldo_ret_trim()
                    .bits(if !self.ram1_on && !self.ram2_on && !self.ram3_on {
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
