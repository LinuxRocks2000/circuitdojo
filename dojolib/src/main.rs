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
