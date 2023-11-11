<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# DisplayRom

![display hw info screen](https://billmock.gpark.biz/images/dip_switch_disp_hw_info_enus.png)

- To view such information during normal operation, please check and configure the [DIP SWITCH execution mode](./dip_switch.md#application-mode-dip-switch-configuration).

- The information will appear for 10 seconds, divided into four lines, displaying details about ROM and the program:
    - Line 1 : `{Boot Cnt}   Ver:{x.y.z}`
        - **Boot Cnt**: Number of boots for the BillMock hardware. It increments by 1 each time the device is powered off and on.
        - **x.y.z**: Firmware version of the BillMock program [billmock-app-rs](https://github.com/pmnxis/billmock-app-rs).

    - Line 2 : `S/N : {Serial Number}`
        - **Serial Number**: Unique serial number assigned to the BillMock hardware during mass production.

    - Line 3 : `TID : {TID}`
        - **TID**: The unique Terminal ID of the card terminal. It is a value set in the card terminal when connecting it to the payment gateway (PG). It is also a unique identifier in the PG's system.

    - Line 4 : `Uptime : {Uptime} Mins`
        - **Uptime**: Represents the duration the BillMock hardware has been powered on, measured in minutes.

- This feature is available from firmware version `0.2.1` and hardware version `0.4` or `Mini 0.4` onwards. It is not supported on earlier hardware versions.

- From hardware version 0.5 or Mini 0.5 onwards, you can use the SVC button by pressing 2 seconds.
  > ![svc button](https://billmock.gpark.biz/images/svc_button.jpg)

- This feature is available starting from firmware version `0.2.0` and hardware `0.4` or `Mini 0.4` and later. It is not available for previous hardware versions.

- When exiting [DispRom](./feature_disp_rom.md) through the DIP switch on hardware version `0.4` or `Mini 0.4` and above, the display is also shown.
