/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod components;
mod semi_layer;
mod types;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::peripherals::*;
use embassy_stm32::usart::{self, BufferedInterruptHandler, BufferedUart, Config as UartConfig};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_io::asynch::{Read, Write};
use semi_layer::buffered_opendrain::BufferedOpenDrain;
use {defmt_rtt as _, panic_probe as _};

use crate::components::dip_switch::DipSwitch;
use crate::components::host_side_bill::HostSideBill;
use crate::components::start_button::StartButton;
use crate::components::vend_side_bill::VendSideBill;
use crate::semi_layer::buffered_wait::{InputEventChannel, InputPortKind};
use crate::semi_layer::timing::{DualPoleToggleTiming, SharedToggleTiming, ToggleTiming};

bind_interrupts!(struct Irqs {
    USART2 => BufferedInterruptHandler<peripherals::USART2>;
});

// Test queue stuff

// End of test queue stuff

const CARD_GADGET_RX_BUFFER_SIZE: usize = 768; // most of packet is 320~330 bytes
const CARD_GADGET_TX_BUFFER_SIZE: usize = 128; // most of pakcet is 6~12 bytes, but some uncommon command can be long

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
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let vend_legacy = VendSideBill::new(
        Output::new(p.PB0, Level::Low, Speed::Low), // REAL_INH
        (
            ExtiInput::new(Input::new(p.PB2, Pull::None), p.EXTI2),
            InputPortKind::Vend,
        ), // REAL_VND
        (
            ExtiInput::new(Input::new(p.PB14, Pull::None), p.EXTI14),
            InputPortKind::Jam,
        ), // REAL_JAM (moved)
        &ASYNC_INPUT_EVENT_CH,
        &COMMON_TIMING,
    );

    let start_1p = StartButton::new(
        (
            ExtiInput::new(Input::new(p.PB11, Pull::None), p.EXTI11),
            InputPortKind::Start1P,
        ), // REAL_STR0
        Output::new(p.PA0, Level::Low, Speed::Low), // VIRT0_VND
        &ASYNC_INPUT_EVENT_CH,
        &START_BUTTON_LED_TIMING,
    );

    let start_2p = StartButton::new(
        (
            ExtiInput::new(Input::new(p.PD1, Pull::None), p.EXTI1),
            InputPortKind::Start2P,
        ), // REAL_STR1
        Output::new(p.PA1, Level::Low, Speed::Low), // VIRT1_VND
        &ASYNC_INPUT_EVENT_CH,
        &START_BUTTON_LED_TIMING,
    );

    let host_1p = HostSideBill::new(
        (
            ExtiInput::new(Input::new(p.PD0, Pull::None), p.EXTI0),
            InputPortKind::Inhibit1,
        ), // VIRT0_INH
        Output::new(p.PD3, Level::Low, Speed::Low), // VIRT0_BSY
        Output::new(p.PD2, Level::Low, Speed::Low), // VIRT0_VND
        Output::new(p.PB9, Level::Low, Speed::Low), // VIRT0_JAM
        Output::new(p.PB3, Level::Low, Speed::Low), // VIRT0_STR
        &ASYNC_INPUT_EVENT_CH,
        &COMMON_TIMING,
    );

    let host_2p = HostSideBill::new(
        (
            ExtiInput::new(Input::new(p.PA15, Pull::None), p.EXTI15),
            InputPortKind::Inhibit2,
        ), // VIRT1_INH
        Output::new(p.PB4, Level::Low, Speed::Low), // VIRT1_BSY
        Output::new(p.PC13, Level::Low, Speed::Low), // VIRT1_VND
        Output::new(p.PB8, Level::Low, Speed::Low), // VIRT1_JAM
        Output::new(p.PB5, Level::Low, Speed::Low), // VIRT1_STR
        &ASYNC_INPUT_EVENT_CH,
        &COMMON_TIMING,
    );

    let dipsw = DipSwitch::new(
        Input::new(p.PC6, Pull::Up),  // DIPSW0
        Input::new(p.PA12, Pull::Up), // DIPSW1
        Input::new(p.PA11, Pull::Up), // DIPSW2
        Input::new(p.PA9, Pull::Up),  // DIPSW3
        Input::new(p.PB13, Pull::Up), // DIPSW4
        Input::new(p.PB12, Pull::Up), // DIPSW5
    );

    let (mut led0, mut led1) = (
        BufferedOpenDrain::new(
            Output::new(p.PA4, Level::High, Speed::Low),
            &COMMON_LED_TIMING,
        ), // INDICATE0
        BufferedOpenDrain::new(
            Output::new(p.PA5, Level::High, Speed::Low),
            &COMMON_LED_TIMING,
        ), // INDICATE1
    );

    // temporary usart configuration
    let mut tx_buffer = [0u8; CARD_GADGET_TX_BUFFER_SIZE];
    let mut rx_buffer = [0u8; CARD_GADGET_RX_BUFFER_SIZE];
    let uart2_config = UartConfig::default();

    let mut usart2: BufferedUart<'_, USART2> = BufferedUart::new(
        p.USART2,
        Irqs,
        p.PA3, // R_RXD
        p.PA2, // T_RXD
        &mut tx_buffer,
        &mut rx_buffer,
        uart2_config,
    );

    // usart2.write_all(b"Hello Embassy World!\r\n").await.unwrap();
    info!("wrote Hello, starting echo");
    let mut buf = [0; CARD_GADGET_RX_BUFFER_SIZE];

    loop {
        info!("high");
        // led.set_high();
        Timer::after(Duration::from_millis(300)).await;

        info!("low");
        // led.set_low();
        Timer::after(Duration::from_millis(300)).await;

        match usart2.read(&mut buf).await {
            Ok(cnt) => info!("uart read {} : {:?}", cnt, buf[0..cnt]),
            _ => {}
        }
    }
}
