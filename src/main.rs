/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#![no_main]
#![cfg_attr(not(test), no_std)]
#![feature(type_alias_impl_trait)]

mod components;
mod semi_layer;
mod types;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::{Channel as HwChannel, ExtiInput};
use embassy_stm32::gpio::{Input, Level, Output, Pin, Pull, Speed};
use embassy_stm32::usart::{Config as UsartConfig, Uart};
use embassy_stm32::Config as Stm32Config;
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

use crate::components::dip_switch::DipSwitch;
use crate::components::host_side_bill::HostSideBill;
use crate::components::serial_device::{card_reader_device_spawn, CardReaderDevice};
use crate::components::start_button::StartButton;
use crate::components::vend_side_bill::VendSideBill;
use crate::semi_layer::buffered_opendrain::{buffered_opendrain_spawn, BufferedOpenDrain};
use crate::semi_layer::buffered_wait::{InputEventChannel, InputEventKind, InputPortKind};
use crate::semi_layer::timing::{DualPoleToggleTiming, SharedToggleTiming, ToggleTiming};

bind_interrupts!(struct Irqs {
    USART2 => embassy_stm32::usart::InterruptHandler<peripherals::USART2>;
});

// Common Input event channel
static ASYNC_INPUT_EVENT_CH: InputEventChannel = Channel::new();

// Open-drain signal timing that shared or const-ish
static COMMON_COIN_SIGNAL_TIMING: SharedToggleTiming = SharedToggleTiming::default();
static COMMON_ALT_SIGNAL_TIMING: ToggleTiming = ToggleTiming::default();
static COMMON_TIMING: DualPoleToggleTiming =
    DualPoleToggleTiming::new(&COMMON_COIN_SIGNAL_TIMING, &COMMON_ALT_SIGNAL_TIMING);

// LED and start button LED related timing that shared or const-ish.
static START_BUTTON_LED_STD_TIMING: SharedToggleTiming =
    SharedToggleTiming::new_custom(ToggleTiming {
        high_ms: 500,
        low_ms: 500,
    });
static COMMON_LED_STD_TIMING: SharedToggleTiming = SharedToggleTiming::new_custom(ToggleTiming {
    high_ms: 500,
    low_ms: 500,
});
static COMMON_LED_ALT_TIMING: ToggleTiming = ToggleTiming {
    high_ms: 1000,
    low_ms: 1000,
};
static START_BUTTON_LED_TIMING: DualPoleToggleTiming =
    DualPoleToggleTiming::new(&START_BUTTON_LED_STD_TIMING, &COMMON_LED_ALT_TIMING);
static COMMON_LED_TIMING: DualPoleToggleTiming =
    DualPoleToggleTiming::new(&COMMON_LED_STD_TIMING, &COMMON_LED_ALT_TIMING);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let stm32_config = {
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
    };
    // stm32_config.rcc. = Some(Hertz(32_000_000));
    let p = embassy_stm32::init(stm32_config);
    info!("billmock-app-rs starting...");

    // Vend legacy device initialization
    let vend_legacy = make_static!(VendSideBill::new(
        Output::new(p.PB0.degrade(), Level::Low, Speed::Low), // REAL_INH
        ExtiInput::new(
            Input::new(p.PB2, Pull::None).degrade(), // REAL_VND
            p.EXTI2.degrade(),                       // EXTI2
        ),
        InputPortKind::Vend,
        ExtiInput::new(
            Input::new(p.PB14, Pull::None).degrade(), // REAL_JAM (moved from PB15 at HW v0.3)
            p.EXTI14.degrade(),                       // EXTI14
        ),
        InputPortKind::Jam,
        &ASYNC_INPUT_EVENT_CH,
        &COMMON_TIMING,
    ));
    vend_legacy.start_tasks(&spawner);
    info!("Legacy vend side bill module loaded");

    // Player 1 button with (LED+switch) initialization
    let start_1p = make_static!(StartButton::new(
        ExtiInput::new(
            Input::new(p.PB11, Pull::None).degrade(), // REAL_STR0
            p.EXTI11.degrade(),                       // EXTI11
        ),
        InputPortKind::Start1P,
        Output::new(p.PA0.degrade(), Level::Low, Speed::Low), // VIRT0_VND
        &ASYNC_INPUT_EVENT_CH,
        &START_BUTTON_LED_TIMING,
    ));
    start_1p.start_tasks(&spawner);
    info!("Real player 1 button module loaded");

    // Player 2 button with (LED+switch) initialization
    let start_2p = make_static!(StartButton::new(
        ExtiInput::new(
            Input::new(p.PD1, Pull::None).degrade(), // REAL_STR1
            p.EXTI1.degrade(),                       // EXTI1
        ),
        InputPortKind::Start2P,
        Output::new(p.PA1.degrade(), Level::Low, Speed::Low), // VIRT1_VND
        &ASYNC_INPUT_EVENT_CH,
        &START_BUTTON_LED_TIMING,
    ));
    start_2p.start_tasks(&spawner);
    info!("Real player 2 button module loaded");

    // Game IO PCB side player 1 mocked module initialization
    let host_1p = make_static!(HostSideBill::new(
        ExtiInput::new(
            Input::new(p.PD0, Pull::None).degrade(), // VIRT0_INH
            p.EXTI0.degrade(),                       // EXTI0
        ),
        InputPortKind::Inhibit1,
        Output::new(p.PD3.degrade(), Level::Low, Speed::Low), // VIRT0_BSY
        Output::new(p.PD2.degrade(), Level::Low, Speed::Low), // VIRT0_VND
        Output::new(p.PB9.degrade(), Level::Low, Speed::Low), // VIRT0_JAM
        Output::new(p.PB3.degrade(), Level::Low, Speed::Low), // VIRT0_STR
        &ASYNC_INPUT_EVENT_CH,
        &COMMON_TIMING,
    ));
    host_1p.start_tasks(&spawner);
    info!("Game IO PCB side player 1 module loaded");

    // Game IO PCB side player 2 mocked module initialization
    let host_2p = make_static!(HostSideBill::new(
        ExtiInput::new(
            Input::new(p.PA15, Pull::None).degrade(), // VIRT1_INH
            p.EXTI15.degrade(),                       // EXTI15
        ),
        InputPortKind::Inhibit2,
        Output::new(p.PB4.degrade(), Level::Low, Speed::Low), // VIRT1_BSY
        Output::new(p.PC13.degrade(), Level::Low, Speed::Low), // VIRT1_VND
        Output::new(p.PB8.degrade(), Level::Low, Speed::Low), // VIRT1_JAM
        Output::new(p.PB5.degrade(), Level::Low, Speed::Low), // VIRT1_STR
        &ASYNC_INPUT_EVENT_CH,
        &COMMON_TIMING,
    ));
    host_2p.start_tasks(&spawner);
    info!("Game IO PCB side player 2 module loaded");

    // DIP switch module initialization
    // todo! - dip switch gives event when new changes status keep some time.
    let _dipsw = DipSwitch::new(
        Input::new(p.PC6, Pull::Up),  // DIPSW0
        Input::new(p.PA12, Pull::Up), // DIPSW1
        Input::new(p.PA11, Pull::Up), // DIPSW2
        Input::new(p.PA9, Pull::Up),  // DIPSW3
        Input::new(p.PB13, Pull::Up), // DIPSW4
        Input::new(p.PB12, Pull::Up), // DIPSW5
    );
    info!("DIP switch module loaded");

    // LED0 indicator inside of PCB initialization. for debug / indication.
    let led0 = make_static!(BufferedOpenDrain::new(
        Output::new(p.PA4.degrade(), Level::High, Speed::Low),
        &COMMON_LED_TIMING,
    )); // INDICATE0

    // LED1 indicator inside of PCB initialization. for debug / indication.
    let led1 = make_static!(BufferedOpenDrain::new(
        Output::new(p.PA5.degrade(), Level::High, Speed::Low),
        &COMMON_LED_TIMING,
    )); // INDICATE1
    unwrap!(spawner.spawn(buffered_opendrain_spawn(led0)));
    unwrap!(spawner.spawn(buffered_opendrain_spawn(led1)));
    info!("Debug LEDs module loaded");

    led0.alt_forever_blink().await;
    led1.set_high().await;

    // USART2 initialization for CardReaderDevice
    let usart2_rx_buf = make_static!([0u8; components::serial_device::CARD_READER_RX_BUFFER_SIZE]);

    let usart2_config = {
        let mut ret = UsartConfig::default();
        ret.baudrate = 115200;
        ret.assume_noise_free = false;
        ret
    };

    let (usart2_tx, usart2_rx) = {
        let (tx, rx) = Uart::new(
            p.USART2,
            p.PA3,
            p.PA2,
            Irqs,
            p.DMA1_CH2,
            p.DMA1_CH1,
            usart2_config,
        )
        .split();
        (tx, rx.into_ring_buffered(usart2_rx_buf))
    };

    let card_reader = make_static!(CardReaderDevice::new(usart2_tx, usart2_rx));
    unwrap!(spawner.spawn(card_reader_device_spawn(card_reader)));
    info!("Credit card reader module loaded");

    info!("All module loaded, welcome business logic");

    loop {
        // write event based business logic here.

        match card_reader
            .channel
            .try_recv()
            .map_or(None, |x| x.coin_count())
        {
            Some((c, d)) => {
                match d % 10 != 2 {
                    true => &host_1p,
                    false => &host_2p,
                }
                .out_vend
                .tick_tock(c)
                .await;
            }
            None => {}
        }

        match ASYNC_INPUT_EVENT_CH.try_recv().ok() {
            Some(event) => {
                // info!("EVENT comes - port:{}, kind:{}", event.port, event.kind);
                info!("Some event comes");
                match (event.port, InputEventKind::from(event.kind)) {
                    (InputPortKind::Start1P, InputEventKind::Pressed) => {
                        info!("Start1P Pressed");
                        host_1p.out_start.set_high().await;
                    }
                    (InputPortKind::Start1P, InputEventKind::Released) => {
                        info!("Start1P Released");
                        host_1p.out_start.set_low().await;
                    }
                    (InputPortKind::Start2P, InputEventKind::Pressed) => {
                        info!("Start2P Pressed");
                        host_2p.out_start.set_high().await;
                    }
                    (InputPortKind::Start2P, InputEventKind::Released) => {
                        info!("Start2P Released");
                        host_2p.out_start.set_low().await;
                    }
                    (InputPortKind::Inhibit1, InputEventKind::Pressed) => {
                        vend_legacy.out_inhibit.set_high().await;
                    }
                    (InputPortKind::Inhibit1, InputEventKind::Released) => {
                        vend_legacy.out_inhibit.set_low().await;
                    }
                    (InputPortKind::Vend, InputEventKind::LongPressed(duration_10ms)) => {
                        if duration_10ms > 1 {
                            info!("Vend LongPressed");

                            // this is proof of concept, doesn't cover all start1p/2p complex selection
                            host_1p.out_vend.tick_tock(1).await;
                        }
                    }
                    (InputPortKind::Vend, InputEventKind::Pressed) => {
                        info!("Vend Pressed")
                    }
                    (InputPortKind::Vend, InputEventKind::Released) => {
                        info!("Vend Released")
                    }
                    // not implement JAM side
                    _ => {}
                }
            }
            None => {}
        }

        // return preemption on exeucutor, there should be better method.
        Timer::after(Duration::from_millis(100)).await;
    }
}
