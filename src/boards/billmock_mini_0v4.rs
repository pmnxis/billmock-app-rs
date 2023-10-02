/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

//! Hardware initialization code for BillMock Hardware Version mini 0.4
//! The code follows on version mini 0.4 schematic
//! https://github.com/pmnxis/BillMock-HW-RELEASE/blob/master/sch/BillMock-Mini-HW-0v4.pdf

use embassy_stm32::crc::{Config as CrcConfig, Crc, InputReverseConfig};
use embassy_stm32::exti::{Channel as HwChannel, ExtiInput};
use embassy_stm32::gpio::{Input, Level, Output, Pin, Pull, Speed};
use embassy_stm32::i2c::I2c;
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::{Config as UsartConfig, Uart};
use embassy_stm32::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

use super::{Hardware, SharedResource};
use super::{PLAYER_1_INDEX, PLAYER_2_INDEX};
use crate::components;
use crate::components::dip_switch::DipSwitch;
use crate::components::eeprom::Novella;
use crate::components::host_side_bill::HostSideBill;
use crate::components::serial_device::CardReaderDevice;
use crate::components::vend_side_bill::VendSideBill;
use crate::semi_layer::buffered_opendrain::BufferedOpenDrain;
use crate::types::buffered_opendrain_kind::BufferedOpenDrainKind;
use crate::types::player::Player;

bind_interrupts!(struct Irqs {
    USART2 => embassy_stm32::usart::InterruptHandler<peripherals::USART2>;
    I2C1 => embassy_stm32::i2c::InterruptHandler<peripherals::I2C1>;
});

static mut USART2_RX_BUF: [u8; components::serial_device::CARD_READER_RX_BUFFER_SIZE] =
    [0u8; components::serial_device::CARD_READER_RX_BUFFER_SIZE];

pub fn hardware_init_mini_0v4(
    p: embassy_stm32::Peripherals,
    shared_resource: &'static SharedResource,
) -> Hardware {
    // USART2 initialization for CardReaderDevice
    let usart2_rx_buf = unsafe { &mut USART2_RX_BUF };

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

    let mut i2c = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        p.DMA1_CH4,
        p.DMA1_CH3,
        Hertz(100_000),
        Default::default(),
    );

    let Ok(crc_config) = CrcConfig::new(InputReverseConfig::Halfword, false, 0x1234) else {
        panic!("Something went horribly wrong")
    };

    let mut crc = Crc::new(p.CRC, crc_config);

    let async_input_event_ch = &shared_resource.async_input_event_ch.channel;

    Hardware {
        vend_sides: [
            VendSideBill::new(
                Player::Player1,
                Output::new(p.PA1.degrade(), Level::Low, Speed::Low), // REAL0_INH
                ExtiInput::new(
                    Input::new(p.PB14, Pull::None).degrade(), // REAL0_VND
                    p.EXTI14.degrade(),                       // EXTI14
                ),
                ExtiInput::new(
                    Input::new(p.PB2, Pull::None).degrade(), // REAL0_STR
                    p.EXTI2.degrade(),                       // EXTI2
                ),
                async_input_event_ch,
                &shared_resource.arcade_players_timing[PLAYER_1_INDEX],
            ),
            VendSideBill::new(
                Player::Player2,
                Output::new(p.PA0.degrade(), Level::Low, Speed::Low), // REAL1_INH
                ExtiInput::new(
                    Input::new(p.PD1, Pull::None).degrade(), // REAL1_VND
                    p.EXTI1.degrade(),                       // EXTI1
                ),
                ExtiInput::new(
                    Input::new(p.PB11, Pull::None).degrade(), // REAL1_STR
                    p.EXTI11.degrade(),                       // EXTI11
                ),
                async_input_event_ch,
                &shared_resource.arcade_players_timing[PLAYER_2_INDEX],
            ),
        ],
        host_sides: [
            HostSideBill::new(
                Player::Player1,
                ExtiInput::new(
                    Input::new(p.PD0, Pull::None).degrade(), // VIRT0_INH
                    p.EXTI0.degrade(),                       // EXTI0
                ),
                Output::new(p.PD3.degrade(), Level::Low, Speed::Low), // VIRT0_BSY
                Output::new(p.PD2.degrade(), Level::Low, Speed::Low), // VIRT0_VND
                Output::new(p.PB4.degrade(), Level::Low, Speed::Low), // VIRT0_JAM
                Output::new(p.PB3.degrade(), Level::Low, Speed::Low), // VIRT0_STR
                async_input_event_ch,
                &shared_resource.arcade_players_timing[PLAYER_1_INDEX],
            ),
            HostSideBill::new(
                Player::Player2,
                ExtiInput::new(
                    Input::new(p.PA15, Pull::None).degrade(), // VIRT1_INH
                    p.EXTI15.degrade(),                       // EXTI15
                ),
                Output::new(p.PC14.degrade(), Level::Low, Speed::Low), // VIRT1_BSY
                Output::new(p.PC13.degrade(), Level::Low, Speed::Low), // VIRT1_VND
                Output::new(p.PB5.degrade(), Level::Low, Speed::Low),  // VIRT1_JAM
                Output::new(p.PC15.degrade(), Level::Low, Speed::Low), // VIRT1_STR
                async_input_event_ch,
                &shared_resource.arcade_players_timing[PLAYER_2_INDEX],
            ),
        ],
        indicators: [
            BufferedOpenDrain::new(
                Output::new(p.PA5.degrade(), Level::High, Speed::Low),
                &shared_resource.indicator_timing,
                BufferedOpenDrainKind::Indicator(1).const_str(),
            ),
            BufferedOpenDrain::new(
                Output::new(p.PA4.degrade(), Level::High, Speed::Low),
                &shared_resource.indicator_timing,
                BufferedOpenDrainKind::Indicator(2).const_str(),
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
        eeprom: Novella::const_new(i2c, crc),
    }
}
