# CircuitDojo
A dojo for your circuits.

This is BSD-licensed DAQ software. It is designed to replicate the essential functionality of products like NI's MyDaq, but much cheaper and simpler.
The technical limitations make it ill-suited for enterprise use; the main use case is education, especially in introductory classes.

The FOSS nature allows CircuitDojo to overcome several particularly important limitations imposed by NI:
* The MyDaq is comparatively extremely expensive. You can run CircuitDojo on an Arduino UNO, which usually costs around $10 (or more like $30 if you buy official).
  A new MyDaq costs a minimum of $250.
* The MyDaq only works on Windows. You can't run it on Mac or on any Linux. CircuitDojo, on the other hand, can be compiled and used on any operating system!
* It is hard to extend the MyDaq's feature set. You have to buy a different product. Because CircuitDojo runs on Arduino boards, they can be reprogrammed at will;
  the software is fully open source, so any part or the whole of it can be modified or rewritten without much trouble.
* The MyDaq software is temperamental. Installing the software occasionally bricks Windows laptops! I actually witnessed this in person in one of my classes. CircuitDojo is packaged
  as a rootless executable: you don't need to install it, and you don't need superuser privileges to use it, so it can't accidentally damage your computer.

This is not to say CircuitDojo is perfect. There are many reasons *not* to use it:
* It has ridiculous latency. The update rate is standardized at 60hz. It cannot be used for oscilloscope applications or anything that needs fast or precise observations.
* It is less self-contained. You're pretty much stuck with wires sticking out of a pcb.
* It is restricted to real time digital. Analog and PWM support is high on the list, but it's not there *yet*. If you need anything oscillating or anything variable,
  CircuitDojo won't work.
* It has a limited voltage range. You have to use 5 volt logic; higher will damage the board, and lower will not reliably measure. The MyDaq can go much higher than this and is
  more tolerant.

CircuitDojo is not meant to be a particularly powerful DAQ. It is meant to be an extremely cheap DAQ sufficient for simple 5v digital logic manipulation.
If all you need is low-frequency digital logic manipulation at 5 volts, CircuitDojo is quite sufficient!

## Building the Board
Presently, all you need is an Arduino UNO and an appropriate cable to connect to it.

Follow these steps:
1. Install the Arduino IDE from [Arduino's official website](https://www.arduino.cc/en/software/). On Linux this will often be available from your distro's repositories or as a FlatPak.
2. Clone this repository - you can just click "Code" => "Download ZIP" to get a compressed archive version, which works fine.
   (If you know how to use git already, disregard the zip thing).
3. Open dojocore/dojocore.ino IN the Arduino IDE. Do not change anything.
4. Connect your Arduino UNO to your computer.
  * On Linux, you should see a new device called `/dev/ttyACM0` or similar show up immediately - if it doesn't, check your cable and board.
  * On Windows, you may need to install Arduino drivers - you can read the tutorial [here](https://docs.arduino.cc/tutorials/generic/DriverInstallation/) for help with that.
5. Make sure your Arduino UNO is selected in the IDE's board selector. It should be by default.
6. Click "Upload" (be careful not to "Verify", as this will not actually install dojocore) and wait until you get a notification saying "Done Uploading".
7. Your CircuitDojo board is now ready for use! You can close the Arduino IDE.

If you encounter issues, submit a bug report on this repository, or (if you're a Gatech student) email me at my student address.

Note: eventually I intend to sell open-source-hardware ATMEGA328P-based boards with CircuitDojo preinstalled (for considerably less than an official Uno,
and probably slightly less than a ripoff). If you're a Gatech student and want to work on PCBs with me, shoot me an email!

## Using the Software
Right now there are no binary releases. Hopefully that will change soon.

If you can compile on Mac and send me a dmg, that would be pretty great! I don't have a Mac to compile on.

To use the software, you'll need to set up a Rust toolchain (see [rustup.rs](https://rustup.rs/)) and
`cargo build` the Rust project in `desktop/`. This will leave an appropriate executable at `target/build/desktop`.

You can also just `cargo run` from the `desktop/` directory to launch the application.

Once you've compiled, you don't need to install or even get superuser privileges.

It will prompt you for a serial port: select the one with your Arduino UNO attached, and click "start". The application will hang for a second or two depending on your board
(it's establishing a serial connection), and then switch over to a screen with all of your board's digital pins visible! Click the boxes saying "output", "input", or "unset" to
toggle between them, and for outputs, click anywhere else in the block to toggle HIGH/LOW.

### The Port Picker
![Screenshot of the port picker](/port_picker.png)

### The Main Screen
I tethered pin 7 and pin 5 on my board for this one.
![Screenshot of the main screen](/pins_screenshot.png)

## Where Can I Use This?
You can use it anywhere *personally*, of course. It is not officially supported by any courses at any universities.

If you're a professor and you've allowed CircuitDojos in your course, it'd be pretty cool if you were to shoot me an email so I can update this!

## Bugs
Bugs happen. They should be submitted in this repository; anyone at Gatech can also send me an email to my student account.
If the software crashes, it will print out some diagnostic information: if you can capture this, please include the text in your bug report.

## Contributing
While anyone can make changes to the source code on their own, I will only accept changes submitted through the normal process (e.g. fork, make changes, pull request).

There is as yet no standardized style; just try to make it perform well. No changes that break ATMEGA328P support or cross-platform support will be accepted.

Some low-hanging fruit:
* Finish implementing analog, digital pullup, and PWM support
* Add "board layout files" that set the pin modes and their labels and locks changes
* Clean up dojolib's hacky I/O thread
* Fix the connection-setup freeze
* Test (and make changes to support) other boards (especially the Mega)

## BSD License
Copyright 2025 Tyler Clarke

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
