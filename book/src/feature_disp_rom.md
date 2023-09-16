<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# DisplayRom

![display rom screen](https://billmock.gpark.biz/images/dip_switch_rom_disp_enus.png)

- To view such information during normal operation, please check and configure the [DIP SWITCH execution mode](./dip_switch.md#application-mode-dip-switch-configuration).

- The information will appear for 10 seconds, divided into four lines, displaying details about ROM and the program:
    - Line 1: `{Git Hash}   {TID}`
        - **Git Hash**: The shortened git commit hash of the BillMock firmware program [billmock-app-rs](https://github.com/pmnxis/billmock-app-rs).
        - **TID**: The unique Terminal ID of the card terminal. It is a value set in the card terminal when connecting it to the payment gateway (PG). It is also a unique identifier in the PG's system.

    - Line 2: `Cards: {1P Count} + {2P Count} = {Credit Sum}`
        - **1P Count**: Accumulated clock counts for Player **1** based on pre-configured clock settings and consumer payments on the card terminal.
        - **2P Count**: Accumulated clock counts for Player **2** based on pre-configured clock settings and consumer payments on the card terminal.
        - **Credit Sum**: The sum of 1P Count and 2P Count, representing credits processed by the card terminal.

    - Line 3: `Bills: {1P Count} + {2P Count} = {Coin Sum}`
        - **1P Count**: Accumulated clock counts for Player **1** based on bill acceptor or coin acceptor clock settings and consumer payments.
        - **2P Count**: Accumulated clock counts for Player **2** based on bill acceptor or coin acceptor clock settings and consumer payments.
        - **Coin Sum**: The sum of 1P Count and 2P Count, representing currency processed by the bill acceptor or coin acceptor.

    - Line 4: `1P: {1P Sum Count}, {2P Sum Count}`
        - **1P Sum Count**: The total clock counts accumulated for Player **1** by the card terminal and bill acceptor (or coin acceptor).</br>It's the sum of 1P Count from the second and third lines.
        - **2P Sum Count**: The total clock counts accumulated for Player **2** by the card terminal and bill acceptor (or coin acceptor).</br>It's the sum of 2P Count from the second and third lines.

- All count numbers are displayed in 6 digits (0 ~ 999,999). If the count exceeds 1,000,000, it will be displayed as 000,000, and the next count after that will increment normally, such as 000,001.

- This feature is available starting from firmware version `0.2.0` and hardware `0.4` or `Mini 0.4` and later. It is not available for previous hardware versions.
