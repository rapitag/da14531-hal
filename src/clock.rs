use crate::pac::CRG_TOP;

#[repr(u16)]
pub enum PeripheralClock {
    QuadratureDecoder = (1 << 11),
    Spi = (1 << 10),
    Uart1 = (1 << 7),
    Uart2 = (1 << 6),
    I2c = (1 << 5),
    WakeupController = (1 << 4),
    Timer = (1 << 3),
}

pub struct ClockController {
    crg_top: CRG_TOP,
}

impl ClockController {
    pub fn new(crg_top: CRG_TOP) -> Self {
        Self { crg_top }
    }

    pub fn set_peripheral_clock_state(&mut self, clock: PeripheralClock, state: bool) {
        crate::cm::interrupt::free(|_| {
            self.crg_top.clk_per_reg.modify(|r, w| {
                let mask = match state {
                    true => r.bits() | clock as u16,
                    false => r.bits() & !(clock as u16),
                };

                unsafe { w.bits(mask) }
            });
        });
    }

    pub fn use_lowest_amba_clocks(&mut self) {
        self.crg_top.clk_amba_reg.modify(|_, w| unsafe {
            w.pclk_div().bits(3);
            w.hclk_div().bits(3);
            w
        });
    }

    pub fn use_highest_amba_clocks(&mut self) {
        self.crg_top.clk_amba_reg.modify(|_, w| unsafe {
            w.pclk_div().bits(0);
            w.hclk_div().bits(0);
            w
        });
    }
}
