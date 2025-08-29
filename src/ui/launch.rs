use eframe::egui::{
    Align, CentralPanel, Direction, Layout, Spinner, TopBottomPanel, ViewportBuilder,
};
use log::warn;
use std::{
    io::{self, BufRead, Write},
    sync::{Arc, RwLock, mpsc},
};

pub struct LaunchUI {
    child: std::process::Child,
    tx: mpsc::Sender<String>,
    _stdin_thread: Option<std::thread::JoinHandle<()>>,
}

impl LaunchUI {
    /// Creates a new instance of [`LaunchUI`] and immediately displays it to the user.
    ///
    /// This method will run a new instance of the current executable to display the UI.
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

        let (tx, rx) = mpsc::channel();

        let mut child = std::process::Command::new(std::env::current_exe().unwrap());
        #[cfg(all(not(debug_assertions), feature = "self_update"))]
        child.arg("--xlm-updater-disable");
        child
            .arg("internal-launch-ui")
            .stdin(std::process::Stdio::piped());
        let mut child = child.spawn().unwrap();

        let mut stdin = child.stdin.take().unwrap();
        let stdin_thread = std::thread::spawn(move || {
            for msg in rx.iter() {
                writeln!(stdin, "{msg}").unwrap();
            }
        });

        Some(Self {
            child,
            _stdin_thread: Some(stdin_thread),
            tx,
        })
    }

    pub fn set_progress_text(&self, text: &str) {
        self.tx.send(text.to_string()).unwrap();
    }
}

impl Drop for LaunchUI {
    fn drop(&mut self) {
        self.child.kill().unwrap();
    }
}

/// When launched with a flag, this will be used instead of the main xlm logic. This allows
/// us to launch ourselves to show a UI without having to spawn a window from within Tokio.
pub fn launch_ui_main() {
    let progress_text = Arc::new(RwLock::new(String::new()));
    std::thread::spawn({
        let progress_text = progress_text.clone();
        move || {
            let mut line = String::new();
            let mut reader = io::BufReader::new(io::stdin());
            loop {
                line.clear();
                if reader.read_line(&mut line).is_ok() {
                    *progress_text.write().unwrap() = line.trim().to_string();
                }
            }
        }
    });

    eframe::run_simple_native(
        "XLM",
        eframe::NativeOptions {
            event_loop_builder: None,
            run_and_return: true,
            viewport: ViewportBuilder::default()
                .with_inner_size([800.0, 500.0])
                .with_resizable(false)
                .with_decorations(false),
            ..Default::default()
        },
        move |ctx, _frame| {
            ctx.set_pixels_per_point(1.5);
            TopBottomPanel::bottom("bottom").show(ctx, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    ui.add(Spinner::default());
                    ui.label(progress_text.read().unwrap().as_str());
                    ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("XLM v{}", env!("CARGO_PKG_VERSION")));
                        });
                    });
                });
            });
            CentralPanel::default().show(ctx, |ui| {
                ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                    ui.heading("Starting XIVLauncher\n(this may take a moment)");
                });
            });
        },
    )
    .unwrap();
}
