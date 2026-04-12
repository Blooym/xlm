use eframe::egui::{
    Align, CentralPanel, Direction, Layout, Panel, Spinner, ViewportBuilder, ViewportCommand,
};
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};
use tracing::warn;

pub struct LaunchUI {
    state: Arc<UIState>,
    _handle: std::thread::JoinHandle<()>,
}

struct UIState {
    progress: RwLock<String>,
    should_close: AtomicBool,
}

impl LaunchUI {
    /// Creates a new instance of [`LaunchUI`] and immediately displays it to the user.
    pub fn new() -> Option<Self> {
        // Currently running the UI inside of snap breaks things,
        // for now just don't show it as a workaround.
        // https://github.com/Blooym/xlm/issues/19
        if std::env::var("SNAP").is_ok() {
            warn!(
                "Running inside snap environment - UI functionality has been disabled due to incompatibility."
            );
            return None;
        }

        let state = Arc::new(UIState {
            progress: RwLock::new(String::new()),
            should_close: AtomicBool::new(false),
        });
        let handle = std::thread::spawn({
            let state = state.clone();
            move || {
                eframe::run_ui_native(
                    "XLM",
                    eframe::NativeOptions {
                        event_loop_builder: Some(Box::new(|event_loop| {
                            use winit::platform::wayland::EventLoopBuilderExtWayland;
                            use winit::platform::x11::EventLoopBuilderExtX11;
                            EventLoopBuilderExtX11::with_any_thread(event_loop, true);
                            EventLoopBuilderExtWayland::with_any_thread(event_loop, true);
                        })),
                        run_and_return: true,
                        viewport: ViewportBuilder::default()
                            .with_inner_size([800.0, 500.0])
                            .with_resizable(false)
                            .with_decorations(false),
                        ..Default::default()
                    },
                    move |ctx, _frame| {
                        if state.should_close.load(Ordering::Relaxed) {
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                            return;
                        }
                        ctx.set_pixels_per_point(1.5);
                        Panel::bottom("bottom").show_inside(ctx, |ui| {
                            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                                ui.add(Spinner::default());
                                ui.label(
                                    state
                                        .progress
                                        .read()
                                        .expect("progress text lock should be readable")
                                        .as_str(),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("XLM v{}", env!("CARGO_PKG_VERSION")));
                                    });
                                });
                            });
                        });
                        CentralPanel::default().show_inside(ctx, |ui| {
                            ui.with_layout(
                                Layout::centered_and_justified(Direction::TopDown),
                                |ui| {
                                    ui.heading("Starting XIVLauncher\n(this may take a moment)");
                                },
                            );
                        });
                    },
                )
                .unwrap();
            }
        });

        Some(Self {
            state,
            _handle: handle,
        })
    }

    pub fn set_progress_text(&self, text: &str) {
        *self
            .state
            .progress
            .write()
            .expect("should be able to set ui progress text") = text.to_string();
    }
}

impl Drop for LaunchUI {
    fn drop(&mut self) {
        self.state.should_close.store(true, Ordering::Relaxed);
    }
}
