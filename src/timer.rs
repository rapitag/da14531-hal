use crate::{
    interrupt::{InterruptController, Irq},
    pac::{CRG_TOP, TIMER0},
};

const SYSTEM_CLOCK_FREQ: u32 = 16_000_000;
const LOW_POWER_CLOCK_FREQ: u32 = 32_000;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum BaseClockDiv {
    Div1 = 0,
    Div2 = 1,
    Div4 = 2,
    Div8 = 3,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ClockSel {
    SystemClock = 1,
    LowPowerClock = 0,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum PwmMode {
    SystemClock = 1,
    High = 0,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum TimerClockDiv {
    Off = 1,
    Div10 = 0,
}

pub enum Timer2PwmChannel {
    Pwm2,
    Pwm3,
    Pwm4,
    Pwm5,
    Pwm6,
    Pwm7,
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

    pub fn set_clock_div(&mut self, div: BaseClockDiv) {
        let crg_top = unsafe { &*CRG_TOP::ptr() };
        crg_top
            .clk_per_reg
            .modify(|_, w| unsafe { w.tmr_div().bits(div as u8) })
    }

    pub fn init(
        &mut self,
        interrupt_controller: &mut InterruptController,
        clk_sel: ClockSel,
        pwm_mode: PwmMode,
        clk_div: TimerClockDiv,
    ) {
        self.timer.timer0_ctrl_reg.modify(|_, w| {
            w.tim0_clk_sel().bit(clk_sel as u8 == 1);
            w.pwm_mode().bit(pwm_mode as u8 == 1);
            w.tim0_clk_div().bit(clk_div as u8 == 1);
            w
        });

        interrupt_controller.set_priority(Irq::SwTim0, 2);
        interrupt_controller.enable_irq(Irq::SwTim0);
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

    pub fn register_handler(&self, handler: fn()) {
        unsafe {
            TIMER0_HANDLER = Some(handler);
        }
    }

    pub fn init_triple_pwm(&mut self, clk_sel: ClockSel, freq_hz: u32) {
        let pwm_freq = match clk_sel {
            ClockSel::SystemClock => ((SYSTEM_CLOCK_FREQ / freq_hz) - 1) as u16,
            ClockSel::LowPowerClock => ((LOW_POWER_CLOCK_FREQ / freq_hz) - 1) as u16,
        };

        self.timer
            .triple_pwm_ctrl_reg
            .modify(|_, w| w.triple_pwm_clk_sel().bit(clk_sel as u8 == 1));

        self.timer
            .triple_pwm_frequency
            .modify(|_, w| unsafe { w.pwm_freq().bits(pwm_freq) })
    }

    pub fn start_triple_pwm(&mut self) {
        self.timer
            .triple_pwm_ctrl_reg
            .modify(|_, w| w.triple_pwm_enable().set_bit());
    }

    pub fn stop_triple_pwm(&mut self) {
        self.timer
            .triple_pwm_ctrl_reg
            .modify(|_, w| w.triple_pwm_enable().clear_bit());
    }

    pub fn set_triple_pwm_duty_cycle(&mut self, channel: Timer2PwmChannel, start: u16, end: u16) {
        match channel {
            Timer2PwmChannel::Pwm2 => {
                self.timer
                    .pwm2_start_cycle
                    .write(|w| unsafe { w.start_cycle().bits(start) });
                self.timer
                    .pwm2_end_cycle
                    .write(|w| unsafe { w.end_cycle().bits(end) });
            }
            Timer2PwmChannel::Pwm3 => {
                self.timer
                    .pwm3_start_cycle
                    .write(|w| unsafe { w.start_cycle().bits(start) });
                self.timer
                    .pwm3_end_cycle
                    .write(|w| unsafe { w.end_cycle().bits(end) });
            }
            Timer2PwmChannel::Pwm4 => {
                self.timer
                    .pwm4_start_cycle
                    .write(|w| unsafe { w.start_cycle().bits(start) });
                self.timer
                    .pwm4_end_cycle
                    .write(|w| unsafe { w.end_cycle().bits(end) });
            }
            Timer2PwmChannel::Pwm5 => {
                self.timer
                    .pwm5_start_cycle
                    .write(|w| unsafe { w.start_cycle().bits(start) });
                self.timer
                    .pwm5_end_cycle
                    .write(|w| unsafe { w.end_cycle().bits(end) });
            }
            Timer2PwmChannel::Pwm6 => {
                self.timer
                    .pwm6_start_cycle
                    .write(|w| unsafe { w.start_cycle().bits(start) });
                self.timer
                    .pwm6_end_cycle
                    .write(|w| unsafe { w.end_cycle().bits(end) });
            }
            Timer2PwmChannel::Pwm7 => {
                self.timer
                    .pwm7_start_cycle
                    .write(|w| unsafe { w.start_cycle().bits(start) });
                self.timer
                    .pwm7_end_cycle
                    .write(|w| unsafe { w.end_cycle().bits(end) });
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn SWTIM_Handler() {
    if let Some(handler) = TIMER0_HANDLER {
        handler();
    }
}

static mut TIMER0_HANDLER: Option<fn()> = None;
