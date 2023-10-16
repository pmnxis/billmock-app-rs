/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

pub(crate) const KICC_STX: u8 = 0x02;
pub(crate) const KICC_ACK: u8 = 0x06;
pub(crate) const KICC_NACK: u8 = 0x15;
pub(crate) const KICC_ETX: u8 = 0x03;

pub(crate) const RAW_DATA_ACK: [u8; 3] = [KICC_ACK, KICC_ACK, KICC_ACK];
pub(crate) const RAW_DATA_NACK: [u8; 3] = [KICC_NACK, KICC_NACK, KICC_NACK];
