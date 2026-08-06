use iced::widget::{
    button, center, container, row, text,
};
use iced::{
    Element, Fill, Task, Theme, window,
};

use iced_native_frame::{
    FrameAction, NativeFrame,
};

fn main() -> iced::Result {
    iced::application(
        App::new,
        App::update,
        App::view,
    )
    .window(window::Settings {
        decorations: false,
        resizable: true,

        // Client-side shadows and transparent rounded corners need a
        // transparent native surface on Linux.
        //
        // Windows uses DWM rounding and shadow instead.
        transparent: cfg!(target_os = "linux"),

        ..window::Settings::default()
    })
    .theme(App::theme)
    .run()
}

#[derive(Debug)]
struct App {
    frame: NativeFrame,
    window_id: Option<window::Id>,
    frame_error: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    WindowFound(Option<window::Id>),
    FrameInstalled(Result<(), String>),
    Frame(FrameAction),

    FilePressed,
    EditPressed,
    ViewPressed,
    SettingsPressed,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                frame: NativeFrame::default(),
                window_id: None,
                frame_error: None,
            },
            window::latest().map(Message::WindowFound),
        )
    }

    fn update(
        &mut self,
        message: Message,
    ) -> Task<Message> {
        match message {
            Message::WindowFound(Some(window_id)) => {
                self.window_id = Some(window_id);

                self.frame
                    .install(window_id)
                    .map(Message::FrameInstalled)
            }

            Message::WindowFound(None) => {
                Task::none()
            }

            Message::FrameInstalled(Ok(())) => {
                Task::none()
            }

            Message::FrameInstalled(Err(error)) => {
                eprintln!(
                    "Native frame installation failed: {error}"
                );

                self.frame_error = Some(error);

                Task::none()
            }

            Message::Frame(action) => {
                self.frame.update(action)
            }

            Message::EditPressed | Message::SettingsPressed | Message::ViewPressed | Message::FilePressed => {
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let body = center(
            container(
                text(
                    self.frame_error
                        .as_deref()
                        .unwrap_or("Iced native frame"),
                )
                .size(24),
            )
            .width(Fill)
            .height(Fill),
        );

        let Some(window_id) = self.window_id else {
            return body.into();
        };

        let title_icon: Element<'_, Message> =
            iced::widget::svg(
                iced::widget::svg::Handle::from_path(
                    "assets/app-icon.svg",
                ),
            )
            .width(Fill)
            .height(Fill)
            .into();

        let title_content: Element<'_, Message> = row![
            iced::widget::button(text("File").size(12.0))
                .on_press(Message::FilePressed)
                .style(button::text)
                .padding([3, 7]),

            iced::widget::button(text("Edit").size(12.0))
                .on_press(Message::EditPressed)
                .style(button::text)
                .padding([3, 7]),

            iced::widget::button(text("View").size(12.0))
                .on_press(Message::ViewPressed)
                .style(button::text)
                .padding([3, 7]),

            iced::widget::space().width(iced::Length::Fill),

            iced::widget::button(text("Settings").size(12.0))
                .on_press(Message::SettingsPressed)
                .style(button::text)
                .padding([3, 7]),
        ]
        .spacing(2)
        .align_y(iced::Alignment::Center)
        .into();

        self.frame.view(
            window_id,
            "Iced Native Frame",
            Some(title_icon),
            Some(title_content),
            body,
            Message::Frame,
        )
    }

    fn theme(&self) -> Theme {
        Theme::GruvboxDark
    }
}