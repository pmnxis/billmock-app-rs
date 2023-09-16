<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Port - Miscellaneous

## Credit Card Reader Port

<table>
<tr>
<td>

![J3](https://billmock.gpark.biz/images/pcb_0v4_port/J3.png)
</td>
<td>

|                |
| -------------- |
|  |
|  |
| **Designator** |
|  J3  |
|  |
|  |
| **Role** |
| Card reader RS232+5V |
|  |
|  |
| **Connector** |
| Molex 5268-04 |
|  |
|  |
| **Housing** |
| Molex 5264-04 |
|  |
|  |
| **Crimp** |
| Molex 5263 |

| **Pin #** | **Pin Name**   | Anotation |
| :-------: | -------------- | --------- |
| `1`       | `GND` |  |
| `2`       | `TXD` | billmock In. |
| `3`       | `RXD` | billmock out. |
| `4`       | `5V`  | 5V Power out |

</td></tr>
</table>

-  5V Power output maximum rating is Peak 2.2A 300mS trip, 1.1A nominal MAX.

------------

## DC Power Jack

<table>
<tr>
<td>

![J1](https://billmock.gpark.biz/images/pcb_0v4_port/J1.png)
</td>
<td>

|                |
| -------------- |
|  |
|  |
| **Designator**     |
|  J1  |
|  |
|  |
| **Role** |
| Extra DC Power Jack <!--별도 DC 전원 잭--> |
|  |
|  |
| **Connector**      |
| DC Jack 5.5pi - 2.0pi |

</td></tr>
</table>

- 12V input is recommended (maximum 16V).
- In addition to the bottom DC jack, it is also recommended to receive power through the 10-pin Molex ports on the bottom left (J5) and bottom right (J4).
- Using the top terminal (J9) or the top 10-pin Molex (J7/J8) for power input is not recommended.

<!--
- 12V 입력을 권장합니다. (최대 16V)
- 하단 DC 잭 이외에 하단 왼쪽(J5), 하단 오른쪽(J4)의 10핀 몰렉스 포트로 받는 것도 권장합니다.
- 상단 터미널 (J9)/ 상단 10핀 몰렉스(J7/J8)로 전원 입력을 하는 것은 비추천합니다.
-->

------------

### Program debugging (SWD/JTAG)

<table>
<tr>
<td>

![DEBUG](https://billmock.gpark.biz/images/pcb_0v4_port/debug_port.png)
</td>
<td>

|                |
| -------------- |
|  |
|  |
| **Role** |
| STM32 SWD |
|  |
|  |
| **Connector** |
| TC2030 |

</td></tr>
</table>

- Detail information is in [BillMock-HW-RELEASE](https://github.com/pmnxis/BillMock-HW-RELEASE)
