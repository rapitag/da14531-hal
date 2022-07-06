//! HAL interface to the I2C peripheral.

use core::ops::Deref;

use crate::{
    gpio::{AfI2cScl, AfI2cSda, Disconnected, Pin},
    pac::{i2c, CRG_TOP, I2C},
};

pub enum I2cSpeed {
    /// 100 kbit/s
    Standard,
    /// 400 kbit/s
    FullSpeed,
}

pub enum AddressingMode {
    Bits7,
    Bits10,
}

pub struct I2c<T>(T);

impl<T> I2c<T>
where
    T: Instance,
{
    pub fn new(
        crg_top: &CRG_TOP,
        i2c: T,
        scl: Pin<Disconnected>,
        sda: Pin<Disconnected>,
        speed: I2cSpeed,
        addressing_mode: AddressingMode,
    ) -> Self {
        let _scl: Pin<AfI2cScl> = scl.into_alternate();
        let _sda: Pin<AfI2cSda> = sda.into_alternate();

        cortex_m::interrupt::free(|_| {
            // Enable the clock for the I2C Controller
            crg_top.clk_per_reg.modify(|_, w| w.i2c_enable().set_bit());
        });

        // Disable the I2C Controller
        i2c.i2c_enable_reg.write(|w| w.ctrl_enable().clear_bit());

        // There is a two ic_clk delay when enabling or disabling the controller
        while i2c.i2c_enable_reg.read().ctrl_enable().bit() {}

        // Enable all interrupts
        i2c.i2c_intr_mask_reg.write(|w| unsafe { w.bits(0) });

        i2c.i2c_ss_scl_hcnt_reg
            .write(|w| unsafe { w.bits(0x00000048) });
        i2c.i2c_ss_scl_lcnt_reg
            .write(|w| unsafe { w.bits(0x0000004F) });

        i2c.i2c_con_reg.write(|w| {
            // Configure the speed mode
            unsafe {
                w.i2c_speed().bits(match speed {
                    I2cSpeed::Standard => 1,
                    I2cSpeed::FullSpeed => 2,
                });
            }

            // Setup as I2C master
            w.i2c_master_mode().set_bit();
            w.i2c_slave_disable().set_bit();

            // Configure addressing mode
            match addressing_mode {
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
        i2c.i2c_rx_tl_reg.write(|w| unsafe { w.rx_tl().bits(0) });
        i2c.i2c_tx_tl_reg.write(|w| unsafe { w.rx_tl().bits(0) });

        // Enable the I2C Controller
        i2c.i2c_enable_reg.write(|w| w.ctrl_enable().set_bit());

        // There is a two ic_clk delay when enabling or disabling the controller
        while !i2c.i2c_enable_reg.read().ctrl_enable().bit() {}

        Self(i2c)
    }

    fn send_byte(&self, byte: u8) -> Result<(), Error> {
        cortex_m::interrupt::free(|_| {
            // Prepare to transmit the write command byte
            self.0.i2c_data_cmd_reg.write(|w| {
                w.i2c_cmd().clear_bit();

                unsafe {
                    w.dat().bits(byte);
                }

                w
            });
        });

        // Wait until TX FIFO is empty
        while self.0.i2c_status_reg.read().tfe().bit_is_clear() {}

        // Wait until master has finished reading the response byte from slave device
        while self.0.i2c_status_reg.read().mst_activity().bit_is_set() {}

        // Read the I2C_TX_ABRT_SOURCE_REG register
        let abort_source = self.0.i2c_tx_abrt_source_reg.read().bits();
        if abort_source != 0 {
            self.0.i2c_clr_tx_abrt_reg.read().bits();
            Err(Error::Transmit)
        } else {
            Ok(())
        }
    }

    fn recv_byte(&self) -> Result<u8, Error> {
        cortex_m::interrupt::free(|_| {
            // Prepare to transmit the read command byte
            self.0.i2c_data_cmd_reg.write(|w| w.i2c_cmd().set_bit());
        });

        // Wait for received data
        while self.0.i2c_rxflr_reg.read().rxflr().bits() == 0 {}

        let out = self.0.i2c_data_cmd_reg.read().dat().bits();

        // Wait until TX FIFO is empty
        while self.0.i2c_status_reg.read().tfe().bit_is_clear() {}

        // Wait until master has finished reading the byte from slave device
        while self.0.i2c_status_reg.read().mst_activity().bit_is_set() {}

        Ok(out)
    }

    fn recv_byte_stop(&self) -> Result<u8, Error> {
        cortex_m::interrupt::free(|_| {
            // Prepare to transmit the read command byte
            self.0.i2c_data_cmd_reg.write(|w| {
                w.i2c_stop().set_bit();
                w.i2c_cmd().set_bit();
                w
            });
        });

        // Wait for received data
        while self.0.i2c_rxflr_reg.read().rxflr().bits() == 0 {}

        let out = self.0.i2c_data_cmd_reg.read().dat().bits();

        // Wait until TX FIFO is empty
        while self.0.i2c_status_reg.read().tfe().bit_is_clear() {}

        // Wait until master has finished reading the byte from slave device
        while self.0.i2c_status_reg.read().mst_activity().bit_is_set() {}

        Ok(out)
    }

    // TODO: Implement section
    fn send_byte_stop(&self, byte: u8) -> Result<(), Error> {
        //     // Clear stopped event.
        //     self.0.events_stopped.write(|w| unsafe { w.bits(0) });

        //     // Start stop condition.
        //     self.0.tasks_stop.write(|w| unsafe { w.bits(1) });

        //     // Wait until stop was sent.
        //     while self.0.events_stopped.read().bits() == 0 {
        //         // Bail out if we get an error instead.
        //         if self.0.events_error.read().bits() != 0 {
        //             self.0.events_error.write(|w| unsafe { w.bits(0) });
        //             return Err(Error::Transmit);
        //         }
        //     }

        cortex_m::interrupt::free(|_| {
            // Prepare to transmit the write command byte
            self.0.i2c_data_cmd_reg.write(|w| {
                w.i2c_cmd().clear_bit();
                w.i2c_stop().set_bit();

                unsafe {
                    w.dat().bits(byte);
                }

                w
            });
        });

        // Wait until TX FIFO is empty
        while self.0.i2c_status_reg.read().tfe().bit_is_clear() {}

        // Wait until master has finished reading the response byte from slave device
        while self.0.i2c_status_reg.read().mst_activity().bit_is_set() {}

        // Read the I2C_TX_ABRT_SOURCE_REG register
        let abort_source = self.0.i2c_tx_abrt_source_reg.read().bits();
        if abort_source != 0 {
            self.0.i2c_clr_tx_abrt_reg.read().bits();
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
        self.0
            .i2c_enable_reg
            .modify(|_, w| w.ctrl_enable().clear_bit());
        while self.0.i2c_enable_reg.read().ctrl_enable().bit() {}

        // Set Slave I2C address.
        self.0
            .i2c_tar_reg
            .modify(|_, w| unsafe { w.ic_tar().bits(address) });

        self.0
            .i2c_enable_reg
            .modify(|_, w| w.ctrl_enable().set_bit());
        while !self.0.i2c_enable_reg.read().ctrl_enable().bit() {}
    }
    // TODO: port this section!
    // /// Return the raw interface to the underlying TWI peripheral.
    // pub fn free(self) -> (T, Pins) {
    //     let scl = self.0.pselscl.read();
    //     let sda = self.0.pselsda.read();
    //     self.0.pselscl.reset();
    //     self.0.pselsda.reset();
    //     (
    //         self.0,
    //         Pins {
    //             scl: unsafe { Pin::from_psel_bits(scl.bits()) },
    //             sda: unsafe { Pin::from_psel_bits(sda.bits()) },
    //         },
    //     )
    // }
}

impl<T> embedded_hal::blocking::i2c::Write for I2c<T>
where
    T: Instance,
{
    type Error = Error;

    fn write<'w>(&mut self, addr: u8, bytes: &'w [u8]) -> Result<(), Error> {
        self.write(addr as u16, bytes)
    }
}

impl<T> embedded_hal::blocking::i2c::Read for I2c<T>
where
    T: Instance,
{
    type Error = Error;

    fn read<'w>(&mut self, addr: u8, bytes: &'w mut [u8]) -> Result<(), Error> {
        self.read(addr as u16, bytes)
    }
}

impl<T> embedded_hal::blocking::i2c::WriteRead for I2c<T>
where
    T: Instance,
{
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
