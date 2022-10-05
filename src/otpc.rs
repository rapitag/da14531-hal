use crate::{crg_top::CrgTop, pac::OTPC};

pub mod config;

use config::OtpcConfig;

use self::config::Mode;

/// Extension trait that constrains the `OTPC` peripheral
pub trait OtpcExt {
    /// Constrains the `OTPC` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Otpc;
}

impl OtpcExt for OTPC {
    fn constrain(self) -> Otpc {
        Otpc { otpc: self }
    }
}

pub struct Otpc {
    otpc: OTPC,
}

impl Otpc {
    pub fn enable(&self, crg_top: &mut CrgTop, otpc_config: OtpcConfig) {
        crg_top.enable_peripheral::<OTPC>();
        self.otpc
            .otpc_mode_reg
            .write(|w| unsafe { w.otpc_mode_mode().bits(otpc_config.mode as u8) })
    }

    pub fn disable(&self, crg_top: &mut CrgTop) {
        self.otpc
            .otpc_mode_reg
            .write(|w| unsafe { w.otpc_mode_mode().bits(Mode::default() as u8) });
        crg_top.disable_peripheral::<OTPC>();
    }
}
