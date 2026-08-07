//! A custom, theme-aware window frame for [Iced].
//!
//! `iced-native-frame` draws a title bar out of ordinary Iced widgets and
//! wires it to the platform through Iced's winit-backed [`iced::window`] API.
//! The only native code it contains is a `WM_NCHITTEST` subclass on Windows,
//! which is what the Windows 11 Snap Layout flyout requires and what winit
//! does not expose.
//!
//! ```ignore
//! use iced_native_frame::{DecorationMode, NativeFrame, NativeFrameConfig};
//!
//! let frame = NativeFrame::new(
//!     NativeFrameConfig::platform_default()
//!         .decoration_mode(DecorationMode::Custom),
//! );
//!
//! let settings = frame.window_settings(iced::window::Settings {
//!     resizable: true,
//!     ..Default::default()
//! });
//! ```
//!
//! A single-window application then needs only [`NativeFrame::install_latest`]
//! in its `boot` and [`NativeFrame::decorate`] in its `view`, and never has to
//! name an [`iced::window::Id`] at all — see the `minimal` example. Windows
//! opened by hand use [`NativeFrame::install`] and [`NativeFrame::view`]
//! instead; see `minimal_multi_window`.
//!
//! [Iced]: https://iced.rs

pub mod native_frame;

pub use native_frame::NativeFrame;
pub use native_frame::action::{CaptionControl, FrameAction};
pub use native_frame::config::{DecorationMode, NativeFrameConfig};
