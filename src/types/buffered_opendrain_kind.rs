/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use crate::types::const_convert::ConstInto;
use crate::types::player::Player;

pub const OUTPUT_NAME_STRS: [&str; 15] = [
    "P1_HostSideOut-Busy", // 0
    "P2_HostSideOut-Busy",
    "P1_HostSideOut-Vend", // 2
    "P2_HostSideOut-Vend",
    "P1_HostSideOut-Jam", // 4
    "P2_HostSideOut-Jam",
    "P1_HostSideOut-Start", // 6
    "P2_HostSideOut-Start",
    "P1_VendSideOut-Inhibit", // 8
    "P2_VendSideOut-Inhibit",
    "P1_VendSideOut-StartLED", // 10, would be not use
    "P2_VendSideOut-StartLED", // would be not use
    "LED1-Indicator",          // 12
    "LED2-Indicator",
    "Unknown",
];

#[allow(unused)]
pub enum BufferedOpenDrainKind {
    HostSideOutBusy(Player),
    HostSideOutVend(Player),
    HostSideOutJam(Player),
    HostSideOutStart(Player),
    VendSideInhibit(Player),
    VendSideStartLed(Player), // deprecated in 0.3
    Indicator(u8),
    Unknown,
}

impl ConstInto<&'static str> for BufferedOpenDrainKind {
    fn const_into(self) -> &'static str {
        // hard-code is efficient in this case
        match self {
            Self::HostSideOutBusy(Player::Player1) => OUTPUT_NAME_STRS[0],
            Self::HostSideOutBusy(Player::Player2) => OUTPUT_NAME_STRS[1],
            Self::HostSideOutVend(Player::Player1) => OUTPUT_NAME_STRS[2],
            Self::HostSideOutVend(Player::Player2) => OUTPUT_NAME_STRS[3],
            Self::HostSideOutJam(Player::Player1) => OUTPUT_NAME_STRS[4],
            Self::HostSideOutJam(Player::Player2) => OUTPUT_NAME_STRS[5],
            Self::HostSideOutStart(Player::Player1) => OUTPUT_NAME_STRS[6],
            Self::HostSideOutStart(Player::Player2) => OUTPUT_NAME_STRS[7],
            Self::VendSideInhibit(Player::Player1) => OUTPUT_NAME_STRS[8],
            Self::VendSideInhibit(Player::Player2) => OUTPUT_NAME_STRS[9],
            Self::VendSideStartLed(Player::Player1) => OUTPUT_NAME_STRS[10],
            Self::VendSideStartLed(Player::Player2) => OUTPUT_NAME_STRS[11],
            Self::Indicator(0) => OUTPUT_NAME_STRS[12],
            Self::Indicator(1) => OUTPUT_NAME_STRS[13],
            _ => OUTPUT_NAME_STRS[14],
        }
    }
}

impl ConstInto<&'static str> for &BufferedOpenDrainKind {
    fn const_into(self) -> &'static str {
        // hard-code is efficient in this case
        match self {
            BufferedOpenDrainKind::HostSideOutBusy(Player::Player1) => OUTPUT_NAME_STRS[0],
            BufferedOpenDrainKind::HostSideOutBusy(Player::Player2) => OUTPUT_NAME_STRS[1],
            BufferedOpenDrainKind::HostSideOutVend(Player::Player1) => OUTPUT_NAME_STRS[2],
            BufferedOpenDrainKind::HostSideOutVend(Player::Player2) => OUTPUT_NAME_STRS[3],
            BufferedOpenDrainKind::HostSideOutJam(Player::Player1) => OUTPUT_NAME_STRS[4],
            BufferedOpenDrainKind::HostSideOutJam(Player::Player2) => OUTPUT_NAME_STRS[5],
            BufferedOpenDrainKind::HostSideOutStart(Player::Player1) => OUTPUT_NAME_STRS[6],
            BufferedOpenDrainKind::HostSideOutStart(Player::Player2) => OUTPUT_NAME_STRS[7],
            BufferedOpenDrainKind::VendSideInhibit(Player::Player1) => OUTPUT_NAME_STRS[8],
            BufferedOpenDrainKind::VendSideInhibit(Player::Player2) => OUTPUT_NAME_STRS[9],
            BufferedOpenDrainKind::VendSideStartLed(Player::Player1) => OUTPUT_NAME_STRS[10],
            BufferedOpenDrainKind::VendSideStartLed(Player::Player2) => OUTPUT_NAME_STRS[11],
            BufferedOpenDrainKind::Indicator(0) => OUTPUT_NAME_STRS[12],
            BufferedOpenDrainKind::Indicator(1) => OUTPUT_NAME_STRS[13],
            _ => OUTPUT_NAME_STRS[14],
        }
    }
}

impl BufferedOpenDrainKind {
    pub const fn const_str(&self) -> &'static str {
        match self {
            BufferedOpenDrainKind::HostSideOutBusy(Player::Player1) => OUTPUT_NAME_STRS[0],
            BufferedOpenDrainKind::HostSideOutBusy(Player::Player2) => OUTPUT_NAME_STRS[1],
            BufferedOpenDrainKind::HostSideOutVend(Player::Player1) => OUTPUT_NAME_STRS[2],
            BufferedOpenDrainKind::HostSideOutVend(Player::Player2) => OUTPUT_NAME_STRS[3],
            BufferedOpenDrainKind::HostSideOutJam(Player::Player1) => OUTPUT_NAME_STRS[4],
            BufferedOpenDrainKind::HostSideOutJam(Player::Player2) => OUTPUT_NAME_STRS[5],
            BufferedOpenDrainKind::HostSideOutStart(Player::Player1) => OUTPUT_NAME_STRS[6],
            BufferedOpenDrainKind::HostSideOutStart(Player::Player2) => OUTPUT_NAME_STRS[7],
            BufferedOpenDrainKind::VendSideInhibit(Player::Player1) => OUTPUT_NAME_STRS[8],
            BufferedOpenDrainKind::VendSideInhibit(Player::Player2) => OUTPUT_NAME_STRS[9],
            BufferedOpenDrainKind::VendSideStartLed(Player::Player1) => OUTPUT_NAME_STRS[10],
            BufferedOpenDrainKind::VendSideStartLed(Player::Player2) => OUTPUT_NAME_STRS[11],
            BufferedOpenDrainKind::Indicator(0) => OUTPUT_NAME_STRS[12],
            BufferedOpenDrainKind::Indicator(1) => OUTPUT_NAME_STRS[13],
            _ => OUTPUT_NAME_STRS[14],
        }
    }
}

impl defmt::Format for BufferedOpenDrainKind {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "{}",
            <&Self as ConstInto<&'static str>>::const_into(self)
        )
    }
}
