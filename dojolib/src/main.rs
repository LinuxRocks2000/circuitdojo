/*
Copyright 2025 Tyler Clarke

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

*/
use dojolib::board::*;
use dojolib::*;
use std::io::Write;

fn readline() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn connect(port: &str) {
    println!("Connecting to port {} @115200 baud", port);
    let mut board = Board::new(port, 115200).unwrap();
    board.subscribe(100).unwrap();
    println!("Connected to {}", board.get_name());
    loop {
        let line = readline();
        board.update().unwrap();
        let mut args = line.split(" ");
        match args.next().unwrap().as_ref() {
            "pins" => {
                for pin in board.pins() {
                    println!(
                        "* [{}] {} {}: {}",
                        pin.hw_id,
                        match pin.tp {
                            PinType::DigitalPullup => "DP WP",
                            PinType::Digital => "DP",
                            PinType::Analog => "AP",
                        },
                        pin.ident,
                        match pin.status {
                            PinStatus::DigitalInputting(level) => {
                                format!("Inputting {}", if level { "HIGH" } else { "LOW" })
                            }
                            PinStatus::DigitalOutputting(level) => {
                                format!("Outputting {}", if level { "HIGH" } else { "LOW" })
                            }
                            _ => String::new(),
                        }
                    );
                }
            }
            "setoutput" => {
                let pin_num = args.next().unwrap().parse::<u8>().unwrap();
                board.set_output(pin_num).unwrap();
            }
            "setinput" => {
                let pin_num = args.next().unwrap().parse::<u8>().unwrap();
                board.set_input(pin_num).unwrap();
            }
            "digitalwrite" => {
                let pin_num = args.next().unwrap().parse::<u8>().unwrap();
                let value = match args.next().unwrap() {
                    "HIGH" => true,
                    "LOW" => false,
                    _ => panic!(),
                };
                board.digital_write(pin_num, value).unwrap();
            }
            _ => {
                println!("bad command");
            }
        }
    }
}

fn main() {
    println!("DojoLib v{DOJOLIB_VERSION} by Tyler Clarke");
    let options = ports().unwrap();
    if options.len() == 0 {
        println!("No ports found. Abort.");
    } else if options.len() == 1 {
        connect(&options[0]);
    } else {
        println!("Please choose a serial port:");
        for (id, port) in options.iter().enumerate() {
            println!("{id}. {port}");
        }
        print!("> ");
        std::io::stdout().flush().unwrap();
        loop {
            let data = readline();
            match data.parse::<usize>() {
                Ok(dat) => {
                    if let Some(port) = options.get(dat) {
                        connect(port);
                        break;
                    }
                }
                Err(_) => {}
            }
        }
    }
}
