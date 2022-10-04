use embedded_hal::watchdog::{Watchdog, WatchdogDisable, WatchdogEnable};

use crate::pac::SYS_WDOG;

pub const WATCHDOG_DEFAULT_PERIOD: u8 = 0xC8;

/// Extension trait that constrains the `SYS_WDOG` peripheral
pub trait SysWdogExt {
    /// Constrains the `SYS_WDOG` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> SysWdog;
}

impl SysWdogExt for SYS_WDOG {
    fn constrain(self) -> SysWdog {
        SysWdog {
            sys_wdog: self,
            period: WATCHDOG_DEFAULT_PERIOD,
        }
    }
}

/// Wraps the System Watchdog (SYS_WDOG) peripheral
pub struct SysWdog {
    sys_wdog: SYS_WDOG,
    period: u8,
}

impl SysWdog {
    pub fn start(&mut self, period: u8) {
        self.period = period;

        self.feed();

        self.sys_wdog
            .watchdog_ctrl_reg
            .write(|w| w.nmi_rst().set_bit());

        // TODO: Implement this!
        // wdg_resume ();
        // -> SetWord16(RESET_FREEZE_REG, FRZ_WDOG);
    }

    pub fn feed(&mut self) {
        self.sys_wdog
            .watchdog_reg
            .modify(|_, w| unsafe { w.wdog_val().bits(self.period) });
    }

    pub fn freeze(&mut self) {
        let gpreg = unsafe { &*crate::pac::GPREG::ptr() };

        gpreg.set_freeze_reg.modify(|_, w| w.frz_wdog().set_bit());
    }
}

impl WatchdogEnable for SysWdog {
    type Time = u8;

    fn start<T: Into<Self::Time>>(&mut self, period: T) {
        self.start(period.into())
    }
}

impl Watchdog for SysWdog {
    fn feed(&mut self) {
        self.feed()
    }
}

impl WatchdogDisable for SysWdog {
    fn disable(&mut self) {
        self.freeze();
    }
}
