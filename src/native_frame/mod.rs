mod icons;
mod platform;

use iced::widget::{column, container, row, text};
use iced::{
    Alignment, Border, Center, Color, Element, Fill, Shadow, Task,
    Theme, Vector, window,
};

use iced::mouse;
use iced::widget::mouse_area;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

#[derive(Debug, Clone, Copy)]
pub struct NativeFrameConfig {
    pub title_bar_height: f32,
    pub caption_button_width: f32,
    pub resize_border: f32,
    pub system_menu_width: f32,
    pub resizable: bool,

    pub corner_radius: f32,
    pub client_shadow: bool,
    pub outer_padding: f32,
    pub native_rounding: bool,
    pub native_shadow: bool,

    pub window_icon_size: f32,
    pub title_spacing: f32,
    pub title_padding: f32,
}

impl NativeFrameConfig {
    pub fn platform_default() -> Self {
        #[cfg(windows)]
        {
            Self {
                title_bar_height: 32.0,
                caption_button_width: 46.0,
                resize_border: 6.0,
                system_menu_width: 0.0,
                resizable: true,

                corner_radius: 8.0,
                client_shadow: false,
                outer_padding: 0.0,
                native_rounding: true,
                native_shadow: true,

                window_icon_size: 16.0,
                title_spacing: 8.0,
                title_padding: 8.0,
            }
        }

        #[cfg(target_os = "linux")]
        {
            Self {
                title_bar_height: 34.0,
                caption_button_width: 44.0,
                resize_border: 6.0,
                system_menu_width: 0.0,
                resizable: true,

                corner_radius: 8.0,
                client_shadow: true,
                outer_padding: 12.0,
                native_rounding: false,
                native_shadow: false,

                window_icon_size: 16.0,
                title_spacing: 8.0,
                title_padding: 8.0,
            }
        }

        #[cfg(not(any(windows, target_os = "linux")))]
        {
            Self {
                title_bar_height: 34.0,
                caption_button_width: 44.0,
                resize_border: 6.0,
                system_menu_width: 0.0,
                resizable: true,

                corner_radius: 8.0,
                client_shadow: false,
                outer_padding: 0.0,
                native_rounding: false,
                native_shadow: false,

                window_icon_size: 16.0,
                title_spacing: 8.0,
                title_padding: 8.0,
            }
        }
    }
}

impl Default for NativeFrameConfig {
    fn default() -> Self {
        Self::platform_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CaptionControl {
    None = 0,
    Minimize = 1,
    Maximize = 2,
    Close = 3,
}

impl CaptionControl {
    pub(crate) fn from_raw(value: u8) -> Self {
        match value {
            1 => Self::Minimize,
            2 => Self::Maximize,
            3 => Self::Close,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FrameAction {
    Drag(window::Id),
    Minimize(window::Id),
    ToggleMaximize(window::Id),
    Close(window::Id),
    ShowSystemMenu(window::Id),
    Resize(window::Id, window::Direction),
    Hover(CaptionControl),
    Leave(CaptionControl),
    MaximizedChanged(bool),
}

#[derive(Debug)]
pub(crate) struct Shared {
    pub(crate) config: NativeFrameConfig,
    pub(crate) hot: AtomicU8,
    pub(crate) pressed: AtomicU8,
    pub(crate) maximized: AtomicBool,
    pub(crate) active: AtomicBool,
}

impl Shared {
    fn new(config: NativeFrameConfig) -> Self {
        Self {
            config,
            hot: AtomicU8::new(CaptionControl::None as u8),
            pressed: AtomicU8::new(CaptionControl::None as u8),
            maximized: AtomicBool::new(false),
            active: AtomicBool::new(true),
        }
    }

    pub(crate) fn hot(&self) -> CaptionControl {
        CaptionControl::from_raw(self.hot.load(Ordering::Acquire))
    }

    pub(crate) fn pressed(&self) -> CaptionControl {
        CaptionControl::from_raw(self.pressed.load(Ordering::Acquire))
    }

    pub(crate) fn set_hot(&self, value: CaptionControl) {
        self.hot.store(value as u8, Ordering::Release);
    }

    pub(crate) fn set_pressed(&self, value: CaptionControl) {
        self.pressed.store(value as u8, Ordering::Release);
    }

    pub(crate) fn is_maximized(&self) -> bool {
        self.maximized.load(Ordering::Acquire)
    }

    pub(crate) fn set_maximized(&self, value: bool) {
        self.maximized.store(value, Ordering::Release);
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone)]
pub struct NativeFrame {
    shared: Arc<Shared>,
}

impl NativeFrame {
    pub fn new(config: NativeFrameConfig) -> Self {
        Self {
            shared: Arc::new(Shared::new(config)),
        }
    }

    pub fn config(&self) -> NativeFrameConfig {
        self.shared.config
    }

    pub fn is_maximized(&self) -> bool {
        self.shared.is_maximized()
    }

    pub fn install(
        &self,
        window_id: window::Id,
    ) -> Task<Result<(), String>> {
        platform::install(window_id, Arc::clone(&self.shared))
    }

    pub fn update<Message>(
        &self,
        action: FrameAction,
    ) -> Task<Message>
    where
        Message: Send + 'static,
    {
        match action {
            FrameAction::Drag(id) => window::drag(id),
            FrameAction::Minimize(id) => window::minimize(id, true),
            FrameAction::Close(id) => window::close(id),
            FrameAction::ShowSystemMenu(id) => window::show_system_menu(id),
            FrameAction::Resize(id, direction) => {
                window::drag_resize(id, direction)
            }

            FrameAction::ToggleMaximize(id) => {
                self.shared.set_maximized(!self.shared.is_maximized());
                window::toggle_maximize(id)
            }

            FrameAction::Hover(control) => {
                self.shared.set_hot(control);
                Task::none()
            }

            FrameAction::Leave(control) => {
                if self.shared.hot() == control {
                    self.shared.set_hot(CaptionControl::None);
                }

                self.shared.set_pressed(CaptionControl::None);
                Task::none()
            }

            FrameAction::MaximizedChanged(maximized) => {
                self.shared.set_maximized(maximized);
                Task::none()
            }
        }
    }

    pub fn view<'a, Message>(
        &self,
        window_id: window::Id,
        title: &'a str,
        window_icon: Option<Element<'a, Message>>,
        title_content: Option<Element<'a, Message>>,
        content: impl Into<Element<'a, Message>>,
        map_action: impl Fn(FrameAction) -> Message + Clone + 'a,
    ) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        let config = self.shared.config;

        let surface = column![
            self.title_bar(
                window_id,
                title,
                window_icon,
                title_content,
                map_action,
            ),
            content.into(),
        ]
        .width(Fill)
        .height(Fill);

        container(
            container(surface)
                .width(Fill)
                .height(Fill)
                .clip(true)
                .style(move |theme| {
                    surface_style(theme, config)
                }),
        )
        .padding(config.outer_padding)
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn title_bar<'a, Message>(
        &self,
        window_id: window::Id,
        title: &'a str,
        window_icon: Option<Element<'a, Message>>,
        title_content: Option<Element<'a, Message>>,
        map_action: impl Fn(FrameAction) -> Message + Clone + 'a,
    ) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        let config = self.shared.config;

        /*
        * Leading title region:
        *
        * [ clickable icon ][ draggable title ]
        *
        * The whole region is wrapped in a draggable MouseArea. The nested icon
        * MouseArea captures its own press first, preventing the parent from
        * starting a window drag when the icon is clicked.
        */
        let mut leading_row = row![]
            .spacing(config.title_spacing)
            .align_y(Center);

        if let Some(icon) = window_icon {
            let icon_slot = container(icon)
                .width(config.window_icon_size)
                .height(config.window_icon_size)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .clip(true);

            let clickable_icon: Element<'a, Message> =
                mouse_area(icon_slot)
                    .on_press(
                        map_action.clone()(
                            FrameAction::ShowSystemMenu(window_id),
                        ),
                    )
                    // Keep the normal arrow cursor, like a native title-bar icon.
                    .into();

            leading_row = leading_row.push(clickable_icon);
        }

        leading_row = leading_row.push(
            text(title)
                .size(13),
        );

        let leading = container(leading_row)
            .padding(iced::Padding {
                top: 0.0,
                right: config.title_padding,
                bottom: 0.0,
                left: config.title_padding,
            })
            .height(config.title_bar_height)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Center);

        let leading = self.draggable_region(
            window_id,
            leading,
            map_action.clone(),
        );

        /*
        * Application-owned row.
        *
        * It consumes all width remaining between the title and caption buttons.
        *
        * Its parent MouseArea starts a drag only when a child did not capture
        * the event. Therefore:
        *
        * - Buttons remain clickable.
        * - Menus remain clickable.
        * - Inputs remain interactive.
        * - Empty containers and Space::horizontal() drag the window.
        */
        let title_content: Element<'a, Message> = match title_content {
            Some(content) => container(content)
                .width(Fill)
                .height(config.title_bar_height)
                .align_x(iced::alignment::Horizontal::Left)
                .align_y(iced::alignment::Vertical::Center)
                .into(),

            None => container(row![])
                .width(Fill)
                .height(config.title_bar_height)
                .into(),
        };

        let title_content = self.draggable_region(
            window_id,
            title_content,
            map_action.clone(),
        );

        /*
        * Fixed-width native caption controls.
        */
        let controls = row![
            self.caption_button(
                window_id,
                CaptionControl::Minimize,
                map_action.clone(),
            ),
            self.caption_button(
                window_id,
                CaptionControl::Maximize,
                map_action.clone(),
            ),
            self.caption_button(
                window_id,
                CaptionControl::Close,
                map_action,
            ),
        ]
        .height(config.title_bar_height)
        .align_y(Center);

        container(
            row![
                leading,
                title_content,
                controls,
            ]
            .width(Fill)
            .height(config.title_bar_height)
            .align_y(Center),
        )
        .width(Fill)
        .height(config.title_bar_height)
        .style(title_bar_style)
        .into()
    }

    fn draggable_region<'a, Message>(
        &self,
        window_id: window::Id,
        content: impl Into<Element<'a, Message>>,
        map_action: impl Fn(FrameAction) -> Message + Clone + 'a,
    ) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        mouse_area(content)
            .on_press(
                map_action.clone()(
                    FrameAction::Drag(window_id),
                ),
            )
            .on_double_click(
                map_action.clone()(
                    FrameAction::ToggleMaximize(window_id),
                ),
            )
            .on_right_press(
                map_action(
                    FrameAction::ShowSystemMenu(window_id),
                ),
            )
            .into()
    }

    fn caption_button<'a, Message>(
        &self,
        window_id: window::Id,
        control: CaptionControl,
        map_action: impl Fn(FrameAction) -> Message + Clone + 'a,
    ) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        let config = self.shared.config;

        #[cfg(windows)]
        let _ = (window_id, &map_action);

        let icon_size = match control {
            CaptionControl::Minimize => 16.0,
            CaptionControl::Maximize => 12.0,
            CaptionControl::Close => 16.0,
            CaptionControl::None => 0.0,
        };

        let icon = iced::widget::svg(
            icons::handle(control),
        )
        .width(icon_size)
        .height(icon_size)
        .style({
            let frame = self.clone();

            move |theme: &Theme, _status| {
                let palette = theme.palette();
                let hot = frame.shared.hot() == control;

                let color = if control == CaptionControl::Close && hot {
                    Color::WHITE
                } else if frame.shared.is_active() {
                    palette.background.strong.text
                } else {
                    palette.background.weak.text
                };

                iced::widget::svg::Style {
                    color: Some(color),
                }
            }
        });

        let visual = container(icon)
            .width(config.caption_button_width)
            .height(config.title_bar_height)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style({
                let frame = self.clone();

                move |theme| {
                    frame.caption_style(theme, control)
                }
            });

        #[cfg(windows)]
        {
            visual.into()
        }

        #[cfg(not(windows))]
        {
            let action = match control {
                CaptionControl::Minimize => {
                    FrameAction::Minimize(window_id)
                }

                CaptionControl::Maximize => {
                    FrameAction::ToggleMaximize(window_id)
                }

                CaptionControl::Close => {
                    FrameAction::Close(window_id)
                }

                CaptionControl::None => unreachable!(),
            };

            mouse_area(visual)
                .on_enter(
                    map_action.clone()(FrameAction::Hover(control)),
                )
                .on_exit(
                    map_action.clone()(FrameAction::Leave(control)),
                )
                .on_press(map_action(action))
                .interaction(mouse::Interaction::Pointer)
                .into()
        }
    }

    fn caption_style(
        &self,
        theme: &Theme,
        control: CaptionControl,
    ) -> container::Style {
        let palette = theme.palette();

        let hot = self.shared.hot() == control;
        let pressed = self.shared.pressed() == control;

        let background = match (control, hot, pressed) {
            (CaptionControl::Close, true, true) => {
                Color::from_rgb8(150, 20, 20)
            }

            (CaptionControl::Close, true, false) => {
                Color::from_rgb8(196, 43, 28)
            }

            (_, _, true) => {
                palette.background.strong.color
            }

            (_, true, false) => {
                palette.background.weak.color
            }

            _ => Color::TRANSPARENT,
        };

        container::Style::default().background(background)
    }
}

impl Default for NativeFrame {
    fn default() -> Self {
        Self::new(NativeFrameConfig::default())
    }
}

fn title_bar_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();

    container::Style::default()
        .background(palette.background.weakest.color)
        .color(palette.background.weakest.text)
}

fn surface_style(
    theme: &Theme,
    config: NativeFrameConfig,
) -> container::Style {
    let palette = theme.palette();

    let shadow = if config.client_shadow && config.outer_padding > 0.0 {
        Shadow {
            color: Color {
                a: 0.35,
                ..Color::BLACK
            },
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        }
    } else {
        Shadow::default()
    };

    container::Style::default()
        .background(palette.background.base.color)
        .color(palette.background.base.text)
        .border(Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: config.corner_radius.into(),
        })
        .shadow(shadow)
}