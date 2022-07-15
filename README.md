# STM32+Rust context switching

![output](res/output.gif)

The following is an (almost) dependency-free embedded application implementing context switching (threading) technique.

## Functionality

It performs no useful work apart from serving the purpose of being a challenging exercise.
Put briefly, it initializes required peripherals (timer, and UART), allocates a static storage for a task's stack, registers the task in a context queue, and performs context switching in Pend SV interrupt, when it is triggered by the timer's ISR.
The created task continuously spams into UART configured to operate at baudrate 57600.

## Hardware

- [STM32F030 Demo board](https://stm32-base.org/boards/STM32F030F4P6-STM32F030-DEMO-BOARD-V1.1.html#Header-1);
- UART / USB adapter (optional);
- SWD debugger (optional);

### Wiring

- `PA2` - UART RX;
- `PA3` - UART TX;
- `3v3` - 20 pin JTAG connector's `pin 1` (VTREF);
- `GND` - 20 pin JTAG connector's `pin 18` (GND);
- `PA14` - 20 pin JTAG connector's `pin 9` (TCK);
- `PA13` - 20 ping JTAG connector's `pin 7` (TMS);

*Important: do not power the demo board from USB and SWD's VTREF simultaneously.*

## Implementation details

- No functionality-related third party code (like HAL) was used. Working w/ peripherals has been done "manually";
- To spare efforts on typing special register offsets, I wrote a [naive parser](https://github.com/damurashov/STM32-CubeMX-registers-to-Rust) translating CubeMX-generated `C` code (CubeMX 6.4.0) into that of `Rust` (you can see the output it produces in `src/reg.rs`). Cannot vouch for it to be the one-stop solution, but it works in my case (that would be STM32F030F4 + CubeMX 6.4.0);
- The implementation does not use dynamic allocation, primarily because using `malloc` creates an additional memory footprint. Although this option is provided by the project.
- Code location hints:
	- Examples of using dynamic memory management functions from arm-none-eabi toolchain libraries can be found in `mem.rs` and `src/thread/task.rs`;
	- For entry points, see `init.rs` and `main.rs`;
	- Peripherals-related code is placed in `periph::` module;
	- Synchronization and task management-related code resides in `thread::` module;
	- The most semantically important part of the project is `src/thread/task.rs`, and `src/thread/task.s`. Those two handle task creation, scheduling, and context switching.
