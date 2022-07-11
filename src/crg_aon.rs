use da14531::CRG_AON;

pub trait CrgAonExt {
    fn take(self) -> CrgAon;
}

impl CrgAonExt for CRG_AON {
    fn take(self) -> CrgAon {
        CrgAon
    }
}

pub struct CrgAon;
impl CrgAon {
    const fn ptr() -> *const crate::pac::crg_aon::RegisterBlock {
        crate::pac::CRG_AON::ptr()
    }

    pub fn set_pad_latch_en(&mut self, state: bool) {
        unsafe {
            (*CrgAon::ptr())
                .pad_latch_reg
                .write(|w| w.pad_latch_en().bit(state));
        }
    }
}
