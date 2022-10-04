use crate::cm::interrupt::InterruptNumber;
use crate::pac::{NVIC, TIMER0};

use crate::cm::peripheral::nvic::RegisterBlock as NvicRB;

/// Extension trait that constrains the `SYS_WDOG` peripheral
pub trait NvicExt {
    /// Constrains the `SYS_WDOG` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Nvic;
}

impl NvicExt for NVIC {
    fn constrain(self) -> Nvic {
        Nvic { nvic: self }
    }
}

pub struct Nvic {
    nvic: NVIC,
}

impl Nvic {
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

pub trait Interrupt {
    fn set_priority(nvic: &mut NvicRB, prio: u8);
    fn enable();
    fn disable();
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Irq {
    /// Analog-Digital Converter Interrupt Request.
    Adc = 6,
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

// impl Interrupt for TIMER0 {
//     fn set_priority(nvic: &mut NvicRB, prio: u8) {
//         unsafe {
//             nvic.set_priority(Irq::SwTim0, prio);
//         }
//     }

//     fn enable() {
//         unsafe { NVIC::unmask(Irq::SwTim0) }
//     }

//     fn disable() {
//         NVIC::mask(Irq::SwTim0)
//     }
// }

// macro_rules! interrupt_sources {
//     ($($PER:ident => ($irqname:ident, $irqnum:literal),)+) => {
//         #[derive(Clone, Copy)]
//         #[repr(u8)]
//         pub enum Irq {
//             $(
//                 $irqname = $irqnum,
//             )+
//         }

//         unsafe impl InterruptNumber for Irq {
//             fn number(self) -> u16 {
//                 self as u8 as u16
//             }
//         }

//         $(
//             impl Interrupt for $PER {
//                 fn set_priority(nvic: &mut NvicRB, prio: u8) {
//                     unsafe {
//                         nvic.set_priority(Irq::$irqname, prio);
//                     }
//                 }

//                 fn enable() {
//                     unsafe { NVIC::unmask(Irq::$irqname) }
//                 }

//                 fn disable() {
//                     NVIC::mask(Irq::$irqname)
//                 }
//             }
//         )+
//     };
// }

// interrupt_sources!(
//     TIMER0 => (SwTim0, 15),
// );
