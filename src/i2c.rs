//! HAL interface to the I2C peripheral.

use core::ops::Deref;

use crate::{
    crg_top::CrgTop,
    gpio::{AfI2cScl, AfI2cSda, Disconnected, Pin},
    pac::{i2c, I2C},
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
    sda: Pin<AfI2cSda>,
    scl: Pin<AfI2cScl>,
}

impl Pins {
    fn new(sda: Pin<Disconnected>, scl: Pin<Disconnected>) -> Self {
        Self {
            scl: scl.into_alternate(),
            sda: sda.into_alternate(),
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
    pub fn set_pins(mut self, sda: Pin<Disconnected>, scl: Pin<Disconnected>) -> Self {
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

    pub fn start(&self, crg_top: &CrgTop) {
        assert!(self.pins.is_some());

        // Enable peripheral clock
        CrgTop::enable_peripheral::<I2C>(&crg_top);

        // Disable the I2C Controller
        self.i2c
            .i2c_enable_reg
            .write(|w| w.ctrl_enable().clear_bit());

        // There is a two ic_clk delay when enabling or disabling the controller
        while self.i2c.i2c_enable_reg.read().ctrl_enable().bit() {}

        // Enable all interrupts
        self.i2c.i2c_intr_mask_reg.write(|w| unsafe { w.bits(0) });

        self.i2c
            .i2c_ss_scl_hcnt_reg
            .write(|w| unsafe { w.bits(0x00000048) });
        self.i2c
            .i2c_ss_scl_lcnt_reg
            .write(|w| unsafe { w.bits(0x0000004F) });

        self.i2c.i2c_con_reg.write(|w| {
            // Configure the speed mode
            unsafe {
                w.i2c_speed().bits(match self.speed {
                    Speed::Standard => 1,
                    Speed::FullSpeed => 2,
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
        self.i2c.i2c_enable_reg.write(|w| w.ctrl_enable().set_bit());

        // There is a two ic_clk delay when enabling or disabling the controller
        while !self.i2c.i2c_enable_reg.read().ctrl_enable().bit() {}
    }

    fn send_byte(&self, byte: u8) -> Result<(), Error> {
        crate::cm::interrupt::free(|_| {
            // Prepare to transmit the write command byte
            self.i2c.i2c_data_cmd_reg.write(|w| {
                w.i2c_cmd().clear_bit();

                unsafe {
                    w.dat().bits(byte);
                }

                w
            });
        });

        // Wait until TX FIFO is empty
        while self.i2c.i2c_status_reg.read().tfe().bit_is_clear() {}

        // Wait until master has finished reading the response byte from slave device
        while self.i2c.i2c_status_reg.read().mst_activity().bit_is_set() {}

        // Read the I2C_TX_ABRT_SOURCE_REG register
        let abort_source = self.i2c.i2c_tx_abrt_source_reg.read().bits();
        if abort_source != 0 {
            self.i2c.i2c_clr_tx_abrt_reg.read().bits();
            Err(Error::Transmit)
        } else {
            Ok(())
        }
    }

    fn recv_byte(&self) -> Result<u8, Error> {
        crate::cm::interrupt::free(|_| {
            // Prepare to transmit the read command byte
            self.i2c.i2c_data_cmd_reg.write(|w| w.i2c_cmd().set_bit());
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

    fn recv_byte_stop(&self) -> Result<u8, Error> {
        crate::cm::interrupt::free(|_| {
            // Prepare to transmit the read command byte
            self.i2c.i2c_data_cmd_reg.write(|w| {
                w.i2c_stop().set_bit();
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

    // TODO: Implement section
    fn send_byte_stop(&self, byte: u8) -> Result<(), Error> {
        //     // Clear stopped event.
        //     self.i2c.events_stopped.write(|w| unsafe { w.bits(0) });

        //     // Start stop condition.
        //     self.i2c.tasks_stop.write(|w| unsafe { w.bits(1) });

        //     // Wait until stop was sent.
        //     while self.i2c.events_stopped.read().bits() == 0 {
        //         // Bail out if we get an error instead.
        //         if self.i2c.events_error.read().bits() != 0 {
        //             self.i2c.events_error.write(|w| unsafe { w.bits(0) });
        //             return Err(Error::Transmit);
        //         }
        //     }

        crate::cm::interrupt::free(|_| {
            // Prepare to transmit the write command byte
            self.i2c.i2c_data_cmd_reg.write(|w| {
                w.i2c_cmd().clear_bit();
                w.i2c_stop().set_bit();

                unsafe {
                    w.dat().bits(byte);
                }

                w
            });
        });

        // Wait until TX FIFO is empty
        while self.i2c.i2c_status_reg.read().tfe().bit_is_clear() {}

        // Wait until master has finished reading the response byte from slave device
        while self.i2c.i2c_status_reg.read().mst_activity().bit_is_set() {}

        // Read the I2C_TX_ABRT_SOURCE_REG register
        let abort_source = self.i2c.i2c_tx_abrt_source_reg.read().bits();
        if abort_source != 0 {
            self.i2c.i2c_clr_tx_abrt_reg.read().bits();
            Err(Error::Transmit)
        } else {
            Ok(())
        }
    }

    /// Write to an I2C slave.
    pub fn write(&mut self, address: u16, buffer: &[u8]) -> Result<(), Error> {
        self.set_slave_address(address);

        // Clock out all bytes.
        if let Some((last, before)) = buffer.split_last() {
            for byte in &mut before.into_iter() {
                self.send_byte(*byte)?;
            }
            self.send_byte_stop(*last)?;
        }

        Ok(())
    }

    /// Read from an I2C slave.
    pub fn read(&mut self, address: u16, buffer: &mut [u8]) -> Result<(), Error> {
        self.set_slave_address(address);

        // Read into buffer.
        if let Some((last, before)) = buffer.split_last_mut() {
            for byte in &mut before.into_iter() {
                *byte = self.recv_byte()?;
            }
            *last = self.recv_byte_stop()?;
        }
        // else {
        //     self.send_stop()?;
        // }
        Ok(())
    }

    /// Write data to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two.
    pub fn write_then_read(
        &mut self,
        address: u16,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), Error> {
        self.set_slave_address(address);

        // Send out all bytes in the outgoing buffer.
        if let Some((last, before)) = wr_buffer.split_last() {
            for byte in before {
                self.send_byte(*byte)?;
            }
            self.send_byte_stop(*last)?;
        }

        // Turn around to read data.
        if let Some((last, before)) = rd_buffer.split_last_mut() {
            for byte in &mut before.into_iter() {
                *byte = self.recv_byte()?;
            }
            *last = self.recv_byte_stop()?;
        }
        // else {
        //     self.send_stop()?;
        // }
        Ok(())
    }

    fn set_slave_address(&mut self, address: u16) {
        self.i2c
            .i2c_enable_reg
            .modify(|_, w| w.ctrl_enable().clear_bit());
        while self.i2c.i2c_enable_reg.read().ctrl_enable().bit() {}

        // Set Slave I2C address.
        self.i2c
            .i2c_tar_reg
            .modify(|_, w| unsafe { w.ic_tar().bits(address) });

        self.i2c
            .i2c_enable_reg
            .modify(|_, w| w.ctrl_enable().set_bit());
        while !self.i2c.i2c_enable_reg.read().ctrl_enable().bit() {}
    }
    // TODO: port this section!
    // /// Return the raw interface to the underlying TWI peripheral.
    // pub fn free(self) -> (T, Pins) {
    //     let scl = self.i2c.pselscl.read();
    //     let sda = self.i2c.pselsda.read();
    //     self.i2c.pselscl.reset();
    //     self.i2c.pselsda.reset();
    //     (
    //         self.i2c,
    //         Pins {
    //             scl: unsafe { Pin::from_psel_bits(scl.bits()) },
    //             sda: unsafe { Pin::from_psel_bits(sda.bits()) },
    //         },
    //     )
    // }
}

impl embedded_hal::blocking::i2c::Write for I2c {
    type Error = Error;

    fn write<'w>(&mut self, addr: u8, bytes: &'w [u8]) -> Result<(), Error> {
        self.write(addr as u16, bytes)
    }
}

impl embedded_hal::blocking::i2c::Read for I2c {
    type Error = Error;

    fn read<'w>(&mut self, addr: u8, bytes: &'w mut [u8]) -> Result<(), Error> {
        self.read(addr as u16, bytes)
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
        self.write_then_read(addr as u16, bytes, buffer)
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
