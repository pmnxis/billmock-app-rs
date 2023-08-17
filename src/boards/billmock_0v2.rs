/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use embassy_stm32::exti::{Channel as HwChannel, ExtiInput};
use embassy_stm32::gpio::{Input, Level, Output, Pin, Pull, Speed};
use embassy_stm32::usart::{Config as UsartConfig, Uart};
use embassy_stm32::{bind_interrupts, peripherals};
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

use super::PLAYER_1_INDEX;
use super::{Hardware, SharedResource};
use crate::components;
use crate::components::dip_switch::DipSwitch;
use crate::components::host_side_bill::HostSideBill;
use crate::components::serial_device::CardReaderDevice;
// use crate::components::start_button::StartButton;
use crate::components::vend_side_bill::VendSideBill;
use crate::semi_layer::buffered_opendrain::BufferedOpenDrain;
use crate::semi_layer::buffered_wait::InputPortKind;

bind_interrupts!(struct Irqs {
    USART2 => embassy_stm32::usart::InterruptHandler<peripherals::USART2>;
});

pub fn hardware_init_0v2(
    p: embassy_stm32::Peripherals,
    shared_resource: &'static SharedResource,
) -> Hardware {
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

    // 2023-08-17 , PA0 (Start1P Port out is not used anymore)

    Hardware {
        vend_sides: [
            VendSideBill::new(
                Output::new(p.PB0.degrade(), Level::Low, Speed::Low), // REAL_INH
                ExtiInput::new(
                    Input::new(p.PB2, Pull::None).degrade(), // REAL_VND
                    p.EXTI2.degrade(),                       // EXTI2
                ),
                InputPortKind::Vend1P,
                ExtiInput::new(
                    Input::new(p.PB14, Pull::None).degrade(), // REAL_JAM (moved from PB15 at HW v0.3)
                    p.EXTI14.degrade(),                       // EXTI14
                ),
                InputPortKind::StartJam1P,
                &shared_resource.async_input_event_ch,
                &shared_resource.arcade_players_timing[PLAYER_1_INDEX],
            ),
            VendSideBill::new(
                Output::new(p.PA1.degrade(), Level::Low, Speed::Low), // REAL_INH
                ExtiInput::new(
                    Input::new(p.PD1, Pull::None).degrade(), // REAL_STR1
                    p.EXTI1.degrade(),                       // EXTI1
                ),
                InputPortKind::Vend2P,
                ExtiInput::new(
                    Input::new(p.PB11, Pull::None).degrade(), // REAL_STR0
                    p.EXTI11.degrade(),                       // EXTI11
                ),
                InputPortKind::StartJam2P,
                &shared_resource.async_input_event_ch,
                &shared_resource.arcade_players_timing[PLAYER_1_INDEX],
            ),
        ],
        host_sides: [
            HostSideBill::new(
                ExtiInput::new(
                    Input::new(p.PD0, Pull::None).degrade(), // VIRT0_INH
                    p.EXTI0.degrade(),                       // EXTI0
                ),
                InputPortKind::Inhibit1,
                Output::new(p.PD3.degrade(), Level::Low, Speed::Low), // VIRT0_BSY
                Output::new(p.PD2.degrade(), Level::Low, Speed::Low), // VIRT0_VND
                Output::new(p.PB9.degrade(), Level::Low, Speed::Low), // VIRT0_JAM
                Output::new(p.PB3.degrade(), Level::Low, Speed::Low), // VIRT0_STR
                &shared_resource.async_input_event_ch,
                &shared_resource.arcade_players_timing[PLAYER_1_INDEX],
            ),
            HostSideBill::new(
                ExtiInput::new(
                    Input::new(p.PA15, Pull::None).degrade(), // VIRT1_INH
                    p.EXTI15.degrade(),                       // EXTI15
                ),
                InputPortKind::Inhibit2,
                Output::new(p.PB4.degrade(), Level::Low, Speed::Low), // VIRT1_BSY
                Output::new(p.PC13.degrade(), Level::Low, Speed::Low), // VIRT1_VND
                Output::new(p.PB8.degrade(), Level::Low, Speed::Low), // VIRT1_JAM
                Output::new(p.PB5.degrade(), Level::Low, Speed::Low), // VIRT1_STR
                &shared_resource.async_input_event_ch,
                &shared_resource.arcade_players_timing[PLAYER_1_INDEX],
            ),
        ],
        indicators: [
            BufferedOpenDrain::new(
                Output::new(p.PA4.degrade(), Level::High, Speed::Low),
                &shared_resource.indicator_timing,
            ),
            BufferedOpenDrain::new(
                Output::new(p.PA5.degrade(), Level::High, Speed::Low),
                &shared_resource.indicator_timing,
            ),
        ],
        dipsw: DipSwitch::new(
            Input::new(p.PC6.degrade(), Pull::Up),  // DIPSW0
            Input::new(p.PA12.degrade(), Pull::Up), // DIPSW1
            Input::new(p.PA11.degrade(), Pull::Up), // DIPSW2
            Input::new(p.PA9.degrade(), Pull::Up),  // DIPSW3
            Input::new(p.PB13.degrade(), Pull::Up), // DIPSW4
            Input::new(p.PB12.degrade(), Pull::Up), // DIPSW5
        ),
        card_reader: CardReaderDevice::new(usart2_tx, usart2_rx),
    }
}
