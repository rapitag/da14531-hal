use crate::{
    interrupt::{InterruptController, Irq},
    pac::{CRG_TOP, TIMER0},
};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ClockDiv {
    Div1 = 0,
    Div2 = 1,
    Div4 = 2,
    Div8 = 3,
}

pub struct Timer0 {
    timer: TIMER0,
}

impl Timer0 {
    pub fn new(timer: TIMER0) -> Self {
        Self { timer }
    }

    pub fn start(&mut self) {
        self.timer
            .timer0_ctrl_reg
            .modify(|_, w| w.tim0_ctrl().set_bit());
    }

    pub fn stop(&mut self) {
        self.timer
            .timer0_ctrl_reg
            .modify(|_, w| w.tim0_ctrl().clear_bit());
    }

    pub fn enable_clock(&mut self) {
        let crg_top = unsafe { &*CRG_TOP::ptr() };
        crg_top.clk_per_reg.modify(|_, w| w.tmr_enable().set_bit());
    }

    pub fn set_clock_div(&mut self, div: ClockDiv) {
        let crg_top = unsafe { &*CRG_TOP::ptr() };
        crg_top
            .clk_per_reg
            .modify(|_, w| unsafe { w.tmr_div().bits(div as u8) })
    }

    pub fn init(
        &mut self,
        interrupt_controller: &mut InterruptController,
        clk_sel: bool,
        pwm_mode: bool,
        clk_div: bool,
    ) {
        self.timer.timer0_ctrl_reg.modify(|_, w| {
            w.tim0_clk_sel().bit(clk_sel);
            w.pwm_mode().bit(pwm_mode);
            w.tim0_clk_div().bit(clk_div);
            w
        });

        interrupt_controller.set_priority(Irq::SwTim0, 2);
    }

    pub fn set_pwm(&mut self, pwm_on: u16, pwm_high: u16, pwm_low: u16) {
        self.timer
            .timer0_on_reg
            .modify(|_, w| unsafe { w.tim0_on().bits(pwm_on) });
        self.timer
            .timer0_reload_m_reg
            .modify(|_, w| unsafe { w.tim0_m().bits(pwm_high) });
        self.timer
            .timer0_reload_n_reg
            .modify(|_, w| unsafe { w.tim0_n().bits(pwm_low) });
    }

    pub fn set_pwm_on(&mut self, pwm_on: u16) {
        self.timer
            .timer0_on_reg
            .modify(|_, w| unsafe { w.tim0_on().bits(pwm_on) });
    }

    pub fn set_pwm_high(&mut self, pwm_high: u16) {
        self.timer
            .timer0_reload_m_reg
            .modify(|_, w| unsafe { w.tim0_m().bits(pwm_high) });
    }

    pub fn set_pwm_low(&mut self, pwm_low: u16) {
        self.timer
            .timer0_reload_n_reg
            .modify(|_, w| unsafe { w.tim0_n().bits(pwm_low) });
    }
}

#[no_mangle]
pub unsafe extern "C" fn SWTIM_Handler() {}
