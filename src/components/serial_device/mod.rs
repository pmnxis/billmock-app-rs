/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

// use embassy_embedded_hal::{adapter::BlockingAsync, adapter::YieldingAsync, SetConfig};
// use embassy_stm32::usart::BufferedUart;
// use embedded_hal_async::serial::*;

const CARD_GADGET_RX_BUFFER_SIZE: usize = 768; // most of packet is 320~330 bytes
const CARD_GADGET_TX_BUFFER_SIZE: usize = 128; // most of pakcet is 6~12 bytes, but some uncommon command can be long
