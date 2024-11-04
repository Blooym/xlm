use eframe::egui::{
    Align, CentralPanel, Direction, Layout, Spinner, TopBottomPanel, ViewportBuilder,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};
use winit::platform::wayland::EventLoopBuilderExtWayland;

#[derive(Default)]
pub struct LaunchUI {
    /// Whether all egui windows should close next frame.
    should_close: Arc<AtomicBool>,
    /// The progress text to show while XLM is performing a setup.
    pub progress_text: Arc<RwLock<&'static str>>,
}

impl LaunchUI {
    /// Starts a new Tokio task and displays an egui window displaying a "XIVLauncher is starting" message.
    /// The egui window blocks inside of the task meaning it cannot be killed by aborting the thread.
    /// To close the window you can call [`LaunchUI::kill`] which will close all existing windows.
    pub fn spawn_background(&self) {
        let close_copy = self.should_close.clone();
        let progress_text_copy = self.progress_text.clone();
        tokio::task::spawn(async move {
            eframe::run_simple_native(
                "XLM",
                eframe::NativeOptions {
                    event_loop_builder: Some(Box::new(|event_loop_builder| {
                        event_loop_builder.with_any_thread(true);
                    })),
                    viewport: ViewportBuilder::default()
                        .with_inner_size([800.0, 500.0])
                        .with_resizable(false)
                        .with_decorations(false),
                    ..Default::default()
                },
                move |ctx, _frame| {
                    if close_copy.load(Ordering::Relaxed) {
                        std::process::exit(0);
                    }

                    ctx.set_pixels_per_point(1.5);
                    TopBottomPanel::bottom("bottom").show(ctx, |ui| {
                        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                            ui.add(Spinner::default());
                            ui.label(*progress_text_copy.read().unwrap());
                            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(format!("XLM v{}", env!("CARGO_PKG_VERSION")));
                                });
                            });
                        });
                    });
                    CentralPanel::default().show(ctx, |ui| {
                        ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                            ui.heading("Starting XIVLauncher\n(this may take several minutes)");
                        });
                    });
                },
            )
            .unwrap();
        });
    }

    /// Closes any running egui windows regardless of the thread they're running on.
    pub fn kill(self) {
        self.should_close.store(true, Ordering::Relaxed);
    }
}
