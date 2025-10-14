# CircuitDojo
A dojo for your circuits.

The goal is to be a simple, cost-effective, cross-platform, and FOSS alternative to the NI MyDAQ for my courses at Georgia Tech.
CircuitDOJO has four main components:
* Dojofile: this is a library and a tool for manipulating and using circuit pattern files. It just sends instructions to a CircuitDOJO and tests the results, basically. It can be used to
  validate a dojo file, compile it to bytecode, run it on a connected CircuitDojo board, and analyze the results in a few different formats (human-readable, JSON, CSV, binary dump).
* CircuitDojo Desktop: a simple cross-platform desktop gui for connecting to a CircuitDojo board, managing pins, and reading outputs, in a fairly nice way.
* DojoCore: the actual firmware that runs on your circuitdojo board. Right now it's just an arduino program.
* DojoLib: a library for connecting to and managing circuitdojo boards. it's also a command-line tool for managing CircuitDojo boards and sending commands.

You can in theory run Dojocore on any microcontroller capable of UART serial-over-USB at a reasonable baud rate (115200 is the default). Officially supported boards (which are tested with each version) are listed below:
* Arduino UNO R3

Because DojoCore is open-source and simple, porting it to any microcontroller you have on hand should not be a hard task. Just be aware of pin mapping.
DojoLib does not expect to work with more than 64 pins, and these MUST have stable indexes to real pins on the board. You *can* give your pins aliases in the board firmware.
The aliases on an arduino uno are pretty much what you would expect ("Digital 1", "Analog 4", etc).


## DojoLib
The easiest way to test a CircuitDojo board is the `dojolib` command. Just running it will prompt you for a valid serial port and open a simple utility prompt
that can send instructions to a board (all instructions wait for Acknowledge or Error):
* `setup`: send an establish byte.
* `getparams`: get board parameters. it'll just dump them out.
* `write <pin> <value>`: write a digital or analog pin. if digital, value should be HIGH or LOW. if analog, value should be a number between 0 and 1024.
* `setinput <pin> [pullup]`: set a pin to input mode, appending "pullup" to the end of the line if you want a pullup on the digital
* `setoutput <pin>`: set a pin to output mode.
* `sample`: run one sample


## The protocol

Because CircuitDojo needs to work at a reasonably high sample rate for weak microcontrollers, the protocol is very streamlined. The instructions to set a pin, or report a pin value, are very small.
For digital pins, every frame is just one byte!

All numbers are encoded in little-endian, unless otherwise specified (this is slightly faster on AVR devices)

Each frame starts with the one-byte header. The first bit of this is the "instruct" bit: if set, the next 7 bits will be interpreted as an operation code. If unset, the last six bits are a pin number.
If the pin number maps to a digital pin, the second bit is the digital pin's state; otherwise, the second bit is unset, and the value of the analog pin is included as the next 2 bytes.
This is exactly the same in both directions.

Valid operations from computer to board are as follows:
* 0xFF Please Establish: This is just the handshake start. The board will Acknowledge immediately.
* 0x80 Request Board Parameters: ask for a data-dump of the board's capabilities. This isn't part of the handshake because some use cases assume a fixed board (for instance, pattern files with a predefined mapping)
  response should be 0x80 [lowest sampling frequency] 0x81 [pin 1] ... 0x81 [pin n] ... 0x82 [name of the circuitdojo board]
* 0x81 Set Pin Mode Input: set the digital or analog pin represented by the lower 6 bits of the next byte to INPUT mode. If a pullup resistor is available on this pin (most digital pins on an Uno have one), the first
  bit of the next byte may be set to indicate INPUT_PULLUP mode.
* 0x82 Set Pin Mode Output: set the digital or analog pin represented by the lower 6 bits of the next byte to OUTPUT mode
* 0x83 Set Sampling Frequency: set the time-between-samples to the value represented by the next 2 bytes (in milliseconds). practically this should never be lower than 16 (60hz).
* 0x84 Begin Sampling: subscribe to pin value changes
* 0x85 End Sampling: unsubscribe
* 0x86 Run One Sample: if you don't want live sampling, just get one sample! this will send ALL pin values, not just changes. this is used by Dojofile mainly, but you can also use it in Desktop (by clicking "refresh pin values").

Valid operations from board to computer are as follows:
* 0xFF Acknowledge: it MUST send an acknowledge for every successful operation, including Please Establish.
* 0xFE Error: the last operation handled failed (this is guaranteed to be ordered and padded with Acknowledge).
* 0x80 Sampling Bounds: the next two bytes are the lower-bound for sampling frequency. This is not strictly enforced.
* 0x81 Pin Capabilities: the lower 6 bits of the next byte are the pin we're setting capabilities for. the MSB is analog (set) or digital (unset), and the second bit is pullup-available (set). the next bytes are
  the null-terminated "name" of this pin: these identifiers are not strictly standardized but are useful for Dojofile.
* 0x82 Board Description: the description of this board (just a string embedded in firmware: for instance "Arduino UNO R3 running CircuitDojo", null terminated)

Note that EVERY instruction (but no pin state assignments) should have either an Acknowledge or an Error before the response bytes (if they exist).
