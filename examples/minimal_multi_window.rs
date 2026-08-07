//! The same minimal title bar, on as many windows as you like.
//!
//! ```text
//! cargo run --example minimal_multi_window
//! ```
//!
//! `iced::daemon` hands its view function the [`window::Id`] that
//! `iced::application` does not, so the single-window shortcuts —
//! `install_latest` and `decorate` — give way to their explicit forms:
//!
//! | single window            | many windows                        |
//! |--------------------------|-------------------------------------|
//! | `frame.install_latest()` | `frame.install(window_id)`          |
//! | `frame.decorate(..)`     | `frame.view(window_id, ..)`         |
//!
//! One [`NativeFrame`] serves every window. Hover, maximization and focus are
//! tracked per window, so maximizing one leaves the others alone.

use std::collections::HashMap;

use iced::widget::{button, center, column, text};
use iced::{Center, Element, Subscription, Task, window};

use iced_native_frame::{FrameAction, NativeFrame, NativeFrameConfig};

fn main() -> iced::Result {
    // A daemon has no `.window(..)` builder: every window is opened by hand,
    // which is where `window_settings` goes instead.
    iced::daemon(App::new, App::update, App::view)
        .title(App::title)
        .subscription(App::subscription)
        .run()
}

struct App {
    frame: NativeFrame,
    windows: HashMap<window::Id, Window>,
    opened: usize,
}

/// The application's own per-window state. The frame keeps its own, keyed the
/// same way.
struct Window {
    title: String,
    value: i32,
}

#[derive(Debug, Clone)]
enum Message {
    OpenWindow,
    WindowOpened(window::Id),
    Frame(FrameAction),
    Increment(window::Id),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let mut app = Self {
            frame: NativeFrame::new(NativeFrameConfig::platform_default()),
            windows: HashMap::new(),
            opened: 0,
        };

        let open = app.open();

        (app, open)
    }

    /// Opens a window with the frame's platform requirements applied.
    fn open(&mut self) -> Task<Message> {
        self.opened += 1;

        let settings = self.frame.window_settings(window::Settings {
            size: iced::Size::new(420.0, 300.0),
            ..window::Settings::default()
        });

        let (window_id, opened) = window::open(settings);

        self.windows.insert(
            window_id,
            Window {
                title: format!("Window {}", self.opened),
                value: 0,
            },
        );

        opened.map(Message::WindowOpened)
    }

    /// The taskbar title. The one drawn in the title bar is the `title`
    /// argument to `NativeFrame::view` instead.
    fn title(&self, window_id: window::Id) -> String {
        self.windows.get(&window_id).map_or_else(
            || String::from("Multi Window"),
            |window| window.title.clone(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        self.frame.subscription().map(Message::Frame)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenWindow => self.open(),

            // Install once per window, on the window itself.
            Message::WindowOpened(window_id) => self.frame.install(window_id).discard(),

            // A daemon outlives its windows, so the last one to close has to
            // ask the runtime to stop.
            Message::Frame(FrameAction::WindowClosed(window_id)) => {
                let _ = self.windows.remove(&window_id);

                let closed = self
                    .frame
                    .update(FrameAction::WindowClosed(window_id), Message::Frame);

                if self.windows.is_empty() {
                    closed.chain(iced::exit())
                } else {
                    closed
                }
            }

            Message::Frame(action) => self.frame.update(action, Message::Frame),

            Message::Increment(window_id) => {
                if let Some(window) = self.windows.get_mut(&window_id) {
                    window.value += 1;
                }

                Task::none()
            }
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        let Some(window) = self.windows.get(&window_id) else {
            return center(text("This window is closing.")).into();
        };

        let content = center(
            column![
                text(&window.title).size(20),
                text(window.value).size(48),
                button("Increment").on_press(Message::Increment(window_id)),
                button("Open another window").on_press(Message::OpenWindow),
            ]
            .spacing(16)
            .align_x(Center),
        );

        self.frame.view(
            window_id,
            &window.title,
            None,
            None,
            content,
            Message::Frame,
        )
    }
}
