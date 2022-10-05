#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Mode {
    DeepStandby = 0,
    Standby,
    Read,
    Program,
    ProgramVerify,
    InitialRead,
    DmaRead,
}

impl Default for Mode {
    fn default() -> Self {
        Self::DeepStandby
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct OtpcConfig {
    pub(crate) mode: Mode,
}
