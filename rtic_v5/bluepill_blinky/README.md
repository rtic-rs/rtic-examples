# STM32F103 Bluepill RTIC Blink example

Working example of simple LED blinking application for popular Bluepill boards based on the STM32F103C8 chip. Example uses schedule API and peripherials access. You will need `stlink v2` tool or other programmer to flash the board.

## How-to

### Terminal workflow

Rust embedded relies heavily on `terminal workflow`, you will enter commands in the terminal. This can be strange at first, but this enables usage of great things like continious integration tools.

For Mac OS X consider using `iTerm2` instead of Terminal application.
For Windows consider using `powershell` (win + r -> powershell -> enter -> cd C:\examples\rtic_v5\bluepill_blinky)

### Build

Run `cargo build` to compile the code. If you run it for the first time, it will take some time to download and compile dependencies. After that, you will see comething like:

```bash
>cargo build
Finished dev [optimized + debuginfo] target(s) in 0.10s
```

If you see warnings, feel free to ask for help in chat or issues of this repo.

### Connect the board

You need to connect you bluepill board to ST-Link and connect pins:

| BOARD |    | ST-LINK |
|-------|----|---------|
| GND   | -> | GND     |
| 3.3V  | -> | 3.3V    |
| SWCLK | -> | SWCLK   |
| SWDIO | -> | SWDIO   |

Plug in ST-Link to USB port and wait it to initialize.

### Upload

We will use openocd to upload the code wit simple one-line one-command script:

```bash
openocd -f openocd.cfg -c "program target/thumbv7m-none-eabi/debug/app verify reset exit"
```

You will see something like:

```bash
openocd -f openocd.cfg -c "program target/thumbv7m-none-eabi/debug/app verify reset exit"
Open On-Chip Debugger 0.10.0
Licensed under GNU GPL v2
For bug reports, read
	http://openocd.org/doc/doxygen/bugs.html
Info : auto-selecting first available session transport "hla_swd". To override use 'transport select <transport>'.
Info : The selected transport took over low-level target control. The results might differ compared to plain JTAG/SWD
adapter speed: 1000 kHz
adapter_nsrst_delay: 100
none separate
Info : Unable to match requested speed 1000 kHz, using 950 kHz
Info : Unable to match requested speed 1000 kHz, using 950 kHz
Info : clock speed 950 kHz
Info : STLINK v2 JTAG v33 API v2 SWIM v7 VID 0x0483 PID 0x3748
Info : using stlink api v2
Info : Target voltage: 3.196863
Info : stm32f1x.cpu: hardware has 6 breakpoints, 4 watchpoints
target halted due to debug-request, current mode: Thread
xPSR: 0x01000000 pc: 0x080033b4 msp: 0x20005000
** Programming Started **
auto erase enabled
Info : device id = 0x20036410
Info : flash size = 64kbytes
target halted due to breakpoint, current mode: Thread
xPSR: 0x61000000 pc: 0x2000003a msp: 0x20005000
wrote 19456 bytes from file target/thumbv7m-none-eabi/debug/app in 1.118153s (16.992 KiB/s)
** Programming Finished **
** Verify Started **
target halted due to breakpoint, current mode: Thread
xPSR: 0x61000000 pc: 0x2000002e msp: 0x20005000
verified 18588 bytes in 0.288441s (62.933 KiB/s)
** Verified OK **
** Resetting Target **
shutdown command invoked
```

## Troubleshooting

If you are lucky and have new version of OpenOCD, you will need to change `openocd.cfg` file. Openocd will report error during the upload process, so you will just need to change line:

```txt
source [find interface/stlink-v2.cfg]
```

to

```txt
source [find interface/stlink.cfg]
```
