# iced-native-frame

Prototype custom frame for Iced pinned to:

```toml
iced = {
    git = "https://github.com/iced-rs/iced",
    rev = "4cad51ba61e3d76a7e696ad5923d675a20994e34",
    features = ["canvas"],
}
```

## Platform behavior

### Windows

- Native `WM_NCHITTEST` caption regions
- Windows 11 Snap Layout hover
- Native dragging and eight-direction resizing
- Native minimize, maximize/restore, and close commands
- DWM rounded-corner request
- Best-effort DWM native shadow

### Linux

- Iced/winit drag, minimize, maximize/restore, and close operations
- Client-side rounded surface
- Client-side shadow with transparent window and outer padding
- Actual shadow/rounding behavior can vary by X11 window manager or Wayland
  compositor

The default Windows title-bar height is 32 logical pixels. Linux defaults to
34 logical pixels.
