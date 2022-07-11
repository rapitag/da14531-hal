use crate::{
    clock::{ClockController, PeripheralClock},
    interrupt::{InterruptController, Irq},
    pac::WKUP,
};

#[repr(u8)]
pub enum Polarity {
    High = 1,
    Low = 0,
}

pub struct WakeupController {
    wkup: WKUP,
}

impl WakeupController {
    pub fn new(wkup: WKUP) -> Self {
        Self { wkup }
    }

    pub fn register_irq_handler(&mut self, handler: fn()) {
        unsafe {
            IRQ_HANDLER_WAKEUP = Some(handler);
        }
    }

    pub fn enable_irq(
        &mut self,
        clock_controller: &mut ClockController,
        interrupt_controller: &mut InterruptController,
        pin: u8,
        polarity: Polarity,
        events_num: u8,
        debounce_time: u8,
    ) {
        assert!(events_num > 0);

        clock_controller.set_peripheral_clock_state(PeripheralClock::WakeupController, true);

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

        interrupt_controller.set_priority(Irq::WakupQuadec, 2);
        interrupt_controller.enable_irq(Irq::WakupQuadec);
    }
}

#[no_mangle]
pub unsafe extern "C" fn WKUP_QUADEC_Handler() {
    let wkup = &(*(crate::pac::WKUP::ptr()));

    // Reset the interrupt
    wkup.wkup_irq_status_reg
        .modify(|_, w| w.wkup_irq_status().set_bit());

    // No more interrupts of this kind
    wkup.wkup_ctrl_reg
        .modify(|_, w| w.wkup_enable_irq().clear_bit());

    if let Some(handler) = IRQ_HANDLER_WAKEUP {
        handler();
    }
}

static mut IRQ_HANDLER_WAKEUP: Option<fn()> = None;
