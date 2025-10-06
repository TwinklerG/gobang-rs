use std::sync::{
    Arc, Mutex,
    mpsc::{self, Receiver},
};

use eframe::egui::{self, Pos2};
use egui::{Align2, Color32, FontId};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use crate::ai::{AI, GameState};

const BOARD_SIZE: usize = 15;
const GRID_SIZE: usize = 40;
const BOARD_E_SIZE: f32 = 640.0;

#[derive(PartialEq, Eq)]
enum AppState {
    Idle,
    Gaming,
    AIThinking,
    Settlement,
}

pub struct GobangApp {
    board: [[i32; BOARD_SIZE]; BOARD_SIZE],
    ai: Arc<Mutex<AI>>,
    state: AppState,
    last_step: Option<(usize, usize)>,
    rx: Option<Receiver<(usize, usize)>>,

    // Config
    role: &'static str,
    role_black: bool,
    depth: usize,
}
impl GobangApp {
    pub fn new() -> Self {
        Self {
            board: [[0; BOARD_SIZE]; BOARD_SIZE],
            ai: Arc::new(Mutex::new(AI::new())),
            state: AppState::Idle,
            last_step: None,
            rx: None,

            role: "BLACK",
            role_black: true,
            depth: 2,
        }
    }
}
impl eframe::App for GobangApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state == AppState::Idle {
                ui.label("You");
                if ui.button(self.role).clicked() {
                    self.role = if self.role == "BLACK" {
                        "WHITE"
                    } else {
                        "BLACK"
                    }
                }
                ui.add(egui::Slider::new(&mut self.depth, 1..=2).text("recurse depth"));
                if ui.button("Start Game").clicked() {
                    self.state = AppState::Gaming;
                    self.ai.lock().unwrap().depth = self.depth;
                    if self.role == "BLACK" {
                        self.role_black = true;
                    } else {
                        self.role_black = false;
                        self.board[7][7] = 1;
                        self.ai.lock().unwrap().ai_step(7, 7);
                    }
                }
                return;
            }
            let (_, response) = ui.allocate_at_least(
                egui::Vec2::new(BOARD_E_SIZE, BOARD_E_SIZE),
                egui::Sense::click(),
            );
            let painter = ui.painter();
            painter.rect_filled(ui.clip_rect(), 0.0, Color32::from_rgb(239, 228, 176));
            for i in 0..BOARD_SIZE {
                let start = Pos2::new(0.0, 0.0)
                    + egui::Vec2::new((GRID_SIZE + GRID_SIZE * i) as f32, GRID_SIZE as f32);
                let end = Pos2::new(0.0, 0.0)
                    + egui::Vec2::new(
                        (GRID_SIZE + GRID_SIZE * i) as f32,
                        BOARD_E_SIZE - GRID_SIZE as f32,
                    );
                painter.line_segment([start, end], egui::Stroke::new(1.0, egui::Color32::BLACK));
            }
            for i in 0..BOARD_SIZE {
                let start = Pos2::new(0.0, 0.0)
                    + egui::Vec2::new(GRID_SIZE as f32, (GRID_SIZE + GRID_SIZE * i) as f32);
                let end = Pos2::new(0.0, 0.0)
                    + egui::Vec2::new(
                        BOARD_E_SIZE - GRID_SIZE as f32,
                        (GRID_SIZE + GRID_SIZE * i) as f32,
                    );
                painter.line_segment([start, end], egui::Stroke::new(1.0, egui::Color32::BLACK));
            }
            if response.clicked() {
                if self.state == AppState::Gaming {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let (y, x) = (pos.x as usize, pos.y as usize);
                        if (x % GRID_SIZE > GRID_SIZE / 4 && x % GRID_SIZE < GRID_SIZE / 4 * 3)
                            || (y % GRID_SIZE > GRID_SIZE / 4 && y % GRID_SIZE < GRID_SIZE / 4 * 3)
                        {
                            return;
                        }
                        let (x, y) = (
                            (x - GRID_SIZE / 2) / GRID_SIZE,
                            (y - GRID_SIZE / 2) / GRID_SIZE,
                        );
                        if self.board[x][y] == 0 {
                            self.board[x][y] = if self.role_black { 1 } else { 2 };
                            self.last_step = Some((x, y));
                            self.ai.lock().unwrap().human_step(x, y);
                            if !self.ai.lock().unwrap().is_game_over() {
                                let ai = self.ai.clone();
                                let (tx, rx) = mpsc::channel();
                                self.rx = Some(rx);
                                self.state = AppState::AIThinking;
                                #[cfg(not(target_arch = "wasm32"))]
                                tokio::task::spawn(async move {
                                    let (nx, ny) = ai.lock().unwrap().ai();
                                    tx.send((nx, ny)).expect("Can not send data");
                                });
                                #[cfg(target_arch = "wasm32")]
                                spawn_local(async move {
                                    let (nx, ny) = ai.lock().unwrap().ai();
                                    tx.send((nx, ny)).expect("Can not send data");
                                });
                            } else {
                                self.state = AppState::Settlement;
                            }
                        }
                    }
                } else if self.state == AppState::Settlement
                    && response.interact_pointer_pos().is_some()
                {
                    self.state = AppState::Idle;
                    *self = GobangApp::new();
                }
            }
            if let Some(rx) = &self.rx {
                if let Ok((nx, ny)) = rx.try_recv() {
                    self.board[nx][ny] = if self.role_black { 2 } else { 1 };
                    self.state = AppState::Gaming;
                    self.last_step = Some((nx, ny));
                    if self.ai.lock().unwrap().is_game_over() {
                        self.state = AppState::Settlement;
                    }
                }
            }
            for x in 0..BOARD_SIZE {
                for y in 0..BOARD_SIZE {
                    let center = Pos2::new(0.0, 0.0)
                        + egui::Vec2::new(
                            (GRID_SIZE * (y + 1)) as f32,
                            (GRID_SIZE * (x + 1)) as f32,
                        );
                    if self.board[x][y] == 1 {
                        let fill_color = egui::Color32::BLACK;
                        painter.circle_filled(center, (GRID_SIZE / 3) as f32, fill_color);
                    } else if self.board[x][y] == 2 {
                        let fill_color = egui::Color32::WHITE;
                        painter.circle_filled(center, (GRID_SIZE / 3) as f32, fill_color);
                        painter.circle_stroke(
                            center,
                            (GRID_SIZE / 3) as f32,
                            egui::Stroke::new(1.0, egui::Color32::BLACK),
                        );
                    }
                    if let Some((cx, cy)) = self.last_step {
                        if cx == x && cy == y {
                            painter.circle_stroke(
                                center,
                                (GRID_SIZE / 3) as f32,
                                egui::Stroke::new(2.0, egui::Color32::BLUE),
                            );
                        }
                    }
                }
            }
            if self.state == AppState::AIThinking {
                painter.rect_filled(
                    ui.clip_rect(),
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150),
                );
                ui.painter().text(
                    Pos2::new(BOARD_E_SIZE / 2.0, BOARD_E_SIZE / 2.0),
                    Align2::CENTER_CENTER,
                    "AI Thinking",
                    FontId::proportional(32.0),
                    Color32::RED,
                );
            } else if self.state == AppState::Settlement {
                let text = match self.ai.lock().unwrap().state {
                    GameState::Human => Some(format!("HUMAN WINS DEPTH {}", self.depth)),
                    GameState::AI => Some(format!("AI WINS DEPTH {}", self.depth)),
                    _ => None,
                };
                if let Some(text) = text {
                    painter.rect_filled(
                        ui.clip_rect(),
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150),
                    );
                    ui.painter().text(
                        Pos2::new(BOARD_E_SIZE / 2.0, BOARD_E_SIZE / 2.0),
                        Align2::CENTER_CENTER,
                        text,
                        FontId::proportional(32.0),
                        Color32::RED,
                    );
                    ui.painter().text(
                        Pos2::new(BOARD_E_SIZE / 2.0, BOARD_E_SIZE / 4.0 * 3.0),
                        Align2::CENTER_CENTER,
                        "Click anywhere to continue",
                        FontId::proportional(32.0),
                        Color32::RED,
                    );
                }
            }
        });
    }
}
