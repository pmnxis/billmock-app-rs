<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# `billmock-app-rs`

Rust Embedded firmware using **rust embedded-hal**<sup>[1](#footnote_1)</sup> and **embassy-rs**<sup>[2](#footnote_1)</sup> on **STM32G030C8**<sup>[3](#footnote_1)</sup> MCU for Billmock product.

Used rust experimentally.

This repository is aiming three goal.
One for development of production firmware and second is making a proof of concept that rust embedded is usable for actual embedded production. And last goal is setting some example about production-rust-embedded-code.

**This project is currently under development, with ongoing QA testing and some optimization remaining.**

## Billmock
Detail documentation is here [BillMock Manual](https://billmock.pmnxis.net/)

It is hardware and software for the purpose of converting I/O signals related to money payment of arcade game machines for compatibility.

This project began development at the request of **GPARK Co., Ltd**<sup>[4](#footnote_1)</sup> and was designed and developed for the using credit card readers in arcade game machine and compatibility of existing payment systems (open-drain-based), and is open to the public except for code in the NDA area.
The project has been granted an open source disclosure except of NDA era.

## Target Hardware
Based on BillMock-HW 0.5 mini, 0.4 mini and 0.4. <br/>
0.2 and 0.3 HW bring-up codes are still left for recyle the old PCB.
- 0.2 HW has different gpio configuration compare to latest boards.
- 0.3 HW has minor bugs, floating on VendSide-Inhibit and missing net route on VendSide-1P-StartJam.
- 0.4 HW fixed 0.3 HW bugs.
- 0.4 HW mini reduced BOM for mass-manufacturing
- 0.5 HW mini added tact switch for SVC mode call

Current default HW is `0.5 mini`. If need to use old 0.4 or 0.4 mini, following below command lines
```sh
cargo build --features hw_0v4 --no-default-features --release
cargo build --features hw_mini_0v4 --no-default-features --release
```

### Target hardware image
![Actual BillMock PCB 0v3](https://billmock.gpark.biz/images/BillMockPCB_0v4.jpg)

### Hardware design
BillMock hardware schematic repository (only pdf)
https://github.com/pmnxis/BillMock-HW-RELEASE

The schematic printed in PDF is distributed under CC BY-SA 3.0, but the actual Gerber files and project files are private.

#### v 0.5 Mini (2023-10-24)
[BillMock-Mini-HW-0v5.pdf](https://github.com/pmnxis/BillMock-HW-RELEASE/blob/master/sch/BillMock-Mini-HW-0v5.pdf)

#### v 0.4 Mini (2023-09-12 or 2023-09-13)
[BillMock-Mini-HW-0v4.pdf](https://github.com/pmnxis/BillMock-HW-RELEASE/blob/master/sch/BillMock-Mini-HW-0v4.pdf)

#### v 0.4 (2023-09-08)
[BillMock-HW-0v4.pdf](https://github.com/pmnxis/BillMock-HW-RELEASE/blob/master/sch/BillMock-HW-0v4.pdf)

#### ~~v 0.3 (2023-08-11)~~ - DEPRECATED
~~[BillMock-HW-0v3.pdf](https://github.com/pmnxis/BillMock-HW-RELEASE/blob/master/sch/BillMock-HW-0v3.pdf)~~

#### ~~v 0.2 (2023-06-13)~~ - DEPRECATED
~~[BillMock-HW-0v2.pdf](https://github.com/pmnxis/BillMock-HW-RELEASE/blob/master/sch/BillMock-HW-0v2.pdf)~~


## Feature diagram
![BillMock feature diagram](https://billmock.gpark.biz/images/billmock_logic_diagram.png)

## Dependencies
See details here [dependencies](docs/dependencies.md)

### NDA Dependencies
- [Dependency Injection for card reader](https://billmock.pmnxis.net/dev/dependency_injection.html)
- [Detail stories](docs/SerialDevice.md)

## License
This program and the accompanying materials are made available under the terms
of the Apache Software License 2.0 which is available at
https://www.apache.org/licenses/LICENSE-2.0, or the MIT license which is 
available at https://opensource.org/licenses/MIT

Also all of codes are based one MIT or Apache Software License 2.0. But some common *.toml files are based on CC0-1.0 license. (Example Cargo.toml)

## Footnote
<a name="footnote_1">1</a> `rust embedded-hal` is hardware abstraction layer written in rust<br>
( https://github.com/rust-embedded/embedded-hal )<br><br>

<a name="footnote_2">2</a> `embassy-rs` is rust embedded framework<br>
( https://github.com/embassy-rs/embassy )<br><br>

<a name="footnote_3">3</a> `STM32G030C8` is STMicroelectronics' MCU with ARM-Cortex M0+ , 64KiB Flash and 8KiB SRAM. <br>
( https://www.st.com/en/microcontrollers-microprocessors/stm32g030c8.html ) <br><br>

<a name="footnote_4">4</a>: `GPARK Co., Ltd.` is a company in South Korea that operates the arcade game industry.
