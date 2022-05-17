use embedded_hal::watchdog::{Watchdog, WatchdogEnable};

use crate::pac::SYS_WDOG;

pub const WATCHDOG_DEFAULT_PERIOD: u8 = 0xC8;

/// Wraps the System Watchdog (SYS_WDOG) peripheral
pub struct SystemWatchdog {
    sys_wdog: SYS_WDOG,
    period: u8,
}

impl SystemWatchdog {
    /// Wrap and start the watchdog
    pub fn new(sys_wdog: SYS_WDOG) -> Self {
        SystemWatchdog {
            sys_wdog,
            period: WATCHDOG_DEFAULT_PERIOD,
        }
    }

    pub fn start(&mut self, period: u8) {
        self.period = period;

        self.feed();

        self.sys_wdog
            .watchdog_ctrl_reg
            .write(|w| { w.nmi_rst().set_bit() });

        // TODO: Implement this!
        // wdg_resume (); 
        // -> SetWord16(RESET_FREEZE_REG, FRZ_WDOG);
    }

    pub fn feed(&mut self) {
        self.sys_wdog
            .watchdog_reg
            .write(|w| unsafe { w.wdog_val().bits(self.period) });
    }
}

impl WatchdogEnable for SystemWatchdog {
    type Time = u8;

    fn start<T: Into<Self::Time>>(&mut self, period: T) {
        self.start(period.into())
    }
}

impl Watchdog for SystemWatchdog {
    fn feed(&mut self) {
        self.feed()
    }
}
