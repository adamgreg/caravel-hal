use anyhow::{Result, ensure};
use embedded_hal::spi::{Operation, SpiDevice};

#[repr(u8)]
enum Cmd {
    Passthru = 0xC4,
    ReadReg = 0x48,
    WriteReg = 0x88,
    ReadStream = 0x40,
    WriteStream = 0x80,
}

#[allow(unused)]
#[repr(u8)]
enum FlashCmd {
    ReadStatusReg = 0x05,
    ReadStatusReg2 = 0x35,
    WriteEnable = 0x06,
    WriteDisable = 0x04,
    ProgramPage = 0x02,
    EnableWriteStatusReg = 0x50,
    WriteStatusReg = 0x01,
    EraseSubsector = 0x20,
    EraseHsector = 0x52,
    EraseSector = 0xD8,
    EraseChip = 0x60,
    ResetChip = 0x99,
    JedecData = 0x9f,
    ReadLoSpeed = 0x03,
    ReadHiSpeed = 0x0B,
}

/// Bitfield definition for the Flash status register
#[bitfield_struct::bitfield(u8)]
pub struct FlashStatus1Bits {
    #[bits(1)]
    pub wip: bool,
    #[bits(1)]
    pub wel: bool,
    #[bits(1)]
    pub bp0: bool,
    #[bits(1)]
    pub bp1: bool,
    #[bits(1)]
    pub bp2: bool,
    #[bits(1)]
    pub bp3: bool,
    #[bits(1)]
    pub sp: bool,
    #[bits(1)]
    pub bpl: bool,
}

/// Perform an SPI write command with the given command and arguments.
macro_rules! write_cmd {
    ($self:expr, $cmd:expr $(, $arg:expr)* $(,)?) => {{
        const CMD: Cmd = $cmd;
        $self.spi.write(&[CMD as u8, $($arg),*])
    }};
}

/// Perform an SPI read command of the given length using the given command and arguments.
macro_rules! read_cmd {
    ($self:expr, $len:literal, $cmd:expr $(, $arg:expr)* $(,)?) => {{
        const CMD: Cmd = $cmd;
        const LEN: usize = $len;
        let mut buf = [0u8; LEN];
        $self.spi
            .transaction(&mut [
                Operation::Write(&[CMD as u8, $($arg),*]),
                Operation::Read(&mut buf),
            ])?;
        buf
    }};
}

// Create an SPI write operation with the given Flash passthru command and arguments.
macro_rules! flash_write_cmd {
    ($self:expr, $flash_cmd:expr $(, $arg:expr)* $(,)?) => {{
        const FLASH_CMD: FlashCmd = $flash_cmd;
        write_cmd!($self, Cmd::Passthru, FLASH_CMD as u8 $(, $arg)*)
    }};
}

// Create an SPI read operation with the given Flash passthru command and arguments.
macro_rules! flash_read_cmd {
    ($self:expr, $len:literal, $flash_cmd:expr $(, $arg:expr)* $(,)?) => {{
        const FLASH_CMD: FlashCmd = $flash_cmd;
        read_cmd!($self, $len, Cmd::Passthru, FLASH_CMD as u8 $(, $arg)*)
    }};
}

/// Driver for the Caravel Housekeeping SPI interface
pub struct HousekeepingSpi<SPI> {
    spi: SPI,
}

impl<SPI> HousekeepingSpi<SPI>
where
    SPI: SpiDevice,
    SPI::Error: core::error::Error + Send + Sync + 'static,
{
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Check manufacturer and product IDs are as expected
    pub fn check_ids(&mut self) -> Result<()> {
        let bytes = read_cmd!(self, 2, Cmd::ReadStream, 0x01);
        let mfg = u16::from_be_bytes(bytes);
        ensure!(
            mfg == 0x0456,
            "Incorrect MFG value ({mfg:#06X}). Expected 0x0456."
        );

        let product = self.read_reg(0x03)?;
        ensure!(
            product == 0x11,
            "Incorrect product value ({product:#04x}). Expected 0x11."
        );

        Ok(())
    }

    pub fn read_reg(&mut self, reg: u8) -> Result<u8> {
        ensure!(
            reg < 0x70,
            "Invalid Housekeeping register address {reg:#04X}"
        );

        Ok(read_cmd!(self, 1, Cmd::ReadReg, reg)[0])
    }

    pub fn write_reg(&mut self, reg: u8, value: u8) -> Result<()> {
        ensure!(
            reg < 0x70,
            "Invalid Housekeeping register address {reg:#04X}"
        );
        Ok(write_cmd!(self, Cmd::WriteReg, reg, value)?)
    }

    pub fn read_project_id(&mut self) -> Result<u32> {
        let bytes = read_cmd!(self, 4, Cmd::ReadStream, 0x04);
        Ok(u32::from_le_bytes([
            bytes[0].reverse_bits(),
            bytes[1].reverse_bits(),
            bytes[2].reverse_bits(),
            bytes[3].reverse_bits(),
        ]))
    }

    pub fn get_flash_status(&mut self) -> Result<FlashStatus1Bits> {
        Ok(flash_read_cmd!(self, 1, FlashCmd::ReadStatusReg)[0].into())
    }

    pub fn get_flash_status2(&mut self) -> Result<u8> {
        Ok(flash_read_cmd!(self, 1, FlashCmd::ReadStatusReg2)[0])
    }

    pub fn flash_busy(&mut self) -> Result<bool> {
        Ok(self.get_flash_status()?.wip())
    }

    pub fn cpu_reset_hold(&mut self) -> Result<()> {
        self.write_reg(0x0B, 1)
    }

    pub fn cpu_reset_release(&mut self) -> Result<()> {
        self.write_reg(0x0B, 0)
    }

    pub fn cpu_reset_toggle(&mut self) -> Result<()> {
        self.cpu_reset_hold()?;
        self.cpu_reset_release()
    }

    pub fn flash_reset(&mut self) -> Result<()> {
        Ok(flash_write_cmd!(self, FlashCmd::ResetChip)?)
    }

    pub fn flash_read_jedec(&mut self) -> Result<[u8; 3]> {
        Ok(flash_read_cmd!(self, 3, FlashCmd::JedecData))
    }

    pub fn flash_identify(&mut self) -> Result<()> {
        let jedec = self.flash_read_jedec()?;
        ensure!(jedec[0] == 0xef, "Winbond Flash not found");
        Ok(())
    }

    pub fn flash_write_enable(&mut self) -> Result<()> {
        Ok(flash_write_cmd!(self, FlashCmd::WriteEnable)?)
    }

    pub fn flash_write_disable(&mut self) -> Result<()> {
        Ok(flash_write_cmd!(self, FlashCmd::WriteDisable)?)
    }

    pub fn flash_erase(&mut self) -> Result<()> {
        self.flash_write_enable()?;
        Ok(flash_write_cmd!(self, FlashCmd::EraseChip)?)
    }

    pub fn flash_erase_sector(&mut self, n_sector: u16) -> Result<()> {
        self.flash_write_enable()?;
        Ok(flash_write_cmd!(
            self,
            FlashCmd::EraseSector,
            ((n_sector >> 4) & 0xFF) as u8,
            ((n_sector << 4) & 0xF0) as u8,
            0
        )?)
    }

    pub fn flash_program_page(&mut self, n_page: u16, data: &[u8]) -> Result<()> {
        self.flash_write_enable()?;
        Ok(self.spi.transaction(&mut [
            Operation::Write(&[
                Cmd::Passthru as u8,
                FlashCmd::ProgramPage as u8,
                ((n_page >> 8) & 0xFF) as u8,
                (n_page & 0xFF) as u8,
                0,
            ]),
            Operation::Write(data),
        ])?)
    }

    pub fn flash_read(&mut self, addr: u32, buf: &mut [u8]) -> Result<()> {
        // Split into multiple transactions if necessary to avoid libusb errors
        let mut read_addr = addr;
        for chunk in buf.chunks_mut(2044) {
            self.spi.transaction(&mut [
                Operation::Write(&[
                    Cmd::Passthru as u8,
                    FlashCmd::ReadLoSpeed as u8,
                    ((read_addr >> 16) & 0xFF) as u8,
                    ((read_addr >> 8) & 0xFF) as u8,
                    (read_addr & 0xFF) as u8,
                ]),
                Operation::Read(chunk),
            ])?;
            read_addr += 2044;
        }
        Ok(())
    }

    pub fn engage_dll(&mut self) -> Result<()> {
        self.write_reg(0x08, 1)?;
        self.write_reg(0x09, 0)
    }

    pub fn read_dll_trim(&mut self) -> Result<u8> {
        let bytes = read_cmd!(self, 4, Cmd::ReadStream, 0x0D);
        // Value should be a kind of thermometer code (0 to 0x3ff_ffff)
        let encoded = u32::from_le_bytes(bytes);
        Ok(encoded.count_ones() as u8)
    }

    pub fn disengage_dll(&mut self) -> Result<()> {
        self.write_reg(0x09, 1)?;
        self.write_reg(0x08, 0)
    }

    pub fn dco_mode(&mut self) -> Result<()> {
        self.write_reg(0x08, 3)?;
        self.write_reg(0x09, 0)
    }

    /// Write the DCO trim value (0 to 26)
    pub fn dco_trim(&mut self, value: u32) -> Result<()> {
        ensure!(value <= 26, "Invalid DCO trim value ({value}). Max is 26.");

        // Value sent as a kind of thermometer code (0 to 0x3ff_ffff)
        let encoded = (1u32 << value) - 1;

        // DCO trim registers are laid out in little-endian order
        let bytes = encoded.to_le_bytes();
        Ok(write_cmd!(
            self,
            Cmd::WriteStream,
            0x0D,
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3]
        )?)
    }
}
