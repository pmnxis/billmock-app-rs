<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Overview

<div><center><a style="font-weight:bold" href="https://billmock.gpark.biz">한국어 매뉴얼(Korean Manual)</a></center></div>

"Billmock" is a system designed to manipulate the currency payment I/O signals that arcade machines receive based on specific conditions. It is primarily used in South Korean arcade machines for tasks such as installing credit card reader or enabling programmable operations for sequential tasks based on various conditions.

To configure it for desired settings on-site, "Billmock" allows preconfigured I/O remapping adjustments through DIP switches. In terms of wiring, it is installed between the HOST GAME PCB and the bill acceptor device.

## Hardware
![Actual BillMock PCB 0v5](https://billmock.gpark.biz/images/BillMockPCB_0v5_mini.jpg)
The hardware revision currently adopted for final mass production is 0.5-MINI, and the software development is also progressing according to this version.

![Actual BillMock PCB 0v4](https://billmock.gpark.biz/images/BillMockPCB_0v4.jpg)
There are three previous hardware revisions available: 0.2, 0.3, and 0.4. The development is focused on compatibility with versions 0.3 and 0.4, which are the ones actively in use. Detailed hardware schematics are here
[BillMock-HW-RELEASE](https://github.com/pmnxis/BillMock-HW-RELEASE)

## Application
The firmware software has been developed in **Rust**, as opposed to the de facto **C** language. While the choice to use Rust is partly based on trust in its reliability, it also serves the purpose of validating its suitability for embedded systems intended for mass production. Therefore, hope to maintain the firmware source code as a precedent, akin to an example code.

## License

This program and the accompanying materials are made available under the terms of the Apache Software License 2.0 which is available at [Apache Software License 2.0](https://www.apache.org/licenses/LICENSE-2.0), or the MIT license which is available at [MIT License](https://opensource.org/licenses/MIT)

Also all of codes are based one MIT or Apache Software License 2.0. But some common *.toml files are based on CC0-1.0 license. (Example Cargo.toml)

