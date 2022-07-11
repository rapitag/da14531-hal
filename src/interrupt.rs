use crate::cm::{interrupt::InterruptNumber, peripheral::NVIC};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Irq {
    /// GPIO Interrupt Request through debounce.
    Gpio0 = 10,
    /// GPIO Interrupt Request through debounce.
    Gpio1 = 11,
    /// GPIO Interrupt Request through debounce.
    Gpio2 = 12,
    /// GPIO Interrupt Request through debounce.
    Gpio3 = 13,
    /// GPIO Interrupt Request through debounce.
    Gpio4 = 14,
    ///  Software Timer (Timer0) Interrupt Request.
    SwTim0 = 15,
    /// Combines the Wake up Capture Timer Interrupt Request,
    /// the GPIO Interrupt and the QuadDecoder Interrupt Request.
    WakupQuadec = 16,
}

unsafe impl InterruptNumber for Irq {
    fn number(self) -> u16 {
        self as u8 as u16
    }
}

pub struct InterruptController {
    nvic: NVIC,
}

impl InterruptController {
    pub fn new(nvic: NVIC) -> Self {
        Self { nvic }
    }

    pub fn set_priority<I>(&mut self, interrupt: I, priority: u8)
    where
        I: InterruptNumber,
    {
        unsafe {
            self.nvic.set_priority(interrupt, priority);
        }
    }

    pub fn enable_irq<I>(&self, interrupt: I)
    where
        I: InterruptNumber,
    {
        unsafe { NVIC::unmask(interrupt) }
    }

    pub fn disable_irq<I>(&self, interrupt: I)
    where
        I: InterruptNumber,
    {
        NVIC::mask(interrupt)
    }

    pub fn clear_pending_interrupts(&mut self) {
        unsafe { self.nvic.icpr[0].write(0xFFFFFFFF) };
    }
}
