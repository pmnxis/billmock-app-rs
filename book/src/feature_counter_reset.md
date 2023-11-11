<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Counter Reset

![display counter reset screen](https://billmock.gpark.biz/images/dip_switch_disp_reset_rom_enus.png)

- The warning message appears for 10 seconds, and the ROM contents are already initialized when the warning message is displayed.

- The counts displayed in [DispRom](./feature_disp_rom.md) for `P1 Card`, `P2 Card`, `P1 Coin`, `P2 Coin` are reset to 0, but information such as boot count and uptime remains unaffected.

- This feature is available starting from firmware version `0.3.1` and hardware version `0.5` or `Mini 0.5`. The SVC button on hardware version `0.5` or `Mini 0.5` must be held for more than 10 seconds to activate this feature.
  > ![svc button](https://billmock.gpark.biz/images/svc_button.jpg)
