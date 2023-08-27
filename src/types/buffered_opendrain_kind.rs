/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use crate::types::const_convert::ConstInto;
use crate::types::player::Player;

#[cfg(debug_assertions)]
const OUTPUT_NAME_STRS: [&str; 15] = [
    "HostOut_P1-    Busy", // 0
    "HostOut_P2-    Busy",
    "HostOut_P1-    Vend", // 2
    "HostOut_P2-    Vend",
    "HostOut_P1-     Jam", // 4
    "HostOut_P2-     Jam",
    "HostOut_P1-   Start", // 6
    "HostOut_P2-   Start",
    "VendOut_P1- Inhibit", // 8
    "VendOut_P2- Inhibit",
    "VendOut_P1-StartLED", // 10, would be not use
    "VendOut_P2-StartLED", // would be not use
    "     LED1-Indicator", // 12
    "     LED2-Indicator",
    "Unknown",
];

#[cfg(not(debug_assertions))]
#[rustfmt::skip]
/// reduced output names
const OUTPUT_NAME_STRS: [&str; 15] = [
    "P1H-oBSY", // 0
    "P2H-oBSY",
    "P1H-oVND", // 2
    "P2H-oVND",
    "P1H-oJAM", // 4
    "P2H-oJAM",
    "P1H-oSTR", // 6
    "P2H-oSTR",
    "P1V-oINH", // 8
    "P2V-oINH",
    "P1V-oSLD", // 10, would be not use
    "P2V-oSLD", // would be not use
    "LED1",     // 12
    "LED2",
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

impl ConstInto<&'static str> for &BufferedOpenDrainKind {
    fn const_into(self) -> &'static str {
        OUTPUT_NAME_STRS[self.get_str_idx()]
    }
}

impl BufferedOpenDrainKind {
    pub const fn get_str_idx(&self) -> usize {
        /*
            // following code consume 444 bytes additionaly
            match self {
                Self::HostSideOutBusy(Player::Player1) => 0,
                Self::HostSideOutBusy(Player::Player2) => 1,
                Self::HostSideOutVend(Player::Player1) => 2,
                Self::HostSideOutVend(Player::Player2) => 3,
                Self::HostSideOutJam(Player::Player1) => 4,
                Self::HostSideOutJam(Player::Player2) => 5,
                Self::HostSideOutStart(Player::Player1) => 6,
                Self::HostSideOutStart(Player::Player2) => 7,
                Self::VendSideInhibit(Player::Player1) => 8,
                Self::VendSideInhibit(Player::Player2) => 9,
                Self::VendSideStartLed(Player::Player1) => 10,
                Self::VendSideStartLed(Player::Player2) => 11,
                Self::Indicator(0) => 12,
                Self::Indicator(1) => 13,
                _ => 14,
            }
        */

        let (a, b): (u8, u8) = match self {
            Self::HostSideOutBusy(p) => (0, *p as u8),
            Self::HostSideOutVend(p) => (2, *p as u8),
            Self::HostSideOutJam(p) => (4, *p as u8),
            Self::HostSideOutStart(p) => (6, *p as u8),
            Self::VendSideInhibit(p) => (8, *p as u8),
            Self::VendSideStartLed(p) => (10, *p as u8),
            Self::Indicator(p) => (12, *p),
            _ => (14, 0),
        };

        // cannot use alpha.max(beta) in const fn
        let b = if b == 0 {
            1
        } else if 2 < b {
            2
        } else {
            b
        };
        (a + b - 1) as usize
    }

    pub const fn const_str(&self) -> &'static str {
        OUTPUT_NAME_STRS[self.get_str_idx()]
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
