//! HAL interface to the I2C peripheral.

use core::ops::Deref;

use crate::{
    crg_top::CrgTop,
    gpio::{AfI2cScl, AfI2cSda, Pin},
    nvic::{Irq, Nvic},
    pac::{i2c, I2C, NVIC},
};

/// Extension trait that constrains the `SYS_WDOG` peripheral
pub trait I2cExt {
    /// Constrains the `SYS_WDOG` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> I2c;
}

impl I2cExt for I2C {
    fn constrain(self) -> I2c {
        I2c {
            i2c: self,
            pins: None,
            speed: Default::default(),
            addressing_mode: Default::default(),
        }
    }
}

pub enum Speed {
    /// 100 kbit/s
    Standard,
    /// 400 kbit/s
    FullSpeed,
}

impl Default for Speed {
    fn default() -> Self {
        Speed::Standard
    }
}

pub enum AddressingMode {
    Bits7,
    Bits10,
}

impl Default for AddressingMode {
    fn default() -> Self {
        AddressingMode::Bits7
    }
}

struct Pins {
    _sda: Pin<AfI2cSda>,
    _scl: Pin<AfI2cScl>,
}

impl Pins {
    fn new(sda: Pin<AfI2cSda>, scl: Pin<AfI2cScl>) -> Self {
        Self {
            _scl: scl,
            _sda: sda,
        }
    }
}

pub struct I2c {
    i2c: I2C,
    pins: Option<Pins>,
    speed: Speed,
    addressing_mode: AddressingMode,
}

impl I2c {
    pub fn set_pins(mut self, sda: Pin<AfI2cSda>, scl: Pin<AfI2cScl>) -> Self {
        self.pins = Some(Pins::new(sda, scl));
        self
    }

    pub fn set_speed(mut self, speed: Speed) -> Self {
        self.speed = speed;
        self
    }

    pub fn set_addressing_mode(mut self, addressing_mode: AddressingMode) -> Self {
        self.addressing_mode = addressing_mode;
        self
    }

    pub fn start(&mut self, nvic: &mut Nvic, crg_top: &CrgTop) {
        assert!(self.pins.is_some());

        // Enable peripheral clock
        CrgTop::enable_peripheral::<I2C>(&crg_top);

        // Disable the I2C Controller
        self.disable_controller();

        // Enable all interrupts
        self.i2c.i2c_intr_mask_reg.write(|w| unsafe { w.bits(0) });

        self.i2c
            .i2c_ss_scl_hcnt_reg
            .write(|w| unsafe { w.bits(0x00000048) });
        self.i2c
            .i2c_ss_scl_lcnt_reg
            .write(|w| unsafe { w.bits(0x0000004F) });

        self.i2c
            .i2c_fs_scl_hcnt_reg
            .write(|w| unsafe { w.bits(0x00000008) });
        self.i2c
            .i2c_fs_scl_lcnt_reg
            .write(|w| unsafe { w.bits(0x00000017) });

        self.i2c.i2c_con_reg.write(|w| {
            // Configure the speed mode
            unsafe {
                // See:
                // * sdk/sdk/platform/driver/i2c/i2c.c:154
                // * sdk/sdk/platform/driver/i2c/i2c.h:288
                w.i2c_speed().bits(match self.speed {
                    Speed::Standard => 1,
                    Speed::FullSpeed => 0,
                });
            }

            // Setup as I2C master
            w.i2c_master_mode().set_bit();
            w.i2c_slave_disable().set_bit();

            // Configure addressing mode
            match self.addressing_mode {
                AddressingMode::Bits7 => {
                    w.i2c_10bitaddr_master().clear_bit();
                }
                AddressingMode::Bits10 => {
                    w.i2c_10bitaddr_master().set_bit();
                }
            }

            w
        });

        // Set threshold for RX/TX FIFO
        self.i2c
            .i2c_rx_tl_reg
            .write(|w| unsafe { w.rx_tl().bits(0) });
        self.i2c
            .i2c_tx_tl_reg
            .write(|w| unsafe { w.rx_tl().bits(0) });

        // Enable the I2C Controller
        self.enable_controller();

        nvic.set_priority(Irq::I2c, 2);
        nvic.enable_irq(Irq::I2c);
    }

    fn send_byte(&self, byte: u8, stop: bool) -> Result<(), Error> {
        // Wait until TX FIFO is empty
        while self.i2c.i2c_status_reg.read().tfe().bit_is_clear() {}

        crate::cm::interrupt::free(|_| {
            // Prepare to transmit the write command byte
            self.i2c.i2c_data_cmd_reg.write(|w| {
                w.i2c_cmd().clear_bit();

                if stop {
                    w.i2c_stop().set_bit();
                }

                unsafe {
                    w.dat().bits(byte);
                }

                w
            });
        });

        // Wait until master has finished reading the response byte from slave device
        // while self.i2c.i2c_status_reg.read().mst_activity().bit_is_set() {}

        // Read the I2C_TX_ABRT_SOURCE_REG register
        let abort_source = self.i2c.i2c_tx_abrt_source_reg.read().bits();
        if abort_source != 0 {
            self.i2c.i2c_clr_tx_abrt_reg.read().bits();
            Err(Error::Transmit)
        } else {
            Ok(())
        }
    }

    fn recv_byte(&self, stop: bool) -> Result<u8, Error> {
        // Wait until TX FIFO is empty
        while self.i2c.i2c_status_reg.read().tfe().bit_is_clear() {}

        crate::cm::interrupt::free(|_| {
            // Prepare to transmit the read command byte
            self.i2c.i2c_data_cmd_reg.write(|w| {
                if stop {
                    w.i2c_stop().set_bit();
                }
                w.i2c_cmd().set_bit();
                w
            });
        });

        // Wait for received data
        while self.i2c.i2c_rxflr_reg.read().rxflr().bits() == 0 {}

        let out = self.i2c.i2c_data_cmd_reg.read().dat().bits();

        // Wait until TX FIFO is empty
        while self.i2c.i2c_status_reg.read().tfe().bit_is_clear() {}

        // Wait until master has finished reading the byte from slave device
        while self.i2c.i2c_status_reg.read().mst_activity().bit_is_set() {}

        Ok(out)
    }

    /// Write to an I2C slave.
    fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let buffer_length = buffer.len();

        // Send out all bytes.
        for (idx, byte) in buffer.iter().enumerate() {
            self.send_byte(*byte, (idx + 1) == buffer_length)?;
        }

        Ok(())
    }

    /// Read from an I2C slave.
    fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let buffer_length = buffer.len();

        // Read into buffer.
        for (idx, byte) in buffer.iter_mut().enumerate() {
            self.send_byte(*byte, (idx + 1) == buffer_length)?;
            *byte = self.recv_byte((idx + 1) == buffer_length)?;
        }

        Ok(())
    }

    /// Write data to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two.
    fn write_then_read(&mut self, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Error> {
        self.write(wr_buffer)?;
        self.read(rd_buffer)?;
        Ok(())
    }

    fn set_slave_address(&mut self, address: u16) {
        self.disable_controller();

        // Set Slave I2C address.
        self.i2c
            .i2c_tar_reg
            .modify(|_, w| unsafe { w.ic_tar().bits(address) });

        self.enable_controller();
    }

    fn enable_controller(&mut self) {
        self.i2c
            .i2c_enable_reg
            .modify(|_, w| w.ctrl_enable().set_bit());
        while !self.i2c.i2c_enable_reg.read().ctrl_enable().bit() {}
    }

    fn disable_controller(&mut self) {
        self.i2c
            .i2c_enable_reg
            .modify(|_, w| w.ctrl_enable().clear_bit());
        while self.i2c.i2c_enable_reg.read().ctrl_enable().bit() {}
    }
}

impl embedded_hal::blocking::i2c::Write for I2c {
    type Error = Error;

    fn write<'w>(&mut self, addr: u8, bytes: &'w [u8]) -> Result<(), Error> {
        self.set_slave_address(addr as u16);
        self.write(bytes)
    }
}

impl embedded_hal::blocking::i2c::Read for I2c {
    type Error = Error;

    fn read<'w>(&mut self, addr: u8, bytes: &'w mut [u8]) -> Result<(), Error> {
        self.set_slave_address(addr as u16);
        self.read(bytes)
    }
}

impl embedded_hal::blocking::i2c::WriteRead for I2c {
    type Error = Error;

    fn write_read<'w>(
        &mut self,
        addr: u8,
        bytes: &'w [u8],
        buffer: &'w mut [u8],
    ) -> Result<(), Error> {
        self.set_slave_address(addr as u16);
        self.write_then_read(bytes, buffer)
    }
}

#[derive(Debug)]
pub enum Error {
    Transmit,
    Receive,
}

pub trait Instance: Deref<Target = i2c::RegisterBlock> + sealed::Sealed {}

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for I2C {}
impl Instance for I2C {}

#[no_mangle]
pub unsafe extern "C" fn I2C_Handler() {
    // if let Some(handler) = TIMER0_HANDLER {
    //     handler();Nvic::
    // }
    NVIC::unpend(Irq::I2c);
}

// static mut TIMER0_HANDLER: Option<fn()> = None;
