use eframe::egui;
use std::sync::{Arc, Mutex, mpsc};

enum Message {
    Ports(Vec<String>),
}

struct CircuitDojoDesktop {
    board: Option<dojolib::BoardCapabilities>,
    ports: Vec<String>,
    rx: mpsc::Receiver<Message>,
    tx: mpsc::Sender<Message>,
    port_picker_selected: String,
}

impl CircuitDojoDesktop {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let context = cc.egui_ctx.clone();
        let (to_worker_tx, to_worker_rx) = mpsc::channel();
        let (to_gui_tx, to_gui_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let tx = to_gui_tx;
            let rx = to_worker_rx;
            let ports = dojolib::ports().unwrap();
            tx.send(Message::Ports(ports));
            context.request_repaint();
        });
        Self {
            board: None,
            ports: vec![],
            rx: to_gui_rx,
            tx: to_worker_tx,
            port_picker_selected: String::new(),
        }
    }
}

impl eframe::App for CircuitDojoDesktop {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(board) = self.board {
            todo!();
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ComboBox::from_label("Please select a serial port")
                    .selected_text(self.port_picker_selected)
                    .show_ui(ui, |ui| {
                        for port in self.ports.iter() {
                            ui.selectable_value(&mut self.port_picker_selected, port, port);
                        }
                    });
            });
        }
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "CircuitDojo Desktop",
        native_options,
        Box::new(|cc| Ok(Box::new(CircuitDojoDesktop::new(cc)))),
    );
}
