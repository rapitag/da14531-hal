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

use crate::pac::crg_top::RegisterBlock as CrgTopRB;

/// Enable/disable peripheral
pub trait Enable {
    fn enable(crg_top: &CrgTopRB);
    fn disable(crg_top: &CrgTopRB);
}

/// Extension trait that constrains the `CRG_TOP` peripheral
pub trait CrgTopExt {
    /// Constrains the `CRG_TOP` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> CrgTop;
}

macro_rules! peripheral_clock_enable {
    ($($PER:ident => ($reg:ident, $field:ident),)+) => {
        $(
            impl crate::Sealed for crate::pac::$PER {}

            impl Enable for crate::pac::$PER {
                fn enable(crg_top: &CrgTopRB) {
                    crate::cm::interrupt::free(|_| {
                        crg_top.$reg.modify(|_, w| w.$field().set_bit());
                    });
                }

                fn disable(crg_top: &CrgTopRB) {
                    crate::cm::interrupt::free(|_| {
                        crg_top.$reg.modify(|_, w| w.$field().clear_bit());
                    });
                }
            }

        )+
    };
}

peripheral_clock_enable!(
    QUADEC => (clk_per_reg, quad_enable),
    SPI => (clk_per_reg, spi_enable),
    UART => (clk_per_reg, uart1_enable),
    UART2 => (clk_per_reg, uart2_enable),
    I2C => (clk_per_reg, i2c_enable),
    WKUP => (clk_per_reg, wakeupct_enable),
    TIMER0 => (clk_per_reg, tmr_enable),
    TIMER1 => (clk_per_reg, tmr_enable),
    OTPC => (clk_amba_reg, otp_enable),
);

impl CrgTopExt for CRG_TOP {
    fn constrain(self) -> CrgTop {
        CrgTop { crg_top: self }
    }
}

pub struct CrgTop {
    crg_top: CRG_TOP,
}

impl CrgTop {
    pub fn enable_peripheral<P: Enable>(&self) {
        P::enable(&self.crg_top);
    }

    pub fn disable_peripheral<P: Enable>(&self) {
        P::disable(&self.crg_top);
    }

    #[inline]
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

    #[inline]
    pub fn use_lowest_amba_clocks(&mut self) {
        self.crg_top.clk_amba_reg.modify(|_, w| unsafe {
            w.pclk_div().bits(3);
            w.hclk_div().bits(3);
            w
        });
    }

    #[inline]
    pub fn use_highest_amba_clocks(&mut self) {
        self.crg_top.clk_amba_reg.modify(|_, w| unsafe {
            w.pclk_div().bits(0);
            w.hclk_div().bits(0);
            w
        });
    }

    #[inline]
    pub fn is_dbg_up(&self) -> bool {
        self.crg_top.sys_stat_reg.read().dbg_is_up().bit()
    }

    #[inline]
    pub fn disable_dbg(&mut self) {
        self.crg_top
            .sys_ctrl_reg
            .modify(|_, w| unsafe { w.debugger_enable().bits(0) });
    }

    #[inline]
    pub fn clkless_wakeup_stat(&self) -> bool {
        self.crg_top
            .ana_status_reg
            .read()
            .clkless_wakeup_stat()
            .bit()
    }

    #[inline]
    pub fn boost_selected(&self) -> bool {
        self.crg_top.ana_status_reg.read().boost_selected().bit()
    }

    #[inline]
    pub fn set_remap_addr(&self, remap_address: u8) {
        self.crg_top
            .sys_ctrl_reg
            .modify(|_, w| unsafe { w.remap_adr0().bits(remap_address) });
    }

    #[inline]
    pub fn set_ram_pwr_ctrl(&self, ram1: u8, ram2: u8, ram3: u8) {
        self.crg_top.ram_pwr_ctrl_reg.modify(|_, w| {
            unsafe {
                w.ram1_pwr_ctrl().bits(ram1);
                w.ram2_pwr_ctrl().bits(ram2);
                w.ram3_pwr_ctrl().bits(ram3);
            }
            w
        });
    }
}
