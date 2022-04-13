use core::fmt;

use embedded_hal::watchdog::{Watchdog, WatchdogEnable};

use crate::pac::SYS_WDOG;

/// Wraps the System Watchdog (SYS_WDOG) peripheral
pub struct SystemWatchdog {
    sys_wdog: SYS_WDOG,
}

impl fmt::Debug for SystemWatchdog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Watchdog")
    }
}

const WATCHDOG_DEFAULT_PERIOD: u8 = 0xC8;

impl SystemWatchdog {
    /// Wrap and start the watchdog
    pub fn new(sys_wdog: SYS_WDOG) -> Self {
        SystemWatchdog { sys_wdog }
    }

    // /// Debug independent watchdog stopped when core is halted
    // pub fn stop_on_debug(&self, dbgmcu: &DBGMCU, stop: bool) {
    //     dbgmcu.apb1_fz.modify(|_, w| w.dbg_sys_wdog_stop().bit(stop));
    // }

    // fn setup(&self, timeout_ms: u32) {
    //     let mut pr = 0;
    //     while pr < MAX_PR && Self::timeout_period(pr, MAX_RL) < timeout_ms {
    //         pr += 1;
    //     }

    //     let max_period = Self::timeout_period(pr, MAX_RL);
    //     let max_rl = u32::from(MAX_RL);
    //     let rl = (timeout_ms * max_rl / max_period).min(max_rl) as u16;

    //     self.access_registers(|sys_wdog| {
    //         sys_wdog.pr.modify(|_, w| w.pr().bits(pr));
    //         sys_wdog.rlr.modify(|_, w| w.rl().bits(rl));
    //     });
    // }

    // fn is_pr_updating(&self) -> bool {
    //     self.sys_wdog.sr.read().pvu().bit()
    // }

    // /// Returns the interval in ms
    // pub fn interval(&self) -> MilliSeconds {
    //     while self.is_pr_updating() {}

    //     let pr = self.sys_wdog.pr.read().pr().bits();
    //     let rl = self.sys_wdog.rlr.read().rl().bits();
    //     let ms = Self::timeout_period(pr, rl);
    //     MilliSeconds::from_ticks(ms)
    // }

    // /// pr: Prescaler divider bits, rl: reload value
    // ///
    // /// Returns ms
    // fn timeout_period(pr: u8, rl: u16) -> u32 {
    //     let divider: u32 = match pr {
    //         0b000 => 4,
    //         0b001 => 8,
    //         0b010 => 16,
    //         0b011 => 32,
    //         0b100 => 64,
    //         0b101 => 128,
    //         0b110 => 256,
    //         0b111 => 256,
    //         _ => unreachable!(),
    //     };
    //     (u32::from(rl) + 1) * divider / 32
    // }

    // fn access_registers<A, F: FnMut(&SYS_WDOG) -> A>(&self, mut f: F) -> A {
    //     // Unprotect write access to registers
    //     self.sys_wdog.kr.write(|w| unsafe { w.key().bits(KR_ACCESS) });
    //     let a = f(&self.sys_wdog);

    //     // Protect again
    //     self.sys_wdog.kr.write(|w| unsafe { w.key().bits(KR_RELOAD) });
    //     a
    // }

    pub fn start(&mut self, _period: u32) {
        // TODO: Implement this!

        // self.setup(period.ticks());

        // self.sys_wdog.kr.write(|w| unsafe { w.key().bits(KR_START) });
    }

    pub fn feed(&mut self) {
        self.sys_wdog
            .watchdog_reg
            .write(|w| unsafe { w.wdog_val().bits(WATCHDOG_DEFAULT_PERIOD) });
    }
}

impl WatchdogEnable for SystemWatchdog {
    type Time = u32;

    fn start<T: Into<Self::Time>>(&mut self, period: T) {
        self.start(period.into())
    }
}

impl Watchdog for SystemWatchdog {
    fn feed(&mut self) {
        self.feed()
    }
}
