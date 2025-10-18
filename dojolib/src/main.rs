use dojolib::connection::*;
use dojolib::*;
use std::io::Write;

fn readline() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn connect(port: &str) {
    println!("Connecting to port {} @115200 baud", port);
    let mut connection = Connection::new(port, 115200).unwrap();
    connection.begin().unwrap();
    let capabilities = connection.request_capabilities().unwrap();
    println!("== Connected to {} ==", capabilities.name);
    println!("Available pins:");
    for pin in capabilities.pins.iter() {
        println!(
            "* [pin {}] {}: {}",
            pin.id,
            match pin.pin_type {
                PinType::Analog => "ANALOG",
                PinType::Digital => "DIGITAL NO PULLUP",
                PinType::DigitalPullup => "DIGITAL PULLUP AVAILABLE",
            },
            pin.identifier
        );
    }
    let mut modes = vec![0; capabilities.pins.len()];
    loop {
        let line = readline();
        let mut args = line.split(' ');
        let command = if let Some(arg) = args.next() {
            arg
        } else {
            continue;
        };
        match command {
            "setoutput" => {
                let pin: u8 = args.next().unwrap().parse().unwrap();
                connection.set_output(pin).unwrap();
                modes[pin as usize] = 2;
            }
            "setinput" => {
                let pin: u8 = args.next().unwrap().parse().unwrap();
                connection.set_input(pin).unwrap();
                modes[pin as usize] = 1;
            }
            "write" => {
                let pin: u8 = args.next().unwrap().parse().unwrap();
                let value = args.next().unwrap() == "HIGH";
                connection.digital_write(pin, value).unwrap();
            }
            "sample" => {
                let mut inputs_count = 0;
                for mode in modes.iter() {
                    if *mode == 1 {
                        inputs_count += 1;
                    }
                }
                for (i, state) in connection.sample(inputs_count).unwrap().into_iter() {
                    println!("* [pin {}] is {}", i, if state { "HIGH" } else { "LOW" });
                }
            }
            _ => {
                println!("invalid command");
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
