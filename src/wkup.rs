use crate::{
    crg_top::CrgTop,
    nvic::{Irq, Nvic},
    pac::WKUP,
};

/// Extension trait that constrains the `CRG_TOP` peripheral
pub trait WkupExt {
    /// Constrains the `CRG_TOP` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Wkup;
}

impl WkupExt for WKUP {
    fn constrain(self) -> Wkup {
        Wkup { wkup: self }
    }
}

#[repr(u8)]
pub enum Polarity {
    High = 1,
    Low = 0,
}

pub struct Wkup {
    wkup: WKUP,
}

impl Wkup {
    pub fn new(wkup: WKUP) -> Self {
        Self { wkup }
    }

    pub fn enable_irq(
        &mut self,
        crg_top: &mut CrgTop,
        nvic: &mut Nvic,
        pin: u8,
        polarity: Polarity,
        events_num: u8,
        debounce_time: u8,
    ) {
        assert!(events_num > 0);

        crg_top.enable_peripheral::<WKUP>();

        // Reset event counter
        self.wkup
            .wkup_irq_status_reg
            .modify(|_, w| w.wkup_cntr_rst().set_bit());

        // Set debounce time
        self.wkup
            .wkup_ctrl_reg
            .modify(|_, w| unsafe { w.wkup_deb_value().bits(debounce_time & 0x3f) });

        // Set wakeup polarity
        self.wkup.wkup_pol_gpio_reg.modify(|r, w| {
            let mask = match polarity {
                Polarity::High => r.wkup_pol_gpio().bits() | (1 << pin),
                Polarity::Low => r.wkup_pol_gpio().bits() & !(1 << pin),
            };
            unsafe {
                w.wkup_pol_gpio().bits(mask);
            }
            w
        });

        // Wait for events_num events and wakeup
        self.wkup
            .wkup_compare_reg
            .modify(|_, w| unsafe { w.wkup_compare().bits(events_num - 1) });

        // Enable IRQ in Wakeup controller
        self.wkup
            .wkup_ctrl_reg
            .modify(|_, w| w.wkup_enable_irq().set_bit());

        // Set wake up pin
        self.wkup.wkup_select_gpio_reg.modify(|r, w| {
            let mask = r.wkup_select_gpio().bits() | (1 << pin);
            unsafe {
                w.wkup_select_gpio().bits(mask);
            }
            w
        });

        nvic.set_priority(Irq::WakupQuadec, 2);
        nvic.enable_irq(Irq::WakupQuadec);
    }
}
