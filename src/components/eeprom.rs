/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use core::borrow::BorrowMut;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;

use card_terminal_adapter::types::*;
use embassy_stm32::crc::Crc;
use embassy_stm32::gpio::OutputOpenDrain; // this can be replaced to Output
use embassy_stm32::i2c::I2c;
use embassy_stm32::peripherals::{self};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Instant, Timer};

use crate::types::fault_log::FaultLog;

// Memory Map - Assume 2KB (16KBits) EEPROM.
// +---------------------------- Memory Map - Assume 2KB (16KBits) EEPROM ---------------------------+
// |                                                                                                 |
// |  Section 0 (0x00-0xFF bytes)   p1_card_cnt    Section =7 (0x600-0x77F bytes) card_reader...     |
// |  +-----------------------------------------+  +-----------------------------------------+       |
// |  | Slot 0  | uptime    | p1_card_cnt | CRC |  | Page 0  | uptime    | lsb               | page0 |
// |  | Slot 1  | uptime    | p1_card_cnt | CRC |  |    card_reader_port_backup (32 bytes)   | page1 |
// |  | ...     | ...       | ...         | ... |  |                       msb         | CRC | page2 |
// |  | Slot 15 | uptime    | p1_card_cnt | CRC |  +--...------------------------------------+       |
// |  +-----------------------------------------+  | Page 0  | uptime    | lsb               | page0 |
// |                                               |    card_reader_port_backup (32 bytes)   | page1 |
// |  Section 1 (0x100-0x1FF bytes) p2_card_cnt    |                               msb | CRC | page2 |
// |  +-----------------------------------------+  +-----------------------------------------+       |
// |  | Slot 0  | uptime    | p2_card_cnt | CRC | <- This section's slot size is single page         |
// |  | Slot 1  | uptime    | p2_card_cnt | CRC |                                                    |
// |  | ...     | ...       | ...         | ... |  Section 0..=3 (normal sections, 16 slot-ish data) |
// |  | Slot 15 | last_time | p2_card_cnt | CRC |  Section  0 : p1_card_cnt          u32    4 bytes  |
// |  +-----------------------------------------+  Section  1 : p1_card_cnt          u32    4 bytes  |
// |   . . . . .                                   Section  2 : p1_coin_cnt          u32    4 bytes  |
// |                                               Section  3 : p2_coin_cnt          u32    4 bytes  |
// |  Section 5 (0x480-0x4FF bytes) hw_boot_cnt                                                      |
// |  +-----------------------------------------+  Section 4..=5 (small sections, 8 slot-ish data)   |
// |  | Slot 0  | uptime    | hw_boot_cnt | CRC |  Section  4 : hw_boot_cnt          u32    4 bytes  |
// |  | ...     | ...       | ...         | ... |  Section  5 : fault_log         Struct    6 bytes  |
// |  | Slot 7  | uptime    | hw_boot_cnt | CRC |                                                    |
// |  +-----------------------------------------+                             2/3 page for slot      |
// |                                               Section 6..=7 (big sections, 8 slot-ish data      |
// |  Section 6 (0x500-0x5FF bytes) raw_terminal   Section  5 : raw_terminal      Struct   13 bytes  |
// |  +-----------------------------------------+                           2 pages for single slot  |
// |  | Slot 0  | uptime    | lsb  raw_terminal |                                                    |
// |  |         raw_terminal          msb | CRC |  Section  6 : card_reader_port_backup    32 bytes  |
// |  +--...------------------------------------+                           3 pages for single slot  |
// |  | Slot 7  | uptime    | lsb  raw_terminal | page0                                              |
// |  |         raw_terminal          msb | CRC | page1                                              |
// |  +-----------------------------------------+                                                    |
// +-------------------------------------------------------------------------------------------------+
//
//   Write cycle endurance of each page is 1,200,000 ~ 4,000,0000
//   Single Page Structure, M24C16's single page size is 16 bytes.
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
//   | 0x0 | 0x1 | 0x2 | 0x3 | 0x4 | 0x5 | 0x6 | 0x7 | 0x8 | 0x9 | 0xA | 0xB | 0xC | 0xD | 0xE | 0xF |
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
//   | uptime   : embassy_time::Duration (inner:u64) |   Actual Data (Max 6 byte-size)   |   CRC16   |
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
//
//   Daul Page Structure, M24C16's each single page size is 16 bytes.
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
//   | 0x0 | 0x1 | 0x2 | 0x3 | 0x4 | 0x5 | 0x6 | 0x7 | 0x8 | 0x9 | 0xA | 0xB | 0xC | 0xD | 0xE | 0xF |
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
//   | uptime   : embassy_time::Duration (inner:u64) |                 Actual Data                   |
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
//   |                   (Max 6+14 = 20 byte-size)    Actual Data                        |   CRC16   |
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
//
//   Triple Page Structure, M24C16's each single page size is 16 bytes.
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
//   | 0x0 | 0x1 | 0x2 | 0x3 | 0x4 | 0x5 | 0x6 | 0x7 | 0x8 | 0x9 | 0xA | 0xB | 0xC | 0xD | 0xE | 0xF |
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
//   | uptime   : embassy_time::Duration (inner:u64) |                 Actual Data                   |
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
//   |                          (Max 6+16+14 = 36 byte-size)    Actual Data                          |
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
//   |                                    Actual Data                                    |   CRC16   |
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+

pub struct MemStorage {
    pub p1_card_cnt: u32,
    pub p2_card_cnt: u32,
    pub p1_coin_cnt: u32,
    pub p2_coin_cnt: u32,
    pub hw_boot_cnt: u32,
    pub fault_log: FaultLog,
    pub raw_terminal: RawTerminalId,
    pub card_reader_port_backup: CardReaderPortBackup,
}

/// Tiny control block for manage single section, it include what page is longest and is dirty state
/// +-----+-----+-----+-----+-----+-----+-----+-----+
/// | b7 |  b6 |  b5 |  b4 |  b3 |  b2 |  b1 |  b0 |
/// +-----+-----+-----+-----+-----+-----+-----+-----+
/// |dirty|  robin: number of what page is longest   |
/// +-----+-----+-----+-----+-----+-----+-----+-----+
pub struct NovellaSectionControlBlock {
    inner: u8,
}

pub type RawNvRobin = u8;
pub type RawRomAddress = u16;
pub type EepromAddress = u8;
pub type DevSelAddress = u8;
type Checksum = u16;

#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(unused)]
pub enum NvMemSectionKind {
    P1CardCnt = 0,      // 1*16, u32
    P2CardCnt = 1,      // 1*16, u32
    P1CoinCnt = 2,      // 1*16, u32
    P2CoinCnt = 3,      // 1*16, u32
    FaultLog = 4,       // 1*16, Not determined 6 bytes
    HwBootCount = 5,    // 1*08, u32
    TerminalId = 6,     // 2*08, 13 bytes
    CardPortBackup = 7, // 3*08, 32 bytes (4+4)*4
}

impl From<u8> for NvMemSectionKind {
    // instead of FromPrimitive for reduce dependancy
    fn from(value: u8) -> Self {
        unsafe {
            // Future rust will support `core::mem::variant_count::<Self>() as u8`
            if value >= SECTION_NUM as u8 {
                assert!(
                    value >= SECTION_NUM as u8,
                    "value should be less then SECTION_NUMBER"
                );
            }
            core::mem::transmute(value)
        }
    }
}

#[derive(Clone, Copy)]
pub struct NovellaSelector<T> {
    pub section: NvMemSectionKind,
    pub marker: PhantomData<T>, // 0-byte guarantees
}

// #[async_trait]
pub trait NovellaRw {
    type InnerType: Sized;
    async fn lock_read(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, NovellaModuleControlBlock>,
    ) -> Self::InnerType;

    async fn lock_write(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, NovellaModuleControlBlock>,
        src: Self::InnerType,
    );
}

fn should_not_happen() -> ! {
    panic!("should not be happens")
}

// this should be generated by macro
impl NovellaRw for NovellaSelector<u32> {
    type InnerType = u32;
    async fn lock_read(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, NovellaModuleControlBlock>,
    ) -> Self::InnerType {
        let cb = mutex.lock().await;

        match self.section {
            NvMemSectionKind::P1CardCnt => cb.data.p1_card_cnt,
            NvMemSectionKind::P2CardCnt => cb.data.p2_card_cnt,
            NvMemSectionKind::P1CoinCnt => cb.data.p1_coin_cnt,
            NvMemSectionKind::P2CoinCnt => cb.data.p2_coin_cnt,
            NvMemSectionKind::HwBootCount => cb.data.hw_boot_cnt,
            _ => {
                should_not_happen();
            }
        }
    }

    async fn lock_write(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, NovellaModuleControlBlock>,
        src: Self::InnerType,
    ) {
        let mut cb = mutex.lock().await;

        match self.section {
            NvMemSectionKind::P1CardCnt => {
                cb.data.p1_card_cnt = src;
            }
            NvMemSectionKind::P2CardCnt => {
                cb.data.p2_card_cnt = src;
            }
            NvMemSectionKind::P1CoinCnt => {
                cb.data.p1_coin_cnt = src;
            }
            NvMemSectionKind::P2CoinCnt => {
                cb.data.p2_coin_cnt = src;
            }
            NvMemSectionKind::HwBootCount => {
                cb.data.hw_boot_cnt = src;
            }
            _ => {
                should_not_happen();
            }
        };

        cb.control_mut(self.section).set_dirty();
    }
}

impl NovellaRw for NovellaSelector<FaultLog> {
    type InnerType = FaultLog;
    async fn lock_read(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, NovellaModuleControlBlock>,
    ) -> Self::InnerType {
        let cb = mutex.lock().await;

        match self.section {
            NvMemSectionKind::FaultLog => cb.data.fault_log.clone(),
            _ => {
                should_not_happen();
            }
        }
    }

    async fn lock_write(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, NovellaModuleControlBlock>,
        src: Self::InnerType,
    ) {
        let mut cb = mutex.lock().await;

        match self.section {
            NvMemSectionKind::FaultLog => {
                cb.data.fault_log = src;
            }
            _ => {
                should_not_happen();
            }
        }

        cb.control_mut(self.section).set_dirty();
    }
}

impl NovellaRw for NovellaSelector<RawTerminalId> {
    type InnerType = RawTerminalId;
    async fn lock_read(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, NovellaModuleControlBlock>,
    ) -> Self::InnerType {
        let cb = mutex.lock().await;

        match self.section {
            NvMemSectionKind::TerminalId => cb.data.raw_terminal.clone(),
            _ => {
                should_not_happen();
            }
        }
    }

    async fn lock_write(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, NovellaModuleControlBlock>,
        src: Self::InnerType,
    ) {
        let mut cb = mutex.lock().await;

        match self.section {
            NvMemSectionKind::FaultLog => {
                cb.data.raw_terminal = src;
            }
            _ => {
                should_not_happen();
            }
        };

        cb.control_mut(self.section).set_dirty();
    }
}

impl NovellaRw for NovellaSelector<CardReaderPortBackup> {
    type InnerType = CardReaderPortBackup;
    async fn lock_read(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, NovellaModuleControlBlock>,
    ) -> Self::InnerType {
        let cb = mutex.lock().await;

        match self.section {
            NvMemSectionKind::CardPortBackup => cb.data.card_reader_port_backup.clone(),
            _ => {
                should_not_happen();
            }
        }
    }

    async fn lock_write(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, NovellaModuleControlBlock>,
        src: Self::InnerType,
    ) {
        let mut cb = mutex.lock().await;

        match self.section {
            NvMemSectionKind::CardPortBackup => {
                cb.data.card_reader_port_backup = src;
            }
            _ => {
                should_not_happen();
            }
        };

        cb.control_mut(self.section).set_dirty();
    }
}

impl NovellaSectionControlBlock {
    fn set_dirty(&mut self) {
        self.inner |= 1 << 7;
    }

    fn clr_dirty(&mut self) {
        self.inner &= !(1 << 7);
    }

    fn is_dirty(&self) -> bool {
        (self.inner & (1 << 7)) != 0
    }

    fn set_robin(&mut self, robin: RawNvRobin) {
        self.inner = (self.inner & (1 << 7)) | (robin & !(1 << 7))
    }

    fn test_and_robin(&mut self, section_info: &NvSectionInfo) -> Option<RawNvRobin> {
        if !self.is_dirty() {
            None
        } else {
            let ret = (section_info.slot_size - 1) & (self.inner + 1);
            self.inner = ret | (1 << 7);
            self.set_dirty();
            Some(ret)
        }
    }
}

// this should be store as const type
#[repr(C)]
struct NvSectionInfo {
    sect_start_page: u16, // sometimes it's u16, real_offset / 128 (r>>7)
    slot_num: u8,
    slot_size: u8,
    real_data_size: u8,
}

#[rustfmt::skip]
const SECTION_TABLE: [NvSectionInfo; 8] = [
    NvSectionInfo{sect_start_page :    0, slot_num : 16, slot_size : 1, real_data_size :  4 },
    NvSectionInfo{sect_start_page :  256, slot_num : 16, slot_size : 1, real_data_size :  4 },
    NvSectionInfo{sect_start_page :  512, slot_num : 16, slot_size : 1, real_data_size :  4 },
    NvSectionInfo{sect_start_page :  768, slot_num : 16, slot_size : 1, real_data_size :  4 },
    NvSectionInfo{sect_start_page : 1024, slot_num :  8, slot_size : 1, real_data_size :  6 },
    NvSectionInfo{sect_start_page : 1152, slot_num :  8, slot_size : 1, real_data_size :  4 },
    NvSectionInfo{sect_start_page : 1280, slot_num :  8, slot_size : 2, real_data_size : 13 },
    NvSectionInfo{sect_start_page : 1536, slot_num :  8, slot_size : 3, real_data_size : 32 },
];

const PAGE_SIZE: usize = 16;
const SECTION_NUM: usize = 8;
const ROM_7B_ADDRESS: u8 = 0b1010000; // Embassy require 7bits address as parameter.
                                      // const ROM_ADDRESS_FIELD_SIZE: usize = core::mem::size_of::<u8>();
const WAIT_DURATION_PER_PAGE: Duration = Duration::from_millis(20); // heuristic value
const CHECKSUM_SIZE: usize = core::mem::size_of::<Checksum>();
const UPTIME_SIZE: usize = core::mem::size_of::<Duration>();
const TOTAL_SLOT_NUM: usize = 96; // should be calculated in compile time
const TOTAL_SLOT_ARR_LEN: usize =
    (TOTAL_SLOT_NUM + core::mem::size_of::<u8>() - 1) / (core::mem::size_of::<u8>() * 8);

static mut I2C_WAIT_UNTIL: Instant = Instant::from_ticks(0);

fn novella_i2c_polling_check_timeout() -> Result<(), embassy_stm32::i2c::Error> {
    if Instant::now() < unsafe { I2C_WAIT_UNTIL } {
        Ok(())
    } else {
        Err(embassy_stm32::i2c::Error::Timeout)
    }
}

fn novella_i2c_polling_set_timeout(duration: Duration) {
    unsafe {
        I2C_WAIT_UNTIL = Instant::now() + duration;
    }
}

fn novella_i2c_polling_set_default_timeout() {
    novella_i2c_polling_set_timeout(WAIT_DURATION_PER_PAGE);
}

pub struct NovellaModuleControlBlock {
    data: MemStorage,
    controls: [NovellaSectionControlBlock; SECTION_NUM],
    uptime: Duration,
}

impl NovellaModuleControlBlock {
    const fn const_default() -> Self {
        // promise default is zero-filled and call `Self::default` later.
        #[allow(invalid_value)]
        unsafe {
            MaybeUninit::uninit().assume_init()
        }
    }

    unsafe fn get_data_raw_slice(&mut self, kind: NvMemSectionKind) -> &mut [u8] {
        match kind {
            NvMemSectionKind::P1CardCnt => core::slice::from_raw_parts_mut(
                (self.data.p1_card_cnt.borrow_mut() as *mut _) as *mut u8,
                core::mem::size_of_val(&self.data.p1_card_cnt),
            ),
            NvMemSectionKind::P2CardCnt => core::slice::from_raw_parts_mut(
                (self.data.p2_card_cnt.borrow_mut() as *mut _) as *mut u8,
                core::mem::size_of_val(&self.data.p2_card_cnt),
            ),
            NvMemSectionKind::P1CoinCnt => core::slice::from_raw_parts_mut(
                (self.data.p1_coin_cnt.borrow_mut() as *mut _) as *mut u8,
                core::mem::size_of_val(&self.data.p1_coin_cnt),
            ),
            NvMemSectionKind::P2CoinCnt => core::slice::from_raw_parts_mut(
                (self.data.p2_coin_cnt.borrow_mut() as *mut _) as *mut u8,
                core::mem::size_of_val(&self.data.p2_coin_cnt),
            ),
            NvMemSectionKind::FaultLog => core::slice::from_raw_parts_mut(
                (self.data.fault_log.borrow_mut() as *mut _) as *mut u8,
                core::mem::size_of_val(&self.data.fault_log),
            ),
            NvMemSectionKind::HwBootCount => core::slice::from_raw_parts_mut(
                (self.data.hw_boot_cnt.borrow_mut() as *mut _) as *mut u8,
                core::mem::size_of_val(&self.data.hw_boot_cnt),
            ),
            NvMemSectionKind::TerminalId => core::slice::from_raw_parts_mut(
                (self.data.raw_terminal.borrow_mut() as *mut _) as *mut u8,
                core::mem::size_of_val(&self.data.raw_terminal),
            ),
            NvMemSectionKind::CardPortBackup => core::slice::from_raw_parts_mut(
                (self.data.card_reader_port_backup.borrow_mut() as *mut _) as *mut u8,
                core::mem::size_of_val(&self.data.card_reader_port_backup),
            ),
        }
    }

    fn control_mut(&mut self, kind: NvMemSectionKind) -> &mut NovellaSectionControlBlock {
        // It's fine with get_unchecked_mut, instead of `get_mut(...) -> Option<I>`.
        // KindT enum is directly limited on element number of internnaly array.
        unsafe { self.controls.get_unchecked_mut(kind as usize) }
    }
}

#[derive(Debug)]
pub enum NovellaInitOk {
    /// Success for all slots
    Success(Duration),
    /// Partially successd, second parameter is fail count
    PartialSucess(Duration, usize),
    /// Assume first boot for this hardware
    FirstBoot,
}

#[derive(Debug)]
pub enum NovellaInitError {
    FirstBoot,
    MissingEeprom,
    // FaultChecksum,
}

#[derive(PartialEq)]
pub enum NovellaReadError {
    FaultChecksum,
    MissingEeprom,
    Unknown,
}

#[derive(PartialEq)]
pub enum NovellaWriteError {
    Wearout,
    MissingEeprom,
    Unknown,
}

pub struct Novella {
    bus: UnsafeCell<I2c<'static, peripherals::I2C1, peripherals::DMA1_CH4, peripherals::DMA1_CH3>>,
    nwp: UnsafeCell<OutputOpenDrain<'static, peripherals::PF0>>,
    crc: UnsafeCell<Crc<'static>>, // crc will be mutexed for reuse HwConfig
    buffer: UnsafeCell<[u8; PAGE_SIZE + core::mem::size_of::<EepromAddress>()]>,
    mem_storage: Mutex<ThreadModeRawMutex, NovellaModuleControlBlock>,
}

#[allow(unused)]
impl Novella {
    /// const_new for hardware initialization
    pub const fn const_new(
        i2c: I2c<'static, peripherals::I2C1, peripherals::DMA1_CH4, peripherals::DMA1_CH3>,
        crc: Crc<'static>,
        nwp: OutputOpenDrain<'static, peripherals::PF0>,
    ) -> Self {
        Self {
            bus: UnsafeCell::new(i2c),
            crc: UnsafeCell::new(crc),
            nwp: UnsafeCell::new(nwp),
            buffer: UnsafeCell::new([0u8; PAGE_SIZE + core::mem::size_of::<EepromAddress>()]),
            mem_storage: Mutex::new(NovellaModuleControlBlock::const_default()),
        }
    }

    pub async fn lock_read<R>(&self, slot: R) -> R::InnerType
    where
        R: NovellaRw,
    {
        slot.lock_read(&self.mem_storage).await
    }

    pub async fn lock_write<R>(&self, slot: R, src: R::InnerType)
    where
        R: NovellaRw,
    {
        slot.lock_write(&self.mem_storage, src).await
    }

    #[inline]
    fn consider_initial_uptime(page_idx: u8) -> bool {
        page_idx == 0
    }

    #[inline]
    fn consider_tailing_checksum(slot_size: u8, page_idx: u8) -> bool {
        slot_size == (page_idx + 1)
    }

    fn get_raw_addr(sect_idx: usize, slot_idx: u8, page_idx: u8) -> RawRomAddress {
        SECTION_TABLE[sect_idx].sect_start_page
            + (SECTION_TABLE[sect_idx].slot_size as RawRomAddress * slot_idx as RawRomAddress
                + page_idx as RawRomAddress)
                * PAGE_SIZE as RawRomAddress
    }

    fn set_write_protect(&self) {
        let nwp = unsafe { &mut *self.nwp.get() };
        nwp.set_high();
    }

    fn clr_write_protect(&self) {
        let nwp = unsafe { &mut *self.nwp.get() };
        nwp.set_low();
    }

    /// data is stored in MemStorage's correspond member variable.
    async fn raw_slot_read(
        &self,
        kind: NvMemSectionKind,
        slot_idx: u8,
    ) -> Result<Duration, NovellaReadError> {
        let bus = unsafe { &mut *self.bus.get() };
        let crc = unsafe { &mut *self.crc.get() };
        let rx_buffer: &mut [u8] = unsafe {
            let buffer = &mut *self.buffer.get();
            // &mut buffer[core::mem::size_of::<EepromAddress>()..]
            // best is using upper code, but care `align` for 32bit processor
            &mut buffer[..PAGE_SIZE]
        };

        crc.reset();

        let sect_idx: usize = kind as u8 as usize;
        let slot_size = SECTION_TABLE[sect_idx].slot_size;

        assert!(SECTION_TABLE[sect_idx].slot_num > slot_idx); // should not happens

        let mut cb = self.mem_storage.lock().await;
        // <- MUTEX SECTION START FROM HERE ->

        let slot_mem = unsafe { cb.get_data_raw_slice(kind) };

        let mut real_data_left = SECTION_TABLE[sect_idx].real_data_size as usize;
        let mut slot_uptime = Duration::from_ticks(0);
        let mut checksum_expected: u16 = 0;

        for page_idx in 0..slot_size {
            // #[cfg(i2c_addr_bits_include_msb)]
            let raw_addr = Self::get_raw_addr(sect_idx, slot_size, page_idx);

            let data_address_slice = (raw_addr as EepromAddress).to_be_bytes();
            let i2c_address = ROM_7B_ADDRESS | ((raw_addr >> 8) as DevSelAddress & 0x7);

            novella_i2c_polling_set_default_timeout(); // preinit for blocking_write_read_timeout

            // blocking function can detect NACK, but async type does not
            let result = bus
                .blocking_write_read_timeout(
                    i2c_address,
                    &data_address_slice,
                    rx_buffer,
                    novella_i2c_polling_check_timeout,
                )
                // .await
                .map_err(|e| match e {
                    embassy_stm32::i2c::Error::Timeout | embassy_stm32::i2c::Error::Nack => {
                        NovellaReadError::MissingEeprom
                    }
                    _ => NovellaReadError::Unknown,
                })?;

            if Self::consider_initial_uptime(page_idx) {
                // Grab time::Duration of slot
                unsafe {
                    slot_uptime = (rx_buffer[0..UPTIME_SIZE].as_ptr() as *const Duration).read();

                    checksum_expected = crc.feed_words(core::slice::from_raw_parts(
                        (rx_buffer as *const _) as *const u32,
                        core::mem::size_of::<Duration>() / core::mem::size_of::<u32>(),
                    )) as Checksum;
                }
            }

            // copy [slot_real_data_reads..MAX(..)]
            let start_read = Self::consider_initial_uptime(page_idx) as usize * UPTIME_SIZE;
            let max_real_data_in_page: usize = PAGE_SIZE
                - start_read
                - (Self::consider_tailing_checksum(slot_size, page_idx) as usize * CHECKSUM_SIZE);

            assert!((max_real_data_in_page <= PAGE_SIZE)); // need compile time assertion

            let size_can_read = max_real_data_in_page.min(real_data_left as usize);

            let slot_mem_start = SECTION_TABLE[sect_idx].real_data_size as usize - real_data_left;
            let dst = &mut slot_mem[slot_mem_start..slot_mem_start + size_can_read];
            let src = &rx_buffer[start_read..start_read + size_can_read];

            dst.copy_from_slice(src);

            checksum_expected = crc.feed_bytes(src) as Checksum;

            real_data_left -= size_can_read;
        }

        // at the end, last buffer filled should contain Checksum.
        let checksum_given = unsafe {
            (rx_buffer[PAGE_SIZE - CHECKSUM_SIZE..PAGE_SIZE].as_ptr() as *const Checksum).read()
        };

        if checksum_given == checksum_expected {
            Ok(slot_uptime)
        } else {
            Err(NovellaReadError::FaultChecksum)
        }
        // <- MUTEX SECTION END HERE ->
    }

    async fn raw_slot_write(
        &self,
        kind: NvMemSectionKind,
        slot_idx: u8,
        uptime: Duration,
    ) -> Result<(), NovellaWriteError> {
        let bus = unsafe { &mut *self.bus.get() };
        let crc = unsafe { &mut *self.crc.get() };
        let mut addr_buffer =
            unsafe { &mut (&mut *self.buffer.get())[..core::mem::size_of::<EepromAddress>()] };
        let mut data_buffer =
            unsafe { &mut (&mut *self.buffer.get())[core::mem::size_of::<EepromAddress>()..] };

        let sect_idx: usize = kind as u8 as usize;
        let slot_size = SECTION_TABLE[sect_idx].slot_size;

        assert!(SECTION_TABLE[sect_idx].slot_num > slot_idx); // should not happens

        let mut cb = self.mem_storage.lock().await;
        // <- MUTEX SECTION START FROM HERE ->
        // JUST COPIED
        let mut real_data_left = SECTION_TABLE[sect_idx].real_data_size as usize;
        let slot_mem = unsafe { cb.get_data_raw_slice(kind) };

        let mut checksum_expected: Checksum = 0;
        crc.reset();

        // Write Oeration
        for page_idx in 0..slot_size {
            data_buffer.fill(0xFF);

            // Set eeprom data address
            // #[cfg(i2c_addr_bits_include_msb)]

            let raw_addr = Self::get_raw_addr(sect_idx, slot_size, page_idx);

            addr_buffer.copy_from_slice(&((raw_addr & 0xFF) as u8).to_be_bytes());

            if Self::consider_initial_uptime(page_idx) {
                // Grab time::Duration of slot
                unsafe {
                    // copy for first slice
                    // To avoid mem align issue, just copy 1 by 1.
                    data_buffer[0..UPTIME_SIZE].copy_from_slice(core::slice::from_raw_parts(
                        &uptime as *const _ as *const u8,
                        core::mem::size_of_val(&uptime),
                    ));

                    let words: &[u32] = core::slice::from_raw_parts(
                        (&uptime as *const _) as *const u32,
                        core::mem::size_of::<Duration>() / core::mem::size_of::<u32>(),
                    );

                    // checksum_expected = crc.feed_words(&words) as Checksum;

                    checksum_expected = crc.feed_bytes(&data_buffer[0..UPTIME_SIZE]) as Checksum;
                }
            }

            let start_write = Self::consider_initial_uptime(page_idx) as usize * UPTIME_SIZE;
            let max_real_data_in_page: usize = PAGE_SIZE
                - start_write
                - (Self::consider_tailing_checksum(slot_size, page_idx) as usize * CHECKSUM_SIZE);

            // assert!((max_real_data_in_page > PAGE_SIZE)); // need compile time assertion

            let size_can_write = max_real_data_in_page.min(real_data_left as usize);

            let slot_mem_start = SECTION_TABLE[sect_idx].real_data_size as usize - real_data_left;
            let dst = &mut data_buffer[start_write..start_write + size_can_write];
            let src = &slot_mem[slot_mem_start..slot_mem_start + size_can_write];

            dst.copy_from_slice(src);

            checksum_expected = crc.feed_bytes(src) as Checksum;

            if Self::consider_tailing_checksum(slot_size, page_idx) {
                unsafe {
                    // To avoid mem align issue, just copy 1 by 1.
                    data_buffer[PAGE_SIZE - CHECKSUM_SIZE..PAGE_SIZE].copy_from_slice(
                        core::slice::from_raw_parts(
                            &checksum_expected as *const _ as *const u8,
                            core::mem::size_of_val(&checksum_expected),
                        ),
                    );
                }
            }

            // #[cfg(i2c_addr_bits_include_msb)]
            let i2c_address = ROM_7B_ADDRESS | ((raw_addr >> 8) as DevSelAddress & 0x7);

            self.clr_write_protect();

            novella_i2c_polling_set_default_timeout();

            let result = bus
                .write(
                    i2c_address,
                    unsafe { &*self.buffer.get() }, // when page is 16, include 1+16 byte will be tx.
                                                    // novella_i2c_polling_check_timeout,
                )
                .await;
            // error handling

            real_data_left -= size_can_write;

            self.set_write_protect();
            Timer::after(Duration::from_millis(5)).await;
        }

        // Read for check, do not copy value to MemStorage
        let mut checksum_double_expected: Checksum = 0;

        // change buffer address for align issue
        let mut addr_buffer = unsafe { &mut (&mut *self.buffer.get())[PAGE_SIZE..] };
        let mut data_buffer = unsafe { &mut (&mut *self.buffer.get())[..PAGE_SIZE] };
        let mut real_data_left = SECTION_TABLE[sect_idx].real_data_size as usize;
        crc.reset();

        for page_idx in 0..slot_size {
            // Set eeprom data address
            // #[cfg(i2c_addr_bits_include_msb)]

            let raw_addr = Self::get_raw_addr(sect_idx, slot_size, page_idx);

            addr_buffer.copy_from_slice(&((raw_addr & 0xFF) as u8).to_be_bytes());

            // #[cfg(i2c_addr_bits_include_msb)]
            let i2c_address = ROM_7B_ADDRESS | ((raw_addr >> 8) as DevSelAddress & 0x7);

            novella_i2c_polling_set_default_timeout(); // preinit for blocking_write_read_timeout

            bus.write_read_timeout(
                i2c_address,
                addr_buffer,
                data_buffer,
                novella_i2c_polling_check_timeout,
            )
            .await
            .map_err(|e| match e {
                embassy_stm32::i2c::Error::Timeout => NovellaWriteError::MissingEeprom,
                _ => NovellaWriteError::Unknown,
            })?;

            if Self::consider_initial_uptime(page_idx) {
                // Grab time::Duration of slot
                let uptime_words: &[u32] = unsafe {
                    core::slice::from_raw_parts(
                        (&data_buffer[0..UPTIME_SIZE] as *const _) as *const u32,
                        core::mem::size_of_val(&uptime) / core::mem::size_of::<u32>(),
                    )
                };
                let uptime_given: &[u32] = unsafe {
                    core::slice::from_raw_parts(
                        (&uptime as *const _) as *const u32,
                        core::mem::size_of_val(&uptime) / core::mem::size_of::<u32>(),
                    )
                };

                if *uptime_given != *uptime_words {
                    return Err(NovellaWriteError::Wearout);
                }

                checksum_double_expected = crc.feed_bytes(&data_buffer[0..UPTIME_SIZE]) as Checksum;
            }

            // copy [slot_real_data_reads..MAX(..)]
            let start_read = Self::consider_initial_uptime(page_idx) as usize * UPTIME_SIZE;
            let max_real_data_in_page: usize = PAGE_SIZE
                - start_read
                - (Self::consider_tailing_checksum(slot_size, page_idx) as usize * CHECKSUM_SIZE);

            assert!((max_real_data_in_page <= PAGE_SIZE)); // need compile time assertion

            let size_can_read = max_real_data_in_page.min(real_data_left as usize);

            let slot_mem_start = SECTION_TABLE[sect_idx].real_data_size as usize - real_data_left;
            let src = &data_buffer[start_read..start_read + size_can_read];

            checksum_double_expected = crc.feed_bytes(src) as Checksum;

            real_data_left -= size_can_read;
        }

        // at the end, last buffer filled should contain Checksum.
        let checksum_given = unsafe {
            (data_buffer[PAGE_SIZE - CHECKSUM_SIZE..PAGE_SIZE].as_ptr() as *const Checksum).read()
        };

        if (checksum_expected != checksum_given) || (checksum_expected != checksum_double_expected)
        {
            Err(NovellaWriteError::Wearout)
        } else {
            Ok(())
        }
        // <- MUTEX SECTION END HERE ->
    }

    /// when success return marked last time
    /// Success to detect eeprom but it's filled in 0xFF or 0xFF are initial factory value, return NovellaInitError::FirstBoot
    /// initialization is not using async/await for safety
    pub async fn init(&self) -> Result<NovellaInitOk, NovellaInitError> {
        #[inline]
        fn consider_initial_uptime(page_idx: u8) -> bool {
            page_idx == 0
        }

        #[inline]
        fn consider_tailing_checksum(slot_size: u8, page_idx: u8) -> bool {
            slot_size == page_idx - 1
        }

        let bus = unsafe { &mut *self.bus.get() };
        let crc = unsafe { &mut *self.crc.get() };
        let buffer: &mut [u8] = unsafe { &mut *self.buffer.get() };
        let mut broken_map_idx = 0;
        let mut broken_map = [0u8; TOTAL_SLOT_ARR_LEN];
        let mut longest = Duration::from_ticks(0); // guarantees smallest
        let mut broken_detected = 0;

        self.set_write_protect();
        crc.reset();

        for sect_idx in 0..SECTION_TABLE.len() {
            let kind = NvMemSectionKind::from(sect_idx as u8);
            let (mut longest_per_sector, mut longest_slot) = (Duration::from_ticks(0), None);

            for slot_idx in 0..SECTION_TABLE[sect_idx].slot_num {
                match self.raw_slot_read(kind, slot_idx).await {
                    Ok(uptime) => {
                        if longest_per_sector <= uptime {
                            longest_per_sector = uptime;
                            longest_slot = Some(slot_idx);
                        }

                        if longest < uptime {
                            longest = uptime;
                        }
                    }
                    Err(NovellaReadError::FaultChecksum) | Err(NovellaReadError::Unknown) => {
                        broken_map[broken_map_idx >> 3] |= 1 << (broken_map_idx & 0x7);
                        broken_detected += 1;
                    }
                    Err(NovellaReadError::MissingEeprom) => {
                        return Err(NovellaInitError::MissingEeprom);
                    }
                }

                broken_map_idx += 1;
            }

            let final_slot = if let Some(longest_slot) = longest_slot {
                // read again for fill longest value on MemStorage.
                self.raw_slot_read(kind, longest_slot).await; //<- Error handling here.
                longest_slot
            } else {
                unsafe {
                    self.mem_storage
                        .lock()
                        .await
                        .get_data_raw_slice(kind)
                        .fill(0x00);
                }
                0
            };
            let mut mem = self.mem_storage.lock().await;
            mem.controls[sect_idx].set_robin(final_slot);
            mem.controls[sect_idx].clr_dirty();
        }

        // Rerun for-loop for following reason,
        // Before run second loop, need longest uptime from whole slots in all sections,
        // and longest index in each section.

        if broken_detected != 0 {
            defmt::error!(
                "Broken cell detected, try healing. broken : {}, broken_map : {:#b}",
                broken_detected,
                broken_map
            );
        }

        broken_map_idx = 0;
        for sect_idx in 0..SECTION_TABLE.len() {
            let kind = NvMemSectionKind::from(sect_idx as u8);

            for slot_idx in 0..SECTION_TABLE[sect_idx].slot_num {
                if (broken_map[broken_map_idx >> 3] & (1 << (broken_map_idx & 0x7))) != 0 {
                    defmt::debug!(
                        "Fixing broken slot... [{}], sect = {}, slot = {}",
                        broken_map_idx,
                        sect_idx,
                        slot_idx
                    );

                    let renew_uptime =
                        Duration::from_ticks(longest.as_ticks() + Instant::now().as_ticks());

                    if self.raw_slot_write(kind, slot_idx, renew_uptime).await
                        == Err(NovellaWriteError::MissingEeprom)
                    {
                        return Err(NovellaInitError::MissingEeprom);
                    }
                }

                broken_map_idx += 1;
            }
        }

        match broken_detected {
            0 => {
                self.mem_storage.lock().await.uptime = Duration::from_ticks(longest.as_ticks());
                Ok(NovellaInitOk::Success(longest))
            }
            TOTAL_SLOT_NUM => {
                self.mem_storage.lock().await.uptime = Duration::from_ticks(0);
                Ok(NovellaInitOk::FirstBoot)
            }
            x => {
                self.mem_storage.lock().await.uptime = Duration::from_ticks(longest.as_ticks());
                Ok(NovellaInitOk::PartialSucess(longest, broken_detected))
            }
        }
    }

    /// Get uptime of this board
    pub async fn get_uptime(&self) -> Duration {
        Duration::from_ticks(
            self.mem_storage.lock().await.uptime.as_ticks() + Instant::now().as_ticks(),
        )
    }

    async fn run(&self) {
        loop {
            for sect_idx in 0..SECTION_TABLE.len() {
                let kind = NvMemSectionKind::from(sect_idx as u8);

                let dirty_or_next_slot = self.mem_storage.lock().await.controls[sect_idx]
                    .test_and_robin(&SECTION_TABLE[sect_idx]);

                if let Some(next_slot) = dirty_or_next_slot {
                    let new_uptime = self.get_uptime().await;

                    defmt::debug!(
                        "EEPROM write on [{:02}][{:02}], ticks : {}",
                        sect_idx,
                        next_slot,
                        new_uptime
                    );

                    match self.raw_slot_write(kind, next_slot, new_uptime).await {
                        Ok(_) => {
                            self.mem_storage.lock().await.controls[sect_idx].clr_dirty();
                        }
                        Err(NovellaWriteError::MissingEeprom) => {
                            self.mem_storage.lock().await.controls[sect_idx].clr_dirty();
                            defmt::error!("MissingEeprom");
                        }
                        Err(NovellaWriteError::Wearout) => {
                            // try next time and slot
                            defmt::error!("Wearout, T_T try next slot later");
                        }
                        Err(NovellaWriteError::Unknown) => {
                            // try next time and slot
                            defmt::error!("Unknown, ?_? try next slot later");
                        }
                    }
                }
            }

            Timer::after(Duration::from_secs(2)).await;
        }
    }
}

#[embassy_executor::task(pool_size = 1)]
pub async fn novella_spawn(instance: &'static Novella) {
    instance.run().await
}
