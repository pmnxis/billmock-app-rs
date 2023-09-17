/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

//! Binary type protocol spec example implementation

use serial_arcade_pay::*;

use crate::common::*;

const BYTE_CMD_HEARTBEAT: u8 = 0x01;
const BYTE_CMD_ACK: u8 = 0x02;
const BYTE_CMD_NACK: u8 = 0x03;
const BYTE_CMD_INCOME: u8 = 0x04;
const BYTE_CMD_CHECK_BUSY_STATE: u8 = 0x05; // Billmock -> Credit
const BYTE_CMD_SET_BUSY_STATE: u8 = 0x05;
const BYTE_CMD_CHECK_INHIBIT: u8 = 0x06;
const BYTE_CMD_FAILED_INCOME: u8 = 0x07;
const BYTE_CMD_SET_INHIBIT: u8 = 0x16; // Billmock -> Credit

const BYTE_OPT_NONE_MARK: u8 = 0x00;
const BYTE_OPT_SOME_MARK: u8 = 0x01;

const INNER_DATA_CMD_OFFSET: usize = 0;
const INNER_DATA_TEXT_OFFSET: usize = 1;
const INNER_DATA_INCOME_PLAYER_OPT_OFFSET: usize = 2;
// const INNER_DATA_INCOME_PLAYER_VAL_OFFSET: usize = 3; // 1 byte
const INNER_DATA_INCOME_PRICE_OPT_OFFSET: usize = 4;
// const INNER_DATA_INCOME_PRICE_VAL_OFFSET: usize = 5; // 4 byte
const INNER_DATA_INCOME_SIGCNT_OPT_OFFSET: usize = 9;
// const INNER_DATA_INCOME_SIGCNT_VAL_OFFSET: usize = 10; // 2 byte
const INNER_DATA_INCOME_DURATION_OPT_OFFSET: usize = 12;
// const INNER_DATA_INCOME_DURATION_VAL_OFFSET: usize = 13; // 2 byte
const INNER_DATA_INCOME_LENGTH: usize = 15;

const INNER_DATA_MINIMUM_LEN: usize = 1;

enum ExampleParseError {
    InternalError,
}

impl From<ExampleParseError> for Option<GenericPaymentRecv> {
    fn from(_value: ExampleParseError) -> Option<GenericPaymentRecv> {
        Some(GenericPaymentRecv::Unknown)
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) type ExampleByteRecv = GenericPaymentRecv;

impl AnyExampleDevice for ExampleByteRecv {
    fn test_type(inner_data: &[u8]) -> bool {
        if inner_data.len() < INNER_DATA_MINIMUM_LEN {
            return false;
        }

        matches!(
            inner_data[INNER_DATA_CMD_OFFSET],
            BYTE_CMD_HEARTBEAT
                | BYTE_CMD_ACK
                | BYTE_CMD_NACK
                | BYTE_CMD_INCOME
                | BYTE_CMD_SET_BUSY_STATE
                | BYTE_CMD_CHECK_INHIBIT
                | BYTE_CMD_FAILED_INCOME
        )
    }

    fn parse(inner_data: &[u8]) -> Option<Self> {
        fn opt_u8(inner_data: &[u8], start_offset: usize) -> Result<Option<u8>, ExampleParseError> {
            if inner_data[start_offset] == BYTE_OPT_NONE_MARK {
                Ok(None)
            } else if inner_data[start_offset] == BYTE_OPT_SOME_MARK {
                Ok(Some(inner_data[start_offset + 1]))
            } else {
                Err(ExampleParseError::InternalError)
            }
        }

        fn opt_u16(
            inner_data: &[u8],
            start_offset: usize,
        ) -> Result<Option<u16>, ExampleParseError> {
            if inner_data[start_offset] == BYTE_OPT_NONE_MARK {
                Ok(None)
            } else if inner_data[start_offset] == BYTE_OPT_SOME_MARK {
                Ok(Some(
                    ((inner_data[start_offset + 1] as u16) << 8)
                        | (inner_data[start_offset + 2] as u16),
                ))
            } else {
                Err(ExampleParseError::InternalError)
            }
        }

        fn opt_u32(
            inner_data: &[u8],
            start_offset: usize,
        ) -> Result<Option<u32>, ExampleParseError> {
            if inner_data[start_offset] == BYTE_OPT_NONE_MARK {
                Ok(None)
            } else if inner_data[start_offset] == BYTE_OPT_SOME_MARK {
                Ok(Some(
                    ((inner_data[start_offset + 1] as u32) << 24)
                        | ((inner_data[start_offset + 2] as u32) << 16)
                        | ((inner_data[start_offset + 3] as u32) << 8)
                        | (inner_data[start_offset + 4] as u32),
                ))
            } else {
                Err(ExampleParseError::InternalError)
            }
        }

        fn inner_income_parse(inner_data: &[u8]) -> Result<GenericIncomeInfo, ExampleParseError> {
            Ok(GenericIncomeInfo {
                player: opt_u8(inner_data, INNER_DATA_INCOME_PLAYER_OPT_OFFSET)?,
                price: opt_u32(inner_data, INNER_DATA_INCOME_PRICE_OPT_OFFSET)?,
                signal_count: opt_u16(inner_data, INNER_DATA_INCOME_SIGCNT_OPT_OFFSET)?,
                pulse_duration: opt_u16(inner_data, INNER_DATA_INCOME_DURATION_OPT_OFFSET)?,
            })
        }

        // assume the text is only include DATA field
        if inner_data.len() < INNER_DATA_MINIMUM_LEN {
            return None;
        }

        match inner_data[INNER_DATA_CMD_OFFSET] {
            BYTE_CMD_HEARTBEAT => Some(Self::Heartbeat),
            BYTE_CMD_ACK => Some(Self::Ack),
            BYTE_CMD_NACK => Some(Self::Nack),
            BYTE_CMD_INCOME => {
                if inner_data.len() != INNER_DATA_INCOME_LENGTH {
                    return Some(Self::Unknown);
                }

                match inner_income_parse(inner_data) {
                    Ok(x) => Some(Self::Income(x)),
                    Err(_x) => Some(Self::Unknown),
                }
            }
            BYTE_CMD_SET_BUSY_STATE => Some(Self::SetBusyState(!matches!(
                inner_data[INNER_DATA_TEXT_OFFSET],
                0
            ))),
            BYTE_CMD_CHECK_INHIBIT => Some(Self::CheckInhibit(
                TinyGenericInhibitInfo::new_from_u8(inner_data[INNER_DATA_TEXT_OFFSET]),
            )),
            BYTE_CMD_FAILED_INCOME => Some(Self::Failed),
            _ => None,
        }
    }

    fn request_form(request: &GenericPaymentRequest, tx_inner_buff: &mut [u8]) -> Option<usize> {
        let (tx_cmd, len_u8) = match request {
            GenericPaymentRequest::Heartbeat | GenericPaymentRequest::ResponseInitialHandshake => {
                (BYTE_CMD_HEARTBEAT, 1u8)
            }
            GenericPaymentRequest::Ack => (BYTE_CMD_ACK, 1u8),
            GenericPaymentRequest::Nack => (BYTE_CMD_NACK, 1u8),
            GenericPaymentRequest::CheckBusy => (BYTE_CMD_CHECK_BUSY_STATE, 1u8),
            GenericPaymentRequest::CheckInhibit => (BYTE_CMD_CHECK_INHIBIT, 1u8),
            GenericPaymentRequest::SetGlobalInhibit(x) => {
                tx_inner_buff[INNER_DATA_TEXT_OFFSET] = match x {
                    true => 0b1111,
                    false => 0,
                };
                (BYTE_CMD_SET_INHIBIT, 2u8)
            }
            GenericPaymentRequest::SetInhibit(x) => {
                tx_inner_buff[INNER_DATA_TEXT_OFFSET] = x.get_raw();
                (BYTE_CMD_SET_INHIBIT, 2u8)
            }
            GenericPaymentRequest::DisplayRom => {
                // skip implementation because, this is just example code
                unimplemented!("DisplayRom not implementated");
            }
        };

        tx_inner_buff[INNER_DATA_CMD_OFFSET] = tx_cmd;

        Some(len_u8 as usize)
    }

    fn degrade(&self) -> GenericPaymentRecv {
        self.clone().into()
    }

    fn self_degrade(self) -> GenericPaymentRecv {
        self.into()
    }
}
