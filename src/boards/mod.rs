/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config as Stm32Config;
use static_cell::make_static;

#[cfg(feature = "hw_0v2")]
use self::billmock_0v2::hardware_init_0v2;
#[cfg(feature = "hw_0v3")]
use self::billmock_0v3::hardware_init_0v3;
#[cfg(feature = "hw_0v4")]
use self::billmock_0v4::hardware_init_0v4;
#[cfg(feature = "hw_mini_0v4")]
use self::billmock_mini_0v4::hardware_init_mini_0v4;
use crate::components::dip_switch::DipSwitch;
use crate::components::eeprom::novella_spawn;
use crate::components::eeprom::Novella;
use crate::components::host_side_bill::HostSideBill;
use crate::components::serial_device::{self, card_reader_device_spawn, CardReaderDevice};
use crate::components::vend_side_bill::VendSideBill;
use crate::semi_layer::buffered_opendrain::{buffered_opendrain_spawn, BufferedOpenDrain};
use crate::semi_layer::buffered_wait_receiver::BufferedWaitReceiver;
use crate::semi_layer::timing::{SharedToggleTiming, ToggleTiming};
use crate::types::input_port::InputPortKind;

pub const PLAYER_INDEX_MAX: usize = 2;
pub const PLAYER_1_INDEX: usize = 0;
pub const PLAYER_2_INDEX: usize = 1;

pub const LED_INDEX_MAX: usize = 2;
pub const LED_1_INDEX: usize = 0;
pub const LED_2_INDEX: usize = 1;

const PROJECT_NAME: &str = env!("PROJECT_NAME");
const VERSION_STR: &str = env!("PROJECT_VERSION");
const COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");
const COMMIT_SHORT: &str = env!("GIT_COMMIT_SHORT_HASH");
const GIT_COMMIT_DATETIME: &str = env!("GIT_COMMIT_DATETIME");
const PRINT_BAR: &str = "+-----------------------------------------------------------+";

#[cfg(feature = "hw_0v2")]
mod billmock_0v2;
#[cfg(feature = "hw_0v3")]
mod billmock_0v3;
#[cfg(feature = "hw_0v4")]
mod billmock_0v4;
#[cfg(feature = "hw_mini_0v4")]
mod billmock_mini_0v4;

pub struct Hardware {
    /// Bill paper and coin acceptor input device for 1 and 2 player sides
    pub vend_sides: [VendSideBill; PLAYER_INDEX_MAX],

    /// GAME I/O PCB for 1 and 2 player sides
    pub host_sides: [HostSideBill; PLAYER_INDEX_MAX],

    /// Two indicators with green light
    pub indicators: [BufferedOpenDrain; LED_INDEX_MAX],

    /// Hexa dip switch
    pub dipsw: DipSwitch,

    /// Card reader for serial arcade payement
    pub card_reader: CardReaderDevice,

    /// Eeprom manager, powered by Novella
    pub eeprom: Novella,
}

impl Hardware {
    /// STM32G030 64Mhz maximum CPU configuation
    #[allow(dead_code)]
    fn mcu_config_ppl_max_speed() -> Stm32Config {
        // STM32G030 maximum CPU clock is 64Mhz.
        let mut ret = Stm32Config::default();
        ret.rcc.mux = embassy_stm32::rcc::ClockSrc::PLL(embassy_stm32::rcc::PllConfig {
            source: embassy_stm32::rcc::PllSrc::HSI16,
            m: embassy_stm32::rcc::Pllm::DIV1,
            n: embassy_stm32::rcc::Plln::MUL8,
            r: embassy_stm32::rcc::Pllr::DIV2,
            q: None,
            p: None,
        });

        ret
    }

    /// STM32G030 16Mhz basic CPU configuration
    #[allow(dead_code)]
    fn mcu_default_config() -> Stm32Config {
        Stm32Config::default()
    }

    /// Initialize MCU PLL and CPU on init hardware
    /// > `Hardware::mcu_pre_init()`
    /// 2 `make_static!(SharedResource::init())`
    /// 3 `Hardware::hardware_init(..)`
    /// 4 `hardware.start_tasks(..)`
    pub fn mcu_pre_init() -> embassy_stm32::Peripherals {
        embassy_stm32::init(Self::mcu_default_config())
    }

    /// Initialize MCU peripherals and nearby components
    /// 1 `Hardware::mcu_pre_init()`
    /// 2 `make_static!(SharedResource::init())`
    /// > `Hardware::hardware_init(..)`
    /// 4 `hardware.start_tasks(..)`
    fn hardware_init(
        peripherals: embassy_stm32::Peripherals,
        shared_resource: &'static SharedResource,
    ) -> Hardware {
        #[cfg(all(
            feature = "hw_0v2",
            not(feature = "hw_0v3"),
            not(feature = "hw_0v4"),
            not(feature = "hw_mini_0v4")
        ))]
        let ret = hardware_init_0v2(peripherals, shared_resource);

        #[cfg(all(
            all(
                feature = "hw_0v3",
                not(feature = "hw_0v2"),
                not(feature = "hw_0v4"),
                not(feature = "hw_mini_0v4")
            ),
            feature = "billmock_default"
        ))]
        let ret = hardware_init_0v3(peripherals, shared_resource);

        #[cfg(all(
            feature = "hw_0v4",
            not(feature = "hw_0v2"),
            not(feature = "hw_0v3"),
            not(feature = "hw_mini_0v4")
        ))]
        let ret = hardware_init_0v4(peripherals, shared_resource);

        #[cfg(all(
            feature = "hw_mini_0v4",
            not(feature = "hw_0v2"),
            not(feature = "hw_0v3"),
            not(feature = "hw_0v4")
        ))]
        let ret = hardware_init_mini_0v4(peripherals, shared_resource);

        ret
    }

    /// Initialize MCU peripherals and nearby components
    /// 1 `Hardware::mcu_pre_init()`
    /// 2 `make_static!(SharedResource::init())`
    /// 3 `Hardware::hardware_init(..)`
    /// > `hardware.start_tasks(..)`
    fn start_tasks(&'static self, spawner: &Spawner) {
        // Vend legacy device initialization
        self.vend_sides[PLAYER_1_INDEX].start_tasks(spawner);
        self.vend_sides[PLAYER_2_INDEX].start_tasks(spawner);

        // Game IO PCB side player 1 and 2 mocked module initialization
        self.host_sides[PLAYER_1_INDEX].start_tasks(spawner);
        self.host_sides[PLAYER_2_INDEX].start_tasks(spawner);

        // LED indicators inside of PCB initialization. for debug / indication.
        unwrap!(spawner.spawn(buffered_opendrain_spawn(&self.indicators[LED_1_INDEX])));
        unwrap!(spawner.spawn(buffered_opendrain_spawn(&self.indicators[LED_2_INDEX])));

        // nothing to do for dipsw for now
        // DIP switch module initialization

        // USART CardReaderDevice module initialization
        unwrap!(spawner.spawn(card_reader_device_spawn(&self.card_reader)));
        serial_device::alert_module_status();

        unwrap!(spawner.spawn(novella_spawn(&self.eeprom)));
    }
}

pub struct SharedResource {
    /// Common Input event channel
    pub async_input_event_ch: BufferedWaitReceiver,

    /// Open-drain signal timing that shared or const-ish
    pub arcade_players_timing: [SharedToggleTiming; PLAYER_INDEX_MAX],

    /// LED and start button LED related timing that shared or const-ish.
    pub indicator_timing: SharedToggleTiming,
}

impl SharedResource {
    /// Initialize necessary shared resource
    /// 1 `Hardware::mcu_pre_init()`
    /// > `make_static!(SharedResource::init())`
    /// 3 `Hardware::hardware_init(..)`
    /// 4 `hardware.start_tasks(..)`
    fn init() -> Self {
        Self {
            async_input_event_ch: BufferedWaitReceiver::new(),
            arcade_players_timing: [SharedToggleTiming::default(), SharedToggleTiming::default()],
            indicator_timing: SharedToggleTiming::new_custom(ToggleTiming {
                high_ms: 500,
                low_ms: 500,
            }),
        }
    }
}

pub struct BoardCorrespondOutputMatchError<Enum: Sized> {
    pub origin: Enum,
}

pub struct Board {
    pub hardware: Hardware,
    pub shared_resource: &'static SharedResource,
}

impl Board {
    pub fn init() -> Self {
        let p = Hardware::mcu_pre_init();

        // print my info
        defmt::println!("{}", PRINT_BAR);
        defmt::println!("Firmware Ver : {} {}", PROJECT_NAME, VERSION_STR);
        defmt::println!("Git Hash     : {}", COMMIT_HASH);
        defmt::println!("Git Datetime : {} | {}", GIT_COMMIT_DATETIME, COMMIT_SHORT);
        defmt::println!("{}", PRINT_BAR);

        let shared_resource = make_static!(SharedResource::init());
        let hardware: Hardware = Hardware::hardware_init(p, shared_resource);

        Self {
            hardware,
            shared_resource,
        }
    }

    pub fn start_tasks(&'static self, spawner: &Spawner) -> &Self {
        self.hardware.start_tasks(spawner);
        self
    }

    pub fn correspond_output(
        &'static self,
        port: &InputPortKind,
    ) -> Result<&BufferedOpenDrain, BoardCorrespondOutputMatchError<InputPortKind>> {
        match port {
            InputPortKind::Vend1P => Ok(&self.hardware.host_sides[PLAYER_1_INDEX].out_vend),
            InputPortKind::Vend2P => Ok(&self.hardware.host_sides[PLAYER_2_INDEX].out_vend),
            InputPortKind::Jam1P => Ok(&self.hardware.host_sides[PLAYER_1_INDEX].out_jam),
            InputPortKind::Jam2P => Ok(&self.hardware.host_sides[PLAYER_2_INDEX].out_jam),
            InputPortKind::Start1P => Ok(&self.hardware.host_sides[PLAYER_1_INDEX].out_start),
            InputPortKind::Start2P => Ok(&self.hardware.host_sides[PLAYER_2_INDEX].out_start),
            InputPortKind::Inhibit1P => Ok(&self.hardware.vend_sides[PLAYER_1_INDEX].out_inhibit),
            InputPortKind::Inhibit2P => Ok(&self.hardware.vend_sides[PLAYER_2_INDEX].out_inhibit),
            x => Err(BoardCorrespondOutputMatchError { origin: *x }),
        }
    }

    pub fn correspond_indicator(&'static self, port: &InputPortKind) -> Option<&BufferedOpenDrain> {
        match port {
            InputPortKind::Vend1P => Some(&self.hardware.indicators[LED_1_INDEX]),
            InputPortKind::Vend2P => Some(&self.hardware.indicators[LED_2_INDEX]),
            _ => None, // this is optional action, thus return with None , not Err
        }
    }

    pub fn correspond_busy(&'static self, port: &InputPortKind) -> Option<&BufferedOpenDrain> {
        match port {
            InputPortKind::Vend1P => Some(&self.hardware.host_sides[PLAYER_1_INDEX].out_busy),
            InputPortKind::Vend2P => Some(&self.hardware.host_sides[PLAYER_2_INDEX].out_busy),
            _ => None, // this is optional action, thus return with None , not Err
        }
    }
}
