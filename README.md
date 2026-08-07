# iced-native-frame

A custom, theme-aware window frame for [Iced], pinned to:

```toml
iced = {
    git = "https://github.com/iced-rs/iced",
    rev = "4cad51ba61e3d76a7e696ad5923d675a20994e34",
    features = ["svg"],
}
```

The title bar is built out of ordinary Iced widgets and driven by Iced's
winit-backed `window::*` API. The only native code in the library is a
`WM_NCHITTEST` subclass on Windows, which is what the Windows 11 Snap Layout
flyout requires and what winit does not expose.

## Architecture

```
src/
├── lib.rs                        public re-exports
└── native_frame/
    ├── mod.rs                    NativeFrame: view, update, subscription, settings
    ├── config.rs                 DecorationMode, NativeFrameConfig
    ├── action.rs                 FrameAction, CaptionControl
    ├── icons.rs                  embedded Lucide glyphs
    ├── style.rs                  theme palette -> frame styling
    ├── resize_handles.rs         invisible edge handles (X11 / Wayland)
    └── platform/
        ├── mod.rs                dispatch + capability constants
        ├── windows.rs            subclass, WM_NCHITTEST, caption regions
        ├── linux.rs              settings + capability query (no native code)
        └── macos.rs              settings only (no Objective-C / AppKit)
```

Everything portable goes through Iced: `window::drag`, `window::drag_resize`,
`window::minimize`, `window::toggle_maximize`, `window::close`,
`window::show_system_menu` and `window::is_maximized`.

No raw X11, Wayland, Cocoa, Objective-C or AppKit handle is touched anywhere in
the crate.

## Usage

```rust
let frame = NativeFrame::new(
    NativeFrameConfig::platform_default()
        .decoration_mode(DecorationMode::Custom),
);

// The one helper. It replaces every platform `#[cfg]` a consumer would
// otherwise have to write.
let settings = frame.window_settings(window::Settings {
    resizable: true,
    ..Default::default()
});
```

```rust
frame.view(
    window_id,
    "Application",
    Some(window_icon),
    Some(title_content),
    body,
    Message::Frame,
)
```

```rust
// In your update:
Message::Frame(action) => frame.update(action, Message::Frame),

// In your subscription. Forward it, or the frame will keep per-window state
// for windows that no longer exist.
frame.subscription().map(Message::Frame)
```

### Multiple windows

State — hover, pressed, maximized, focused — is keyed by `window::Id`, so one
`NativeFrame` can drive any number of windows without them colliding. Call
`install` and `view` once per window; `subscription` releases a window's state
when it closes. Clones of a frame share the same registry.

### Decoration modes

| | `Custom` | `System` |
|---|---|---|
| `window::Settings.decorations` | `false` (`true` on macOS) | `true` |
| Custom frame rendered | yes | no |
| Win32 subclass installed | Windows only | never |

`System` means "use the normal platform-managed decoration path". It is
deliberately *not* called `ServerSide`: a Wayland compositor may still negotiate
client-side decorations when an application asks for normal ones.

### Title-bar layout

```
[leading inset][clickable icon][draggable title][application-owned fill row][caption controls]
```

The application-owned row consumes all width between the title and the caption
controls, so this works as written:

```rust
row![
    file_button,
    edit_button,
    view_button,
    space::horizontal(),
    settings_button,
]
.width(Fill)
```

`MouseArea` checks `shell.is_event_captured()` before acting, so the buttons
keep their clicks while the gaps between them fall through to the draggable
region behind. Double-clicking non-interactive title-bar space toggles maximize;
right-clicking it requests the system menu. The cursor stays a normal arrow —
no grab cursor.

### The application icon

Clicking the icon publishes `FrameAction::WindowIconPressed`, which is distinct
from `FrameAction::ShowSystemMenu`. Forwarding it to `NativeFrame::update` runs
the default behavior (show the system menu where the platform has one).
Intercept it in your own `update` to do something else — check
`NativeFrame::supports_system_menu()` to decide.

## Public API

```rust
pub enum DecorationMode { Custom, System }
impl DecorationMode {
    pub fn uses_custom_frame(self) -> bool;
    pub fn window_decorations(self) -> bool;
}

pub struct NativeFrameConfig { /* fields are public; builder methods too */ }
impl NativeFrameConfig {
    pub fn platform_default() -> Self;
    // .decoration_mode() .title_bar_height() .caption_button_width()
    // .caption_buttons() .resize_border() .resizable() .corner_radius()
    // .frame_border() .client_shadow() .outer_padding() .native_rounding()
    // .native_shadow() .window_icon_size() .title_spacing() .title_padding()
    // .leading_inset() .show_title()
}

// Every variant names its window, so one frame can serve many windows.
pub enum FrameAction {
    Drag(window::Id),
    Resize(window::Id, Direction),
    Minimize(window::Id),
    ToggleMaximize(window::Id),
    Close(window::Id),
    WindowIconPressed(window::Id),
    ShowSystemMenu(window::Id),
    Hover(window::Id, CaptionControl),
    Leave(window::Id, CaptionControl),
    MaximizedChanged(window::Id, bool),
    FocusChanged(window::Id, bool),
    SyncState(window::Id),
    WindowClosed(window::Id),
}
impl FrameAction {
    pub fn window_id(self) -> window::Id;
}

pub enum CaptionControl { Minimize, Maximize, Close }
impl CaptionControl { pub const ALL: [Self; 3]; }

pub struct NativeFrame;
impl NativeFrame {
    pub fn new(config: NativeFrameConfig) -> Self;
    pub fn config(&self) -> NativeFrameConfig;
    pub fn decoration_mode(&self) -> DecorationMode;
    pub fn is_maximized(&self, id: window::Id) -> bool;
    pub fn is_active(&self, id: window::Id) -> bool;
    pub fn tracked_windows(&self) -> usize;
    pub fn supports_system_menu(&self) -> bool;
    pub fn window_settings(&self, settings: window::Settings) -> window::Settings;
    pub fn install(&self, id: window::Id) -> Task<Result<(), String>>;
    pub fn uninstall(&self, id: window::Id) -> Task<()>;
    pub fn subscription(&self) -> Subscription<FrameAction>;
    pub fn update<Message>(&self, action: FrameAction, map_action: ...) -> Task<Message>;
    pub fn view<'a, Message>(&self, ...) -> Element<'a, Message>;

    // Single-window shortcuts. `iced::application` does not hand its view
    // function a `window::Id` — only `iced::daemon` does — so these read it
    // back from the frame and the application never has to hold one.
    pub fn install_latest(&self) -> Task<Result<(), String>>;
    pub fn primary_window(&self) -> Option<window::Id>;
    pub fn decorate<'a, Message>(&self, title, content, map_action) -> Element<'a, Message>;
}
```

## Platform behavior

### Windows

The subclass answers `WM_NCHITTEST` and nothing else of consequence:

* `HTMINBUTTON` / `HTMAXBUTTON` / `HTCLOSE` for the three caption controls —
  `HTMAXBUTTON` is what opens the Windows 11 Snap Layout flyout
* all eight native resize regions, with native cursors
* `HTCLIENT` everywhere else, so Iced keeps receiving title-bar input
* minimal non-client mouse tracking, so the frame paints its own hover and
  pressed states instead of letting `DefWindowProc` run its modal
  caption-button loop
* maximized and activation state mirrored back into the frame

Rounded corners come from `window::Settings.platform_specific.corner_preference`
and the optional extra shadow from `undecorated_shadow`. The crate makes no DWM
calls of its own; the previous direct `DwmSetWindowAttribute` calls were
removed only after verifying the replacements on a live window.

`HTSYSMENU` is no longer used: the application icon is `HTCLIENT` so Iced can
publish `WindowIconPressed` for it.

### Linux — X11 and Wayland

No native code. Custom mode uses an undecorated window plus eight invisible
resize handles that call `window::drag_resize` straight from the mouse press.
The handles only react within `resize_border` of each edge; the space between
them is an inert `space()` that blocks neither the cursor nor events.

**Wayland.** The compositor owns move and resize; the frame only requests them
and never repositions the window itself. `System` mode requests normal
decorations, but the compositor decides whether the result is server-side or
client-side.

**X11.** Custom mode is an undecorated window; system mode lets the window
manager decorate it.

Client-side shadow and rounding are configurable via `client_shadow`,
`outer_padding` and `corner_radius`, and default to **off** — most compositors
include transparent outer padding in tiled and maximized geometry, which makes
the window look inset.

### macOS

Custom mode is a native hybrid title bar, not a copy of the Windows one:

* `decorations = true`, plus `title_hidden`, `titlebar_transparent` and
  `fullsize_content_view`
* native traffic lights stay visible and functional
* native shadow, rounding, edge resizing and full-screen all preserved
* no fake minimize / maximize / close buttons (`caption_buttons: false`)
* `leading_inset` (78 px by default) keeps content clear of the traffic lights

## Limitations

* **X11** — `window::show_system_menu` resolves to winit's X11
  `show_window_menu`, which is an empty function in the pinned winit revision
  (`05b8ff17`). The call is a silent no-op. The crate does not add raw
  Xlib/XCB code for it; handle `WindowIconPressed` yourself instead.
  `supports_system_menu()` returns `false` there.
* **Wayland** — client-side shadow and rounded corners depend entirely on the
  compositor. Transparent `outer_padding` can be counted in tiled/maximized
  geometry, so it is off by default.
* **macOS** — the traffic lights cannot be repositioned without `NSWindow`
  access, so a title bar shorter than ~28 px will clip them. There is no custom
  edge resizing in custom mode; the native decorations handle it.
  `show_system_menu` is a no-op (macOS has no per-window system menu).
* **Linux** — `supports_system_menu()` reports the backend observed when the
  first window was installed, read from the `RawWindowHandle` variant winit
  produced. Before that first install it falls back to the same
  `WAYLAND_DISPLAY` / `WAYLAND_SOCKET` check winit uses, which can disagree
  with reality if Iced was built without the `wayland` feature.
* **Windows** — the caption *command* for a completed click stays native
  (`WM_SYSCOMMAND`). Routing it back through `window::minimize` /
  `toggle_maximize` / `close` would have to travel through the Iced message
  loop, by which point the non-client mouse capture Windows established for the
  click is gone. Drag, resize-drag and the double-click-to-maximize on ordinary
  title-bar space all use the portable Iced path.

## Examples

`minimal` — the smallest single-window application. Three touchpoints, and no
`window::Id` in the application state:

```bash
cargo run --example minimal
```

`minimal_multi_window` — the same title bar on any number of windows, through
`iced::daemon`. One `NativeFrame` serves them all, with hover, maximization and
focus tracked per window:

```bash
cargo run --example minimal_multi_window
```

`native_frame` — every feature at once: title-bar menus, a clickable
application icon with an application-owned fallback menu, all eight resize
directions and live theme switching.

```bash
cargo run --example native_frame -- --decorations custom
```

```bash
cargo run --example native_frame -- --decorations system
```

The mode can also come from `ICED_NATIVE_FRAME_DECORATIONS=custom|system`. The
same binary covers Windows, X11, Wayland and macOS with no source edits.

### Testing the two Linux backends separately

winit picks its backend from the environment: Wayland whenever `WAYLAND_DISPLAY`
or `WAYLAND_SOCKET` is set and non-empty, otherwise X11 when `DISPLAY` is set.
(`WINIT_UNIX_BACKEND` was removed in winit 0.29 and has no effect.)

Native Wayland:

```bash
env -u DISPLAY cargo run --example native_frame
```

X11 / Xwayland:

```bash
env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET cargo run --example native_frame
```

## Validation

```bash
cargo fmt --all
```

```bash
cargo check --all-targets
```

```bash
cargo test
```

[Iced]: https://iced.rs
