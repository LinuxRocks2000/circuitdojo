use dojolib::{
    Board,
    board::{PinMode, PinStatus},
};
use eframe::egui::{self, Align2, Color32, FontId, Rect, Rgba, Sense, Stroke, StrokeKind};

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
    fn draw(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) -> Option<Box<dyn Screen>> {
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
}

impl MainScreen {
    fn new(port: impl AsRef<str>) -> Self {
        let mut board = Board::new(port, 115200).unwrap();
        board.subscribe(100).unwrap();
        Self { board }
    }
}

impl Screen for MainScreen {
    fn draw(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) -> Option<Box<dyn Screen>> {
        self.board.update().unwrap();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                let mut mode_op = None;
                let mut out_op = None;
                for pin in self.board.pins() {
                    let (res, paint) = ui.allocate_painter(egui::Vec2::splat(64.0), Sense::click());
                    paint.rect_filled(
                        res.rect,
                        0.0,
                        match pin.mode {
                            PinMode::Unset => Rgba::from_rgb(0.7, 0.7, 0.7),
                            _ => {
                                if match pin.status {
                                    PinStatus::DigitalOutputting(val) => val,
                                    PinStatus::DigitalInputting(val) => val,
                                    _ => false,
                                } {
                                    Rgba::from_rgb(0.0, 1.0, 0.0)
                                } else {
                                    Rgba::from_rgb(1.0, 0.0, 0.0)
                                }
                            }
                        },
                    );
                    paint.text(
                        res.rect.center_top(),
                        Align2::CENTER_TOP,
                        &pin.ident,
                        FontId::monospace(8.0),
                        Color32::BLACK,
                    );
                    let mode_rect = res.rect.split_top_bottom_at_fraction(0.7).1.shrink(1.0);
                    paint.rect_stroke(
                        mode_rect,
                        0.0,
                        Stroke::new(2.0, Rgba::BLACK),
                        StrokeKind::Inside,
                    );
                    paint.text(
                        mode_rect.center(),
                        Align2::CENTER_CENTER,
                        match pin.mode {
                            PinMode::Input => "INPUT",
                            PinMode::Output => "OUTPUT",
                            PinMode::Unset => "OFF",
                        },
                        FontId::monospace(10.0),
                        Color32::BLACK,
                    );
                    if res.clicked() {
                        if let Some(pointer) = res.interact_pointer_pos() {
                            if mode_rect.contains(pointer) {
                                mode_op = Some((pin.mode, pin.hw_id));
                            } else {
                                if let PinMode::Output = pin.mode {
                                    if let PinStatus::DigitalOutputting(val) = pin.status {
                                        out_op = Some((!val, pin.hw_id));
                                    } else {
                                        out_op = Some((true, pin.hw_id));
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some((mode, id)) = mode_op {
                    match mode {
                        PinMode::Input => {
                            self.board.set_output(id).unwrap();
                        }
                        PinMode::Output | PinMode::Unset => {
                            self.board.set_input(id).unwrap();
                        }
                    }
                }
                if let Some((status, id)) = out_op {
                    self.board.digital_write(id, status).unwrap();
                }
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
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
