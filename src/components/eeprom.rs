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
use embassy_sync::blocking_mutex::Mutex;
use serial_arcade_pay::backup_types::*;
use static_assertions::*;

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
//   | last_time : embassy_time::Instant (inner:u64) |   Actual Data (Max 6 byte-size)   |   CRC16   |
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

pub struct NovellaSelector<T> {
    section: NvMemSectionKind,
    marker: PhantomData<T>, // 0-byte guarantees
}

pub trait NovellaRw {
    type InnerType: Sized;
    fn lock_read(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, RefCell<NovellaModuleControlBlock>>,
    ) -> Self::InnerType;

    fn lock_write(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, RefCell<NovellaModuleControlBlock>>,
        src: Self::InnerType,
    );
}

fn should_not_happen() -> ! {
    panic!("should not be happens")
}

// this should be generated by macro
impl NovellaRw for NovellaSelector<u32> {
    type InnerType = u32;
    fn lock_read(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, RefCell<NovellaModuleControlBlock>>,
    ) -> Self::InnerType {
        mutex.lock(|cb| {
            let cb = &mut *cb.borrow_mut();
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
        })
    }

    fn lock_write(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, RefCell<NovellaModuleControlBlock>>,
        src: Self::InnerType,
    ) {
        mutex.lock(|cb| {
            let cb = &mut *cb.borrow_mut();
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
        })
    }
}

impl NovellaRw for NovellaSelector<FaultLog> {
    type InnerType = FaultLog;
    fn lock_read(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, RefCell<NovellaModuleControlBlock>>,
    ) -> Self::InnerType {
        mutex.lock(|cb| {
            let cb = &mut *cb.borrow_mut();
            match self.section {
                NvMemSectionKind::FaultLog => cb.data.fault_log.clone(),
                _ => {
                    should_not_happen();
                }
            }
        })
    }

    fn lock_write(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, RefCell<NovellaModuleControlBlock>>,
        src: Self::InnerType,
    ) {
        mutex.lock(|cb| {
            let cb = &mut *cb.borrow_mut();
            match self.section {
                NvMemSectionKind::FaultLog => {
                    cb.data.fault_log = src;
                }
                _ => {
                    should_not_happen();
                }
            }

            cb.control_mut(self.section).set_dirty();
        })
    }
}

impl NovellaRw for NovellaSelector<RawTerminalId> {
    type InnerType = RawTerminalId;
    fn lock_read(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, RefCell<NovellaModuleControlBlock>>,
    ) -> Self::InnerType {
        mutex.lock(|cb| {
            let cb = &mut *cb.borrow_mut();
            match self.section {
                NvMemSectionKind::TerminalId => cb.data.raw_terminal.clone(),
                _ => {
                    should_not_happen();
                }
            }
        })
    }

    fn lock_write(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, RefCell<NovellaModuleControlBlock>>,
        src: Self::InnerType,
    ) {
        mutex.lock(|cb| {
            let cb = &mut *cb.borrow_mut();

            match self.section {
                NvMemSectionKind::FaultLog => {
                    cb.data.raw_terminal = src;
                }
                _ => {
                    should_not_happen();
                }
            };

            cb.control_mut(self.section).set_dirty();
        })
    }
}

impl NovellaRw for NovellaSelector<CardReaderPortBackup> {
    type InnerType = CardReaderPortBackup;
    fn lock_read(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, RefCell<NovellaModuleControlBlock>>,
    ) -> Self::InnerType {
        mutex.lock(|cb| {
            let cb = &mut *cb.borrow_mut();
            match self.section {
                NvMemSectionKind::CardPortBackup => cb.data.card_reader_port_backup.clone(),
                _ => {
                    should_not_happen();
                }
            }
        })
    }

    fn lock_write(
        &self,
        mutex: &Mutex<ThreadModeRawMutex, RefCell<NovellaModuleControlBlock>>,
        src: Self::InnerType,
    ) {
        mutex.lock(|cb| {
            let cb = &mut *cb.borrow_mut();

            let dst = match self.section {
                NvMemSectionKind::CardPortBackup => {
                    cb.data.card_reader_port_backup = src;
                }
                _ => {
                    should_not_happen();
                }
            };

            cb.control_mut(self.section).set_dirty();
        })
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
}

#[rustfmt::skip]
const SECTION_TABLE: [NvSectionInfo; 8] = [
    NvSectionInfo{sect_start_page :  0, slot_num : 16, slot_size : 1 },
    NvSectionInfo{sect_start_page : 16, slot_num : 16, slot_size : 1 },
    NvSectionInfo{sect_start_page : 32, slot_num : 16, slot_size : 1 },
    NvSectionInfo{sect_start_page : 48, slot_num : 16, slot_size : 1 },
    NvSectionInfo{sect_start_page : 64, slot_num :  8, slot_size : 1 },
    NvSectionInfo{sect_start_page : 72, slot_num :  8, slot_size : 1 },
    NvSectionInfo{sect_start_page : 80, slot_num :  8, slot_size : 2 },
    NvSectionInfo{sect_start_page : 96, slot_num :  8, slot_size : 3 },
];

const PAGE_SIZE: usize = 16;
const SECTION_NUM: usize = 8;

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

    fn control_mut(&mut self, kind: NvMemSectionKind) -> &mut NovellaSectionControlBlock {
        // It's fine with get_unchecked_mut, instead of `get_mut(...) -> Option<I>`.
        // KindT enum is directly limited on element number of internnaly array.
        unsafe { self.controls.get_unchecked_mut(kind as usize) }
    }
}

pub enum NovellaInitError {
    FirstBoot,
    MissingEeprom,
}

pub struct Novella {
    bus: UnsafeCell<I2c<'static, peripherals::I2C1, peripherals::DMA1_CH4, peripherals::DMA1_CH3>>,
    buffer: UnsafeCell<[u8; PAGE_SIZE]>,
    mem_storage: Mutex<ThreadModeRawMutex, RefCell<NovellaModuleControlBlock>>,
}

impl Novella {
    /// const_new for hardware initialization
    pub const fn const_new(
        i2c: I2c<'static, peripherals::I2C1, peripherals::DMA1_CH4, peripherals::DMA1_CH3>,
    ) -> Self {
        Self {
            bus: UnsafeCell::new(i2c),
            buffer: UnsafeCell::new(unsafe { MaybeUninit::uninit().assume_init() }),
            mem_storage: Mutex::new(RefCell::new(NovellaModuleControlBlock::const_default())),
        }
    }

    pub fn lock_read<R>(&self, slot: R) -> R::InnerType
    where
        R: NovellaRw,
    {
        slot.lock_read(&self.mem_storage)
    }

    pub fn lock_write<R>(&self, slot: R, src: R::InnerType)
    where
        R: NovellaRw,
    {
        slot.lock_write(&self.mem_storage, src)
    }

    /// when success return marked last time
    /// Success to detect eeprom but it's filled in 0xFF or 0xFF are initial factory value, return NovellaInitError::FirstBoot
    pub fn init(&self) -> Result<embassy_time::Instant, NovellaInitError> {
        // implement me

        // lazy zero wipe init (is this really need?)
        self.mem_storage
            .lock(|cb| cb.replace(NovellaModuleControlBlock::default()));

        Err(NovellaInitError::FirstBoot)
    }
}
