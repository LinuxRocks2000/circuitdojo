/*
Copyright 2025 Tyler Clarke

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

*/
use dojolib::{ADC_CONSTANT, Board, board::PinStatus};
use eframe::egui::{self, Color32, Stroke};

trait Screen {
    fn draw(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) -> Option<Box<dyn Screen>>; // draw function that meshes nicely with egui
    // and optionally passes control to a different Screen
}

struct PortPickerScreen {
    ports_list: Vec<String>,
    selected: usize,
}

impl PortPickerScreen {
    fn new() -> Self {
        Self {
            ports_list: dojolib::ports().unwrap(),
            selected: 0,
        }
    }
}

impl Screen for PortPickerScreen {
    fn draw(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<Box<dyn Screen>> {
        let mut rtval: Option<Box<dyn Screen>> = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ComboBox::from_label("Select A Port")
                .selected_text(self.ports_list[self.selected].clone())
                .show_ui(ui, |ui| {
                    for (i, port) in self.ports_list.iter().enumerate() {
                        ui.selectable_value(&mut self.selected, i, port);
                    }
                });
            if ui.button("Start").clicked() {
                rtval = Some(Box::new(MainScreen::new(&self.ports_list[self.selected])));
            }
        });
        rtval
    }
}

struct MainScreen {
    board: Board,
    selected_pin: u8,
}

impl MainScreen {
    fn new(port: impl AsRef<str>) -> Self {
        let mut board = Board::new(port, 115200).unwrap();
        board.subscribe(16).unwrap(); // 16ms sample rate = 60hz
        Self {
            board,
            selected_pin: 0,
        }
    }

    fn digital_ins(&mut self, ui: &mut egui::Ui) {
        for pin in self.board.pins() {
            if let PinStatus::DigitalInputting(value) = pin.status() {
                egui::Frame::new()
                    .inner_margin(5.0)
                    .stroke(Stroke::new(1.0, Color32::BLACK))
                    .outer_margin(3.0)
                    .fill(if value {
                        Color32::DARK_GREEN
                    } else {
                        Color32::DARK_RED
                    })
                    .show(ui, |ui| {
                        ui.label(format!(
                            "[{}] {}: {}",
                            pin.hw_id(),
                            pin.ident(),
                            if value { "HIGH" } else { "LOW" }
                        ))
                    });
            }
        }
    }

    fn digital_outs(&mut self, ui: &mut egui::Ui) {
        for mut pin in self.board.pins() {
            if let PinStatus::DigitalOutputting(value) = pin.status() {
                if egui::Frame::new()
                    .inner_margin(5.0)
                    .stroke(Stroke::new(1.0, Color32::BLACK))
                    .outer_margin(3.0)
                    .fill(if value {
                        Color32::DARK_GREEN
                    } else {
                        Color32::DARK_RED
                    })
                    .show(ui, |ui| {
                        if ui
                            .label(format!(
                                "[{}] {}: {}",
                                pin.hw_id(),
                                pin.ident(),
                                if value { "HIGH" } else { "LOW" }
                            ))
                            .clicked()
                        {
                            pin.digital_write(!value).unwrap();
                        }
                    })
                    .response
                    .clicked()
                {
                    pin.digital_write(!value).unwrap();
                }
            }
        }
    }

    fn analog_ins(&mut self, ui: &mut egui::Ui) {
        for pin in self.board.pins() {
            if let PinStatus::AnalogInputting(value) = pin.status() {
                egui::Frame::new()
                    .inner_margin(5.0)
                    .stroke(Stroke::new(1.0, Color32::BLACK))
                    .outer_margin(3.0)
                    .fill(Color32::DARK_BLUE)
                    .show(ui, |ui| {
                        ui.label(format!(
                            "[{}] {}: {:1.2}V",
                            pin.hw_id(),
                            pin.ident(),
                            value as f32 * ADC_CONSTANT
                        ))
                    });
            }
        }
    }

    fn analog_outs(&mut self, ui: &mut egui::Ui) {
        for mut pin in self.board.pins() {
            if let PinStatus::AnalogOutputting(value) = pin.status() {
                egui::Frame::new()
                    .inner_margin(5.0)
                    .stroke(Stroke::new(1.0, Color32::BLACK))
                    .outer_margin(3.0)
                    .fill(Color32::from_rgb(150, 50, 80))
                    .show(ui, |ui| {
                        let mut slider_value = value as f32 * 100.0 / 255.0;
                        let old_slider = slider_value;
                        ui.label(format!(
                            "[{}] {}: {:2.2}%",
                            pin.hw_id(),
                            pin.ident(),
                            slider_value
                        ));
                        ui.add(egui::Slider::new(&mut slider_value, 0.0..=100.0));
                        if slider_value != old_slider {
                            pin.analog_write((slider_value * 255.0 / 100.0) as u8)
                                .unwrap();
                        }
                    });
            }
        }
    }
}

impl Screen for MainScreen {
    fn draw(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<Box<dyn Screen>> {
        self.board.update().unwrap();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Pin Select");
            ui.columns(3, |cols| {
                let [pin_select, digital, analog] = cols else {
                    unreachable!()
                };
                pin_select.vertical(|ui| {
                    let mut selected_pin = self.board.get_pin(self.selected_pin).unwrap();
                    {
                        egui::ComboBox::from_id_salt("PinSelector")
                            .selected_text(selected_pin.ident())
                            .show_ui(ui, |menu| {
                                for (i, pin) in self.board.pins().enumerate() {
                                    menu.selectable_value(
                                        &mut self.selected_pin,
                                        i as u8,
                                        pin.ident(),
                                    );
                                }
                            });
                    }
                    if ui.button("Set Input").clicked() {
                        selected_pin.set_input().unwrap();
                    }
                    if ui.button("Set Output").clicked() {
                        selected_pin.set_output().unwrap();
                    }
                    if ui.button("Set PWM Out").clicked() {
                        selected_pin.analog_write(0).unwrap();
                    }
                    if ui.button("Disable").clicked() {
                        selected_pin.disable().unwrap();
                    }
                    ui.label(match selected_pin.status() {
                        PinStatus::AnalogInputting(val) => {
                            format!("Analog Input {:1.2}V", val as f32 * ADC_CONSTANT)
                        }
                        PinStatus::AnalogOutputting(val) => {
                            format!("PWM Output {:2.2}%", val as f32 / 255.0 * 100.0)
                        }
                        PinStatus::DigitalInputting(val) => {
                            format!("Digital Input {}", if val { "HIGH" } else { "LOW" })
                        }
                        PinStatus::DigitalOutputting(out) => {
                            format!("Digital Output {}", if out { "HIGH" } else { "LOW" })
                        }
                        PinStatus::DigitalPullupInputting(_) => unreachable!(),
                        PinStatus::NoStatus => format!("Unused"),
                    });
                });
                egui::ScrollArea::vertical()
                    .id_salt("Digital")
                    .show(digital, |ui| {
                        ui.label("Digital Inputs");
                        self.digital_ins(ui);
                        ui.separator();
                        ui.label("Digital Outputs");
                        self.digital_outs(ui);
                    });
                egui::ScrollArea::vertical()
                    .id_salt("Analog")
                    .show(analog, |ui| {
                        ui.label("Analog Inputs");
                        self.analog_ins(ui);
                        ui.separator();
                        ui.label("PWM Outputs");
                        self.analog_outs(ui);
                    });
            });
        });
        ctx.request_repaint();
        None
    }
}

struct CircuitDojoDesktop {
    screen: Box<dyn Screen>,
}

impl CircuitDojoDesktop {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            screen: Box::new(PortPickerScreen::new()),
        }
    }
}

impl eframe::App for CircuitDojoDesktop {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(new_screen) = self.screen.draw(ctx, frame) {
            self.screen = new_screen;
        }
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "CircuitDojo Desktop",
        native_options,
        Box::new(|cc| Ok(Box::new(CircuitDojoDesktop::new(cc)))),
    )
    .unwrap();
}
