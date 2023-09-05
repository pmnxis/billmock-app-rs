<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Host side port map

## Host Side Quick Terminal
<table>
<tr>
<td>

![J6](./images/pcb_0v4_port/J6.png)
</td>
<td>

|                |                |
| -------------- | -------------- |
| Designator     | J6  |
|                | terminal that emulates towards the GAME IO side <!--가상 스타트/코인기/지폐기 포트 (터미널 타입)--> |
| Connector      | 141R-2.54-8P |

| **Pin #** | **Pin Name**   | Annotation                                                |
| :-------: | -------------- | --------------------------------------------------------- |
| `1`       | `GND`          | Product - Pole Power Input (Add' Output) |
| `2`       | `GND`          | Product - Pole Power Input (add' Output) |
| `3`       | `12V`          | Product + Pole Power Input                               |
| `4`       | `V1-INHIBIT`   | Inhibit Input Signal from 1P GAME I/O |
| `5`       | `V1-VEND`      | Emulated Bill acceptor 1P Coin Insertion Output Signal |
| `6`       | `V2-VEND`      | Emulated Bill acceptor 2P Coin Insertion Output Signal |
| `7`       | `V1-START`     | Emulated 1P Start Button Output Signal |
| `8`       | `V2-START`     | Emulated 2P Start Button Output Signal |

<!--
| **Pin #** | **Pin Name**   | Anotation |
| :-------: | -------------- | --------- |
| `1`       | `GND`  |  본 제품 -극 전원 입력 (전원 출력 겸용) |
| `2`       | `GND`  |  본 제품 -극 전원 입력 (전원 출력 겸용) |
| `3`       | `12V`  |  본 제품 +극 전원 입력 |
| `4`       | `V1-INHIBIT` | 가상1P 코인기/지폐기 입수금지 <U>에뮬레이션</U> 입력 신호 |
| `5`       | `V1-VEND`    | 가상1P 코인기/지폐기 진권신호 <U>에뮬레이션</U> 출력 신호 |
| `6`       | `V2-VEND`    | 가상2P 코인기/지폐기 진권신호 <U>에뮬레이션</U> 출력 신호 |
| `7`       | `V1-START`   | 가상1P 스타트 버턴 스위치 <U>에뮬레이션</U> 출력 신호 |
| `8`       | `V2-START`   | 가상2P 스타트 버턴 스위치 <U>에뮬레이션</U> 출력 신호 |

-->

</td></tr>
</table>

- Pins are counted from the left.
- You can also input power directly into BillMock-HW through `12V` and `GND` pins.
- While it's more convenient to strip tde insulation from tde cable in tde middle and tden connect it to tde terminal,
- It's recommended, whenever possible, to use a cable type that comes pre-equipped for the connection.
<!--
- Pin# 왼쪽 부터 카운트함
- `12V`, `GND`로 BillMock-HW 자체의 전원을 입력받을 수 도 있습니다.
- 터미널 단자는 중간에 케이블을 끊어서 피복을 벋긴다음에 연결할 때에 용이하나,
- 가급적이면 케이블 타입을 구비하여 사용하는 것을 권장합니다.
-->

------------

## Host Side Player 1 Port (left)
<table>
<tr>
<td>

![J5](./images/pcb_0v4_port/J5.png)
</td>
<td>

|                |                |
| -------------- | -------------- |
| Designator     | J5  |
|                | Emulated Player 1 side connector towards the GAME IO side <!--가상 1P 코인기/지폐기/스타트 포트--> |
| Connector      | 141R-2.54-8P |

| **Pin #** | **Pin Name**   | Annotation                                             |
| :-------: | -------------- | ------------------------------------------------------ |
| `1`       | `V1-BUSY`      | Emulated Busy state Output Signal                      |
| `2`       | `V1-VEND`      | Emulated Bill acceptor 1P Coin Insertion Output Signal |
| `3`       | `V1-JAM`       | Emulated JAM Output Signal                             |
| `4`       | `V1-START`     | Emulated 1P Start Button Output Signal                 |
| `5`       | `V1-INHIBIT`   | Inhibit Input Signal from 1P GAME I/O                  |
| `6`       | N/C            | Not Connected                                          |
| `7`       | N/C            | Not Connected                                          |
| `8`       | `12V`          | Product + Pole Power Input                             |
| `9`       | `12V`          | Product + Pole Power Input                             |
| `10`      | `GND`          | Product - Pole Power Input                             |

<!--
| **Pin #** | **Pin Name** | Anotation |
| :-------: | -------------| --------- |
| `1`       | `V1-BUSY`    | 가상1P 코인기/지폐기 BUSY신호 <U>에뮬레이션</U> 출력  |
| `2`       | `V1-VEND`    | 가상1P 코인기/지폐기 진권신호 <U>에뮬레이션</U> 출력 신호 |
| `3`       | `V1-JAM`     | 가상1P 코인기/지폐기 종이걸림/고장 <U>에뮬레이션</U> 입력 신호 |
| `4`       | `V1-START`   | 가상1P 스타트 버튼쪽 스위치 <U>에뮬레이션</U> 출력 신호 |
| `5`       | `V1-INHIBIT` | 가상1P 코인기/지폐기 입수금지 <U>에뮬레이션</U> 입력 신호 |
| `6`       | N/C    |  |
| `7`       | N/C    |  |
| `8`       | `12V`  |  +극 전원 입력/출력, 제품 +극 전원 (12V 권장) |
| `9`       | `12V`  |  +극 전원 입력/출력, 제품 +극 전원 (12V 권장) |
| `10`      | `GND`  |  -극 전원 입력/출력. 제품 -극 전원 |

-->

</td></tr>
</table>

- Pins are counted from the left.
- You can also input power directly into BillMock-HW through `12V` and `GND` pins.
- The power pins from this port cannot be used for power output. When power input to this port is blocked, reverse voltage does not flow.
- The "Busy" output signal remains active low from the moment a payment signal is received from the credit card or when the VEND input signal goes active low until the VEND output signal toggles and completes.

<!--
- Pin# 왼쪽 부터 카운트함
- `12V`, `GND`로 BillMock-HW 자체의 전원을 입력받을 수 도 있습니다.
- 이 포트에서 나오는 전원 핀은 전원 출력용으로 사용 할 수 없습니다. 해당 포트로 들어오는 전원 입력이 차단 되는 경우, 역 전압이 흐르지 않습니다.
- Credit card의 payment 신호를 받거나 VEND  입력 신호가  Active Low로 들어온 시점부터 VEND 출력 신호가 Toggle 신호가 끝날 때 까지 Busy 출력신호를 Active Low로 출력합니다.
-->

------------

## Host Side Player 2 Port (right)
<table>
<tr>
<td>

![J4](./images/pcb_0v4_port/J4.png)
</td>
<td>

|                |                |
| -------------- | -------------- |
| Designator     | J4  |
|                | Emulated Player 2 side connector towards the GAME IO side <!--가상 2P 코인기/지폐기/스타트 포트--> |
| Connector      | 141R-2.54-8P |

| **Pin #** | **Pin Name**   | Annotation                                             |
| :-------: | -------------- | ------------------------------------------------------ |
| `1`       | `V1-BUSY`      | Emulated Busy state Output Signal                      |
| `2`       | `V1-VEND`      | Emulated Bill acceptor 2P Coin Insertion Output Signal |
| `3`       | `V1-JAM`       | Emulated JAM Output Signal                             |
| `4`       | `V1-START`     | Emulated 2P Start Button Output Signal                 |
| `5`       | `V1-INHIBIT`   | Inhibit Input Signal from 2P GAME I/O                  |
| `6`       | N/C            | Not Connected                                          |
| `7`       | N/C            | Not Connected                                          |
| `8`       | `12V`          | Product + Pole Power Input                             |
| `9`       | `12V`          | Product + Pole Power Input                             |
| `10`      | `GND`          | Product - Pole Power Input                             |

<!--
| **Pin #** | **Pin Name** | Anotation |
| :-------: | -------------| --------- |
| `1`       | `V1-BUSY`    | 가상2P 코인기/지폐기 BUSY신호 <U>에뮬레이션</U> 출력  |
| `2`       | `V1-VEND`    | 가상2P 코인기/지폐기 진권신호 <U>에뮬레이션</U> 출력 신호 |
| `3`       | `V1-JAM`     | 가상2P 코인기/지폐기 종이걸림/고장 <U>에뮬레이션</U> 입력 신호 |
| `4`       | `V1-START`   | 가상2P 스타트 버튼쪽 스위치 <U>에뮬레이션</U> 출력 신호 |
| `5`       | `V1-INHIBIT` | 가상2P 코인기/지폐기 입수금지 <U>에뮬레이션</U> 입력 신호 |
| `6`       | N/C    |  |
| `7`       | N/C    |  |
| `8`       | `12V`  |  +극 전원 입력/출력, 제품 +극 전원 (12V 권장) |
| `9`       | `12V`  |  +극 전원 입력/출력, 제품 +극 전원 (12V 권장) |
| `10`      | `GND`  |  -극 전원 입력/출력. 제품 -극 전원 |

-->

</td></tr>
</table>

- Pins are counted from the left.
- You can also input power directly into BillMock-HW through `12V` and `GND` pins.
- The power pins from this port cannot be used for power output. When power input to this port is blocked, reverse voltage does not flow.
- The "Busy" output signal remains active low from the moment a payment signal is received from the credit card or when the VEND input signal goes active low until the VEND output signal toggles and completes.

<!--
- Pin# 왼쪽 부터 카운트함
- `12V`, `GND`로 BillMock-HW 자체의 전원을 입력받을 수 도 있습니다.
- 이 포트에서 나오는 전원 핀은 전원 출력용으로 사용 할 수 없습니다. 해당 포트로 들어오는 전원 입력이 차단 되는 경우, 역 전압이 흐르지 않습니다.
- Credit card의 payment 신호를 받거나 VEND  입력 신호가  Active Low로 들어온 시점부터 VEND 출력 신호가 Toggle 신호가 끝날 때 까지 Busy 출력신호를 Active Low로 출력합니다.
-->
