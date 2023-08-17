/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config as Stm32Config;
use embassy_sync::channel::Channel;
use static_cell::make_static;

use self::billmock_0v2::hardware_init_0v2;
use crate::components::dip_switch::DipSwitch;
use crate::components::host_side_bill::HostSideBill;
use crate::components::serial_device::{self, card_reader_device_spawn, CardReaderDevice};
use crate::components::vend_side_bill::VendSideBill;
use crate::semi_layer::buffered_opendrain::{buffered_opendrain_spawn, BufferedOpenDrain};
use crate::semi_layer::buffered_wait::InputEventChannel;
use crate::semi_layer::timing::{DualPoleToggleTiming, SharedToggleTiming, ToggleTiming};

pub const PLAYER_INDEX_MAX: usize = 2;
pub const PLAYER_1_INDEX: usize = 0;
pub const PLAYER_2_INDEX: usize = 1;

pub const LED_INDEX_MAX: usize = 2;
pub const LED_1_INDEX: usize = 0;
pub const LED_2_INDEX: usize = 1;

mod billmock_0v2;

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
}

impl Hardware {
    /// STM32G030 64Mhz maximum CPU configuation
    #[allow(dead_code)]
    fn mcu_config_ppl_max_speed() -> Stm32Config {
        // STM32G030 maximum CPU clock is 64Mhz.
        let mut ret = Stm32Config::default();
        ret.rcc.mux = embassy_stm32::rcc::ClockSrc::PLL(embassy_stm32::rcc::PllConfig {
            source: embassy_stm32::rcc::PllSrc::HSI16,
            m: embassy_stm32::rcc::Pllm::Div1,
            n: 8,
            r: embassy_stm32::rcc::Pllr::Div2,
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
    /// 2 `SharedResource::init()`
    /// 3 `Hardware::hardware_init(..)`
    /// 4 `hardware.start_tasks(..)`
    pub fn mcu_pre_init() -> embassy_stm32::Peripherals {
        embassy_stm32::init(Self::mcu_default_config())
    }

    /// Initialize MCU peripherals and nearby components
    /// 1 `Hardware::mcu_pre_init()`
    /// 2 `SharedResource::init()`
    /// > `Hardware::hardware_init(..)`
    /// 4 `hardware.start_tasks(..)`
    fn hardware_init(
        peripherals: embassy_stm32::Peripherals,
        shared_resource: &'static SharedResource,
    ) -> Hardware {
        hardware_init_0v2(peripherals, shared_resource)
    }

    /// Initialize MCU peripherals and nearby components
    /// 1 `Hardware::mcu_pre_init()`
    /// 2 `SharedResource::init()`
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
    }
}

pub struct SharedResource {
    /// Common Input event channel
    pub async_input_event_ch: InputEventChannel,

    /// Open-drain signal timing that shared or const-ish
    pub arcade_players_timing: [DualPoleToggleTiming; PLAYER_INDEX_MAX],

    /// LED and start button LED related timing that shared or const-ish.
    pub indicator_timing: DualPoleToggleTiming,
}

impl SharedResource {
    /// Initialize necessary shared resource
    /// 1 `Hardware::mcu_pre_init()`
    /// > `SharedResource::init()`
    /// 3 `Hardware::hardware_init(..)`
    /// 4 `hardware.start_tasks(..)`
    fn init() -> Self {
        let player1_timing_shared = make_static!(SharedToggleTiming::default());
        let player1_timing_alt = make_static!(ToggleTiming::default());

        let player2_timing_shared = make_static!(SharedToggleTiming::default());
        let player2_timing_alt = make_static!(ToggleTiming::default());

        let indicator_shared = make_static!(SharedToggleTiming::new_custom(ToggleTiming {
            high_ms: 500,
            low_ms: 500
        }));
        let indicator_alt = make_static!(ToggleTiming {
            high_ms: 1000,
            low_ms: 1000
        });

        Self {
            async_input_event_ch: Channel::new(),
            arcade_players_timing: [
                DualPoleToggleTiming::new(player1_timing_shared, player1_timing_alt),
                DualPoleToggleTiming::new(player2_timing_shared, player2_timing_alt),
            ],
            indicator_timing: DualPoleToggleTiming::new(indicator_shared, indicator_alt),
        }
    }
}

pub struct Board {
    pub hardware: Hardware,
    pub shared_resource: &'static SharedResource,
}

impl Board {
    pub fn init() -> Self {
        let p = Hardware::mcu_pre_init();

        let shared_resource = make_static!(SharedResource::init());
        let hardware: Hardware = Hardware::hardware_init(p, shared_resource);

        Self {
            hardware,
            shared_resource: shared_resource,
        }
    }

    pub fn start_tasks(&'static self, spawner: &Spawner) -> &Self {
        self.hardware.start_tasks(spawner);
        self
    }
}
