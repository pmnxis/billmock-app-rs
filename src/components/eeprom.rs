/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use core::cell::RefCell;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;

use _core::borrow::BorrowMut;
use embassy_stm32::crc::{Config as HwCrcConfig, Crc};
use embassy_stm32::i2c::I2c;
use embassy_stm32::peripherals::{self};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Instant};
use serial_arcade_pay::backup_types::*;
use static_assertions::*;
use static_cell::StaticCell;

use crate::types::const_convert::ConstFrom;

#[repr(packed(2))]
#[derive(Clone)]
pub struct FaultLog {
    current_boot_cnt: u32,
    error_code: u16,
}
assert_eq_size!(FaultLog, [u8; 6]);

// Memory Map - Assume 2KB (16KBits) EEPROM.
// +----------- 2Kbyte EEPROM (Lower) -----------+-------------- 2Kbyte EEPROM (Higher) --------------+
// |                                                                                                  |
// |   Section 0 (0x00-0xFF bytes)   p1_card_cnt    Section 4 (0x400-0x47F bytes) hw_boot_cnt         |
// |   +-----------------------------------------+  +-----------------------------------------+       |
// |   | Page 0  | last_time | p1_card_cnt | CRC |  | Page 0  | last_time | hw_boot_cnt | CRC |       |
// |   | Page 1  | last_time | p1_card_cnt | CRC |  | ...     | ...       | ...         | ... |       |
// |   | ...     | ...       | ...         | ... |  | Page 7  | last_time | hw_boot_cnt | CRC |       |
// |   | Page 15 | last_time | p1_card_cnt | CRC |  +-----------------------------------------+       |
// |   +-----------------------------------------+                                                    |
// |                                                                                                  |
// |   Section 1 (0x100-0x1FF bytes) p2_card_cnt    Section 5 (0x47F-0x4FF bytes) terminal_id[0..6]   |
// |   +-----------------------------------------+  +-----------------------------------------------+ |
// |   | Page 0  | last_time | p2_card_cnt | CRC |  | Page 0  | last_time | terminal_id[0..6] | CRC | |
// |   | Page 1  | last_time | p2_card_cnt | CRC |  | ...     | ...       | ...               | ... | |
// |   | ...     | ...       | ...         | ... |  | Page 7  | last_time | terminal_id[0..6] | CRC | |
// |   | Page 15 | last_time | p2_card_cnt | CRC |  +-----------------------------------------------+ |
// |   +-----------------------------------------+                                                    |
// |                                                Section 0..=3 (big sections, 16 page-ish data)    |
// |   Section 2 (0x200-0x2FF bytes) p1_coin_cnt    Section  0 : p1_card_cnt    u32     4 bytes       |
// |   +-----------------------------------------+  Section  1 : p1_card_cnt    u32     4 bytes       |
// |   | Page 0  | last_time | p1_coin_cnt | CRC |  Section  2 : p1_coin_cnt    u32     4 bytes       |
// |   | Page 1  | last_time | p1_coin_cnt | CRC |  Section  3 : p2_coin_cnt    u32     4 bytes       |
// |   | ...     | ...       | ...         | ... |                                                    |
// |   | Page 15 | last_time | p1_coin_cnt | CRC |  Section 4..=11 (small sections, 8 page-ish data)  |
// |   +-----------------------------------------+  Section  4 : hw_boot_cnt    u32     4 bytes       |
// |                                                Section  5 : terminal_id[0..=5]     6 bytes       |
// |   Section 3 (0x300-0x3FF bytes) p2_coin_cnt    Section  6 : terminal_id[6..=9]     4 bytes       |
// |   +-----------------------------------------+  Section  7 : terminal_id_ext[0..=2] 3 bytes       |
// |   | Page 0  | last_time | p2_coin_cnt | CRC |  Section  8 : card_port1_backup      6 bytes       |
// |   | Page 1  | last_time | p2_coin_cnt | CRC |  Section  9 : card_port2_backup      6 bytes       |
// |   | ...     | ...       | ...         | ... |  Section 10 : card_port3_backup      6 bytes       |
// |   | Page 15 | last_time | p2_coin_cnt | CRC |  Section 11 : card_port4_backup      6 bytes       |
// |   +-----------------------------------------+                                                    |
// +--------------------------------------------------------------------------------------------------+
//
//   Single Page Structure, M24C16's single page size is 16 bytes.
//   Write cycle endurance of each page is 1,200,000 ~ 4,000,0000
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
//   | 0x0 | 0x1 | 0x2 | 0x3 | 0x4 | 0x5 | 0x6 | 0x7 | 0x8 | 0x9 | 0xA | 0xB | 0xC | 0xD | 0xE | 0xF |
//   +-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
//   | last_time : Instant (inner:u64) |   Actual Data (Max 6 byte-size)   |   CRC16   |
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

// Novella part

pub struct MemStruct {
    last_time: u64, // this can be change to u32 version, but need trait degrade
}

/// Tiny control block for manage single section, it include what page is latest and is dirty state
/// +-----+-----+-----+-----+-----+-----+-----+-----+
/// |  b7 |  b6 |  b5 |  b4 |  b3 |  b2 |  b1 |  b0 |
/// +-----+-----+-----+-----+-----+-----+-----+-----+
/// |dirty|  robin: number of what page is latest   |
/// +-----+-----+-----+-----+-----+-----+-----+-----+
pub struct NovellaSectionControlBlock {
    inner: u8,
}

pub type RawNvRobin = u8;
pub type EepromAddress = u8;
type Checksum = u16;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum NvMemSectionKind {
    P1CardCnt,      // 1*16, u32
    P2CardCnt,      // 1*16, u32
    P1CoinCnt,      // 1*16, u32
    P2CoinCnt,      // 1*16, u32
    FaultLog,       // 1*16, Not determined 6 bytes
    HwBootCount,    // 1*08, u32
    TerminalId,     // 2*08, 13 bytes
    CardPortBackup, // 3*08, 32 bytes (4+4)*4
}

impl From<u8> for NvMemSectionKind {
    // instead of FromPrimitive for reduce dependancy
    fn from(value: u8) -> Self {
        unsafe {
            // Future rust will support `core::mem::variant_count::<Self>() as u8`
            assert!(value >= SECTION_NUM as u8);
            core::mem::transmute(value)
        }
    }
}

pub struct NovellaSelector<T> {
    section: NvMemSectionKind,
    marker: PhantomData<T>, // 0-byte guarantees
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

pub enum NvMemSectionReturn {
    P1CardCnt(u32),                    // 1*16, u32+
    P2CardCnt(u32),                    // 1*16, u32+ 1
    P1CoinCnt(u32),                    // 1*16, u32+ 1
    P2CoinCnt(u32),                    // 1*16, u32 + 1
    FaultLog(FaultLog),                // 1*16, Not determined 6 bytes + 1
    HwBootCount(u32),                  // 1*08, u32 + 1
    TerminalId(RawTerminalId),         // 2*08, 13 bytes + 1
    CardPortBackup(RawCardPortBackup), // 3*08, 32 bytes (4+4)*4 + 1
}

impl NovellaSectionControlBlock {
    pub const fn new() -> Self {
        Self { inner: 0x00 }
    }

    pub fn set_dirty(&mut self) {
        self.inner |= 1 << 7;
    }

    pub fn is_dirty(&self) -> bool {
        (self.inner & (1 << 7)) != 0
    }

    pub fn get_robin(&self) -> RawNvRobin {
        self.inner & !(1 << 7)
    }

    pub fn test_and_robin(&mut self) {
        if !self.is_dirty() {
        } else {
            self.inner += 1;
            self.set_dirty()
        }
    }

    fn get_rom_address(&self, section_info: &NvSectionInfo) -> EepromAddress {
        section_info.sect_start_page + (self.get_robin() & (section_info.slot_num - 1))
    }
}

// this should be store as const type
#[repr(C)]
struct NvSectionInfo {
    sect_start_page: u8, // sometimes it's u16, real_offset / 128 (r>>7)
    slot_num: u8,
    slot_size: u8,
    real_data_size: u8,
}

#[rustfmt::skip]
const SECTION_TABLE: [NvSectionInfo; 8] = [
    NvSectionInfo{sect_start_page :  0, slot_num : 16, slot_size : 1, real_data_size :  4 },
    NvSectionInfo{sect_start_page : 16, slot_num : 16, slot_size : 1, real_data_size :  4 },
    NvSectionInfo{sect_start_page : 32, slot_num : 16, slot_size : 1, real_data_size :  4 },
    NvSectionInfo{sect_start_page : 48, slot_num : 16, slot_size : 1, real_data_size :  4 },
    NvSectionInfo{sect_start_page : 64, slot_num :  8, slot_size : 1, real_data_size :  6 },
    NvSectionInfo{sect_start_page : 72, slot_num :  8, slot_size : 1, real_data_size :  4 },
    NvSectionInfo{sect_start_page : 80, slot_num :  8, slot_size : 2, real_data_size : 13 },
    NvSectionInfo{sect_start_page : 96, slot_num :  8, slot_size : 3, real_data_size : 32 },
];

const PAGE_SIZE: usize = 16;
const SECTION_NUM: usize = 8;
const ROM_R_ADDRESS: u8 = 0b1010_0001;
const ROM_W_ADDRESS: u8 = 0b1010_0000;
const ROM_ADDRESS_FIELD_SIZE: usize = core::mem::size_of::<u8>();
const WAIT_DURATION_PER_PAGE: Duration = Duration::from_millis(5); // heuristic value
const CHECKSUM_SIZE: usize = core::mem::size_of::<Checksum>();
const TIMESTAMP_SIZE: usize = core::mem::size_of::<Instant>();

// static EEPROM_WAIT_DEADLINE: Instant =
//     unsafe { MaybeUninit::uninit().assume_init() };
static EEPROM_WAIT_DEADLINE: u64 = 0;

fn novella_i2c_polling_check_timeout() -> Result<(), embassy_stm32::i2c::Error> {
    if Instant::now() < Instant::from_ticks(EEPROM_WAIT_DEADLINE) {
        Ok(())
    } else {
        Err(embassy_stm32::i2c::Error::Timeout)
    }
}

fn novella_i2c_polling_set_timeout(duration: Duration) {
    // EEPROM_WAIT_DEADLINE = Instant::now().as_ticks() + duration.as_ticks();
}

fn novella_i2c_polling_set_default_timeout() {
    novella_i2c_polling_set_timeout(WAIT_DURATION_PER_PAGE);
}

pub struct NovellaModuleControlBlock {
    data: MemStorage,
    controls: [NovellaSectionControlBlock; SECTION_NUM],
}

impl NovellaModuleControlBlock {
    const fn const_default() -> Self {
        // promise default is zero-filled and call `Self::default` later.
        unsafe { MaybeUninit::uninit().assume_init() }
    }

    fn default() -> Self {
        // promise default is zero-filled.
        unsafe { MaybeUninit::zeroed().assume_init() }
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

pub enum NovellaInitError {
    FirstBoot,
    MissingEeprom,
    FaultChecksum,
}

pub enum NovellaReadError {
    FaultChecksum,
    MissingEeprom,
    Unknown,
}

pub enum NovellaWriteError {
    Wearout,
    MissingEeprom,
    Unknown,
}

pub struct Novella {
    bus: UnsafeCell<I2c<'static, peripherals::I2C1, peripherals::DMA1_CH4, peripherals::DMA1_CH3>>,
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
    ) -> Self {
        Self {
            bus: UnsafeCell::new(i2c),
            crc: UnsafeCell::new(crc),
            buffer: UnsafeCell::new(unsafe { MaybeUninit::uninit().assume_init() }),
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
    fn consider_initial_timestamp(page_idx: u8) -> bool {
        page_idx == 0
    }

    #[inline]
    fn consider_tailing_checksum(slot_size: u8, page_idx: u8) -> bool {
        slot_size == page_idx - 1
    }

    /// data is stored in MemStorage's correspond member variable.
    async fn raw_slot_read(
        &self,
        kind: NvMemSectionKind,
        slot_idx: u8,
    ) -> Result<Instant, NovellaReadError> {
        let bus = unsafe { &mut *self.bus.get() };
        let crc = unsafe { &mut *self.crc.get() };
        let rx_buffer: &mut [u8] = unsafe {
            let buffer = &mut *self.buffer.get();
            &mut buffer[core::mem::size_of::<EepromAddress>()..]
        };

        let sect_idx: usize = kind as u8 as usize;
        let slot_size = SECTION_TABLE[sect_idx].slot_size;

        assert!(slot_size <= slot_idx); // should not happens

        let data_address: EepromAddress =
            SECTION_TABLE[sect_idx].sect_start_page + SECTION_TABLE[sect_idx].slot_size * slot_idx;
        let data_address_slice = data_address.to_be_bytes();

        let mut cb = self.mem_storage.lock().await;
        // <- MUTEX SECTION START FROM HERE ->

        let slot_mem = unsafe { cb.get_data_raw_slice(kind) };

        let mut real_data_left = SECTION_TABLE[sect_idx].real_data_size as usize;
        let mut slot_timestamp = Instant::from_ticks(0);
        let mut checksum_expected: u16 = 0;

        for page_idx in 0..slot_size {
            novella_i2c_polling_set_default_timeout(); // preinit for blocking_write_read_timeout
            bus.write_read_timeout(
                ROM_R_ADDRESS,
                &data_address_slice,
                rx_buffer,
                novella_i2c_polling_check_timeout,
            )
            .await
            .map_err(|e| match e {
                embassy_stm32::i2c::Error::Timeout => NovellaReadError::MissingEeprom,
                _ => NovellaReadError::Unknown,
            })?;

            if Self::consider_initial_timestamp(page_idx) {
                // Grab time::Instant of slot
                unsafe {
                    slot_timestamp =
                        (rx_buffer[0..TIMESTAMP_SIZE].as_ptr() as *const Instant).read();
                }
            }

            // copy [slot_real_data_reads..MAX(..)]
            let start_read = Self::consider_initial_timestamp(page_idx) as usize * TIMESTAMP_SIZE;
            let max_real_data_in_page: usize = PAGE_SIZE
                - start_read
                - (Self::consider_tailing_checksum(slot_size, page_idx) as usize * CHECKSUM_SIZE);
            assert!((max_real_data_in_page > PAGE_SIZE)); // need compile time assertion

            let size_can_read = max_real_data_in_page.min(real_data_left as usize);

            let slot_mem_start = SECTION_TABLE[sect_idx].real_data_size as usize - real_data_left;
            let src = &rx_buffer[start_read..start_read + size_can_read];

            for i in 0..size_can_read {
                // instead of `*(dst++) = *(src++)`
                slot_mem[slot_mem_start + i] = src[i];
                // Crc Trait return u32 only, thus type casting to Checksum
                checksum_expected = crc.feed_byte(src[i]) as Checksum;
            }

            real_data_left -= size_can_read;
        }

        // at the end, last buffer filled should contain Checksum.
        let checksum_given = unsafe {
            (rx_buffer[PAGE_SIZE - CHECKSUM_SIZE..PAGE_SIZE].as_ptr() as *const Checksum).read()
        };

        if checksum_given == checksum_expected {
            Ok(slot_timestamp)
        } else {
            Err(NovellaReadError::FaultChecksum)
        }
        // <- MUTEX SECTION END HERE ->
    }

    async fn raw_slot_write(
        &self,
        kind: NvMemSectionKind,
        slot_idx: u8,
        timestamp: Instant,
    ) -> Result<(), NovellaWriteError> {
        let bus = unsafe { &mut *self.bus.get() };
        let crc = unsafe { &mut *self.crc.get() };
        let mut addr_buffer =
            unsafe { &mut (&mut *self.buffer.get())[..core::mem::size_of::<EepromAddress>()] };
        let mut data_buffer =
            unsafe { &mut (&mut *self.buffer.get())[core::mem::size_of::<EepromAddress>()..] };

        let sect_idx: usize = kind as u8 as usize;
        let slot_size = SECTION_TABLE[sect_idx].slot_size;

        assert!(slot_size <= slot_idx); // should not happens

        let mut cb = self.mem_storage.lock().await;
        // <- MUTEX SECTION START FROM HERE ->
        // JUST COPIED
        let mut real_data_left = SECTION_TABLE[sect_idx].real_data_size as usize;
        let slot_mem = unsafe { cb.get_data_raw_slice(kind) };

        let mut checksum_expected: Checksum = 0;

        // Write Oeration
        for page_idx in 0..slot_size {
            // Set eeprom data address
            addr_buffer.copy_from_slice(
                &(SECTION_TABLE[sect_idx].sect_start_page
                    + SECTION_TABLE[sect_idx].slot_size * slot_idx
                    + page_idx * PAGE_SIZE as EepromAddress)
                    .to_be_bytes(),
            );

            if Self::consider_initial_timestamp(page_idx) {
                // Grab time::Instant of slot
                unsafe {
                    // copy for first slice
                    *(data_buffer[0..TIMESTAMP_SIZE].as_mut_ptr() as *mut Instant) = timestamp;

                    let words: &[u32] = core::slice::from_raw_parts(
                        (&timestamp as *const _) as *const u32,
                        core::mem::size_of::<Instant>() / core::mem::size_of::<u32>(),
                    );

                    checksum_expected = crc.feed_words(&words) as Checksum;
                }
            }

            let start_write = Self::consider_initial_timestamp(page_idx) as usize * TIMESTAMP_SIZE;
            let max_real_data_in_page: usize = PAGE_SIZE
                - start_write
                - (Self::consider_tailing_checksum(slot_size, page_idx) as usize * CHECKSUM_SIZE);

            assert!((max_real_data_in_page > PAGE_SIZE)); // need compile time assertion

            let size_can_write = max_real_data_in_page.min(real_data_left as usize);

            let slot_mem_start = SECTION_TABLE[sect_idx].real_data_size as usize - real_data_left;
            let dst = &mut data_buffer[start_write..start_write + size_can_write];

            for i in 0..size_can_write {
                // instead of `*(dst++) = *(src++)`
                dst[i] = slot_mem[slot_mem_start + i];
                // Crc Trait return u32 only, thus type casting to Checksum
                checksum_expected = crc.feed_byte(dst[i]) as Checksum;
            }

            if Self::consider_tailing_checksum(slot_size, page_idx) {
                unsafe {
                    *(data_buffer[PAGE_SIZE - CHECKSUM_SIZE..PAGE_SIZE].as_mut_ptr()
                        as *mut Checksum) = checksum_expected;
                }
            }

            novella_i2c_polling_set_default_timeout();

            bus.write_timeout(
                ROM_W_ADDRESS,
                unsafe { &*self.buffer.get() }, // when page is 16, include 1+16 byte will be tx.
                novella_i2c_polling_check_timeout,
            )
            .await;
        }

        // Read for check, do not copy value to MemStorage
        let mut checksum_double_expected: Checksum = 0;

        for page_idx in 0..slot_size {
            // Set eeprom data address
            addr_buffer.copy_from_slice(
                &(SECTION_TABLE[sect_idx].sect_start_page
                    + SECTION_TABLE[sect_idx].slot_size * slot_idx
                    + page_idx * PAGE_SIZE as EepromAddress)
                    .to_be_bytes(),
            );

            novella_i2c_polling_set_default_timeout(); // preinit for blocking_write_read_timeout
            bus.write_read_timeout(
                ROM_R_ADDRESS,
                &addr_buffer,
                data_buffer,
                novella_i2c_polling_check_timeout,
            )
            .await
            .map_err(|e| match e {
                embassy_stm32::i2c::Error::Timeout => NovellaWriteError::MissingEeprom,
                _ => NovellaWriteError::Unknown,
            })?;

            if Self::consider_initial_timestamp(page_idx) {
                // Grab time::Instant of slot
                let timestamp_words: &[u32] = unsafe {
                    core::slice::from_raw_parts(
                        (&data_buffer[0..TIMESTAMP_SIZE] as *const _) as *const u32,
                        core::mem::size_of::<Instant>() / core::mem::size_of::<u32>(),
                    )
                };
                let timestamp_given: &[u32] = unsafe {
                    core::slice::from_raw_parts(
                        (&timestamp as *const _) as *const u32,
                        core::mem::size_of::<Instant>() / core::mem::size_of::<u32>(),
                    )
                };

                if timestamp_given == timestamp_words {
                    return Err(NovellaWriteError::Wearout);
                } else {
                    checksum_double_expected = crc.feed_words(timestamp_words) as Checksum;
                }
            }

            // copy [slot_real_data_reads..MAX(..)]
            let start_read = Self::consider_initial_timestamp(page_idx) as usize * TIMESTAMP_SIZE;
            let max_real_data_in_page: usize = PAGE_SIZE
                - start_read
                - (Self::consider_tailing_checksum(slot_size, page_idx) as usize * CHECKSUM_SIZE);
            assert!((max_real_data_in_page > PAGE_SIZE)); // need compile time assertion

            let size_can_read = max_real_data_in_page.min(real_data_left as usize);

            let slot_mem_start = SECTION_TABLE[sect_idx].real_data_size as usize - real_data_left;
            let src = &data_buffer[start_read..start_read + size_can_read];

            for i in 0..size_can_read {
                // Crc Trait return u32 only, thus type casting to Checksum
                checksum_double_expected = crc.feed_byte(src[i]) as Checksum;
            }

            real_data_left -= size_can_read;
        }

        // at the end, last buffer filled should contain Checksum.
        let checksum_given = unsafe {
            (data_buffer[PAGE_SIZE - CHECKSUM_SIZE..PAGE_SIZE].as_ptr() as *const Checksum).read()
        };

        if (checksum_expected == checksum_given) && (checksum_expected == checksum_double_expected)
        {
            Err(NovellaWriteError::Wearout)
        } else {
            Ok(())
        }
    }

    // // at the end, last buffer filled should contain Checksum.
    // let checksum_given = unsafe {
    //     (buffer[PAGE_SIZE - CHECKSUM_SIZE..PAGE_SIZE].as_ptr() as *const Checksum).read()
    // };

    // if checksum_given == checksum_expected {
    //     Ok(slot_timestamp)
    // } else {
    //     Err(NovellaReadError::FaultChecksum)
    // }
    // <- MUTEX SECTION END HERE ->

    /// when success return marked last time
    /// Success to detect eeprom but it's filled in 0xFF or 0xFF are initial factory value, return NovellaInitError::FirstBoot
    /// initialization is not using async/await for safety
    pub async fn init(&self) -> Result<Instant, NovellaInitError> {
        #[inline]
        fn consider_initial_timestamp(page_idx: u8) -> bool {
            page_idx == 0
        }

        #[inline]
        fn consider_tailing_checksum(slot_size: u8, page_idx: u8) -> bool {
            slot_size == page_idx - 1
        }

        let bus = unsafe { &mut *self.bus.get() };
        let crc = unsafe { &mut *self.crc.get() };
        let buffer: &mut [u8] = unsafe { &mut *self.buffer.get() };

        // self.mem_storage.lock(|cb| {
        //     let cb = &mut *cb.borrow_mut();

        //     // lazy zero wipe init (is this really need?)
        //     *cb = NovellaModuleControlBlock::default();

        //     let mut latest = Instant::from_ticks(0); // guarantees smallest

        //     // crc.reconfigure() // not exposed as public for now
        //     let mut fail_count = 0;

        //     for sect_idx in 0..SECTION_NUM {
        //         let (mut latest_per_sector, mut latest_slot) = (Instant::from_ticks(0), None);
        //         let mut sect_real_data_reads = 0;
        //         let kind = NvMemSectionKind::from(sect_idx as u8);
        //         let slot_mem = unsafe { cb.get_data_raw_slice(kind) };

        //         for slot_idx in 0..SECTION_TABLE[sect_idx].slot_num {
        //             let data_address: u8 = SECTION_TABLE[sect_idx].sect_start_page
        //                 + SECTION_TABLE[sect_idx].slot_size * slot_idx;
        //             let data_address_slice = data_address.to_be_bytes();

        //             let slot_size = SECTION_TABLE[sect_idx].slot_size;
        //             let mut real_data_left = SECTION_TABLE[sect_idx].real_data_size as usize;
        //             let mut slot_timestamp = Instant::from_ticks(0);
        //             let mut checksum_expected: u16 = 0;

        //             for page_idx in 0..slot_size {
        //                 novella_i2c_polling_set_default_timeout(); // preinit for blocking_write_read_timeout
        //                 let a = bus.blocking_write_read_timeout(
        //                     ROM_R_ADDRESS,
        //                     &data_address_slice,
        //                     buffer,
        //                     novella_i2c_polling_check_timeout,
        //                 );

        //                 if consider_initial_timestamp(page_idx) {
        //                     // Grab time::Instant of slot
        //                     unsafe {
        //                         slot_timestamp =
        //                             (buffer[0..TIMESTAMP_SIZE].as_ptr() as *const Instant).read();
        //                     }
        //                 }

        //                 // copy [slot_real_data_reads..MAX(..)]
        //                 let start_read =
        //                     consider_initial_timestamp(page_idx) as usize * TIMESTAMP_SIZE;
        //                 let max_real_data_in_page: usize = PAGE_SIZE
        //                     - start_read
        //                     - (consider_tailing_checksum(slot_size, page_idx) as usize
        //                         * CHECKSUM_SIZE);
        //                 assert!((max_real_data_in_page > PAGE_SIZE)); // need compile time assertion

        //                 let size_can_read = max_real_data_in_page.min(real_data_left as usize);

        //                 let slot_mem_start =
        //                     SECTION_TABLE[sect_idx].real_data_size as usize - real_data_left;
        //                 let src = &buffer[start_read..start_read + size_can_read];

        //                 for i in 0..size_can_read {
        //                     // instead of `*(dst++) = *(src++)`
        //                     slot_mem[slot_mem_start + i] = src[i];
        //                     // Crc Trait return u32 only, thus type casting to Checksum
        //                     checksum_expected = crc.feed_byte(src[i]) as Checksum;
        //                 }

        //                 real_data_left -= size_can_read;

        //                 // double condition for safety
        //                 if (slot_size <= 1) || (slot_size == (page_idx - 1)) {
        //                     let checksum_given = unsafe {
        //                         (buffer[PAGE_SIZE - CHECKSUM_SIZE..PAGE_SIZE].as_ptr()
        //                             as *const Checksum)
        //                             .read()
        //                     };

        //                     if checksum_given == checksum_expected {
        //                         if latest_per_sector < slot_timestamp {
        //                             latest_per_sector = slot_timestamp;
        //                             latest_slot = Some(slot_idx);
        //                         }

        //                         if latest < slot_timestamp {
        //                             latest = slot_timestamp;
        //                         }
        //                     } else {
        //                         fail_count += 1;

        //                         // reset
        //                         slot_mem.fill(0);
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // });
        Err(NovellaInitError::FirstBoot)
    }
}
