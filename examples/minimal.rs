//! The smallest application that uses a custom title bar.
//!
//! ```text
//! cargo run --example minimal
//! ```
//!
//! Three touchpoints, marked `1.` through `3.` below, and no `window::Id`
//! anywhere in the application state. For menus in the title bar, a window
//! icon, system decorations or theme switching, see the `native_frame`
//! example; for several windows at once, see `minimal_multi_window`.

use iced::widget::{button, center, column, text};
use iced::{Center, Element, Subscription, Task, window};

use iced_native_frame::{FrameAction, NativeFrame, NativeFrameConfig};

fn main() -> iced::Result {
    let frame = NativeFrame::new(NativeFrameConfig::platform_default());

    // 1. Let the frame apply its platform requirements to the window settings,
    //    and install itself on the window Iced opens for us.
    let settings = frame.window_settings(window::Settings {
        size: iced::Size::new(480.0, 320.0),
        ..window::Settings::default()
    });

    iced::application(
        {
            let frame = frame.clone();

            move || {
                (
                    Counter::new(frame.clone()),
                    frame.install_latest().discard(),
                )
            }
        },
        Counter::update,
        Counter::view,
    )
    .window(settings)
    .subscription(Counter::subscription)
    .run()
}

struct Counter {
    frame: NativeFrame,
    value: i32,
}

#[derive(Debug, Clone)]
enum Message {
    Frame(FrameAction),
    Increment,
}

impl Counter {
    fn new(frame: NativeFrame) -> Self {
        Self { frame, value: 0 }
    }

    // 2. Forward the frame's window events, so it can track maximization,
    //    focus and closed windows — then hand every action back to it.
    fn subscription(&self) -> Subscription<Message> {
        self.frame.subscription().map(Message::Frame)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Frame(action) => self.frame.update(action, Message::Frame),

            Message::Increment => {
                self.value += 1;

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = center(
            column![
                text(self.value).size(48),
                button("Increment").on_press(Message::Increment),
            ]
            .spacing(20)
            .align_x(Center),
        );

        // 3. Wrap the application content in the frame.
        self.frame.decorate("Minimal", content, Message::Frame)
    }
}
