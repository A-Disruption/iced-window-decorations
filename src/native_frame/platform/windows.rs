use iced::Task;
use iced::window;
use iced::window::raw_window_handle::RawWindowHandle;

use std::collections::HashMap;
use std::ffi::c_void;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use windows_sys::Win32::Foundation::{
    HWND, LPARAM, LRESULT, POINT, RECT, WPARAM,
};
use windows_sys::Win32::Graphics::Dwm::{
    DWMNCRP_ENABLED, DWMNCRP_USEWINDOWSTYLE,
    DWMWA_NCRENDERING_POLICY, DWMWA_WINDOW_CORNER_PREFERENCE,
    DWMWCP_DONOTROUND, DWMWCP_ROUND,
    DwmSetWindowAttribute,
};
use windows_sys::Win32::Graphics::Gdi::{
    InvalidateRect, ScreenToClient,
};
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    TRACKMOUSEEVENT, TME_LEAVE, TME_NONCLIENT, TrackMouseEvent,
};
use windows_sys::Win32::UI::Shell::{
    DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetClientRect, IsZoomed, PostMessageW,
    HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT, HTCAPTION, HTCLIENT,
    HTCLOSE, HTLEFT, HTMAXBUTTON, HTMINBUTTON, HTRIGHT,
    HTSYSMENU, HTTOP, HTTOPLEFT, HTTOPRIGHT,
    SC_CLOSE, SC_MAXIMIZE, SC_MINIMIZE, SC_RESTORE,
    SIZE_MAXIMIZED,
    WM_CAPTURECHANGED, WM_NCACTIVATE, WM_NCDESTROY,
    WM_NCHITTEST, WM_NCLBUTTONDOWN, WM_NCLBUTTONUP,
    WM_NCMOUSELEAVE, WM_NCMOUSEMOVE, WM_SIZE, WM_SYSCOMMAND,
};

use crate::native_frame::{CaptionControl, Shared};

const SUBCLASS_ID: usize = 0x4943_4544;

type WindowKey = usize;
type Registry = HashMap<WindowKey, Arc<Shared>>;

static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();

fn window_key(hwnd: HWND) -> WindowKey {
    hwnd as usize
}

fn registry() -> &'static Mutex<Registry> {
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn registry_guard() -> std::sync::MutexGuard<'static, Registry> {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(crate) fn install(
    window_id: window::Id,
    shared: Arc<Shared>,
) -> Task<Result<(), String>> {
    window::run(window_id, move |window| {
        let handle = window.window_handle().map_err(|error| {
            format!("Could not obtain native window handle: {error}")
        })?;

        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            return Err(
                "Iced window did not provide a Win32 window handle"
                    .to_owned(),
            );
        };

        let hwnd = handle.hwnd.get() as HWND;

        shared.maximized.store(
            unsafe { IsZoomed(hwnd) != 0 },
            Ordering::Release,
        );

        registry_guard().insert(window_key(hwnd), Arc::clone(&shared));

        let installed = unsafe {
            SetWindowSubclass(
                hwnd,
                Some(subclass_proc),
                SUBCLASS_ID,
                0,
            )
        };

        if installed == 0 {
            registry_guard().remove(&window_key(hwnd));

            return Err(
                "SetWindowSubclass failed for the Iced window"
                    .to_owned(),
            );
        }

        apply_dwm_style(hwnd, &shared);

        unsafe {
            InvalidateRect(hwnd, ptr::null(), 0);
        }

        Ok(())
    })
}

fn apply_dwm_style(
    hwnd: HWND,
    shared: &Shared,
) {
    let corner_preference = if shared.config.native_rounding
        && shared.config.corner_radius > 0.0
    {
        DWMWCP_ROUND
    } else {
        DWMWCP_DONOTROUND
    };

    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            &corner_preference as *const _ as *const c_void,
            mem::size_of_val(&corner_preference) as u32,
        );
    }

    let rendering_policy = if shared.config.native_shadow {
        DWMNCRP_ENABLED
    } else {
        DWMNCRP_USEWINDOWSTYLE
    };

    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_NCRENDERING_POLICY as u32,
            &rendering_policy as *const _ as *const c_void,
            mem::size_of_val(&rendering_policy) as u32,
        );
    }
}

unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    _reference_data: usize,
) -> LRESULT {
    if message == WM_NCDESTROY {
        registry_guard().remove(&window_key(hwnd));

        unsafe {
            RemoveWindowSubclass(
                hwnd,
                Some(subclass_proc),
                SUBCLASS_ID,
            );

            return DefSubclassProc(
                hwnd,
                message,
                wparam,
                lparam,
            );
        }
    }

    let shared = registry_guard()
        .get(&window_key(hwnd))
        .cloned();

    let Some(shared) = shared else {
        return unsafe {
            DefSubclassProc(hwnd, message, wparam, lparam)
        };
    };

    match message {
        WM_NCHITTEST => {
            let hit = hit_test(hwnd, lparam, &shared);
            let control = CaptionControl::from_hit_test(hit);

            set_control(hwnd, &shared.hot, control);

            if control != CaptionControl::None {
                track_non_client_leave(hwnd);
            }

            return hit;
        }

        WM_NCMOUSEMOVE => {
            let control =
                CaptionControl::from_hit_test(wparam as LRESULT);

            set_control(hwnd, &shared.hot, control);
            track_non_client_leave(hwnd);
        }

        WM_NCMOUSELEAVE => {
            set_control(
                hwnd,
                &shared.hot,
                CaptionControl::None,
            );

            set_control(
                hwnd,
                &shared.pressed,
                CaptionControl::None,
            );
        }

        WM_NCLBUTTONDOWN => {
            let control =
                CaptionControl::from_hit_test(wparam as LRESULT);

            if control != CaptionControl::None {
                set_control(hwnd, &shared.pressed, control);
                set_control(hwnd, &shared.hot, control);
                track_non_client_leave(hwnd);

                return 0;
            }
        }

        WM_NCLBUTTONUP => {
            let released =
                CaptionControl::from_hit_test(wparam as LRESULT);

            let pressed = shared.pressed();

            set_control(
                hwnd,
                &shared.pressed,
                CaptionControl::None,
            );

            if pressed != CaptionControl::None
                && pressed == released
            {
                execute_caption_command(hwnd, pressed);
            }

            if released != CaptionControl::None {
                return 0;
            }
        }

        WM_CAPTURECHANGED => {
            set_control(
                hwnd,
                &shared.pressed,
                CaptionControl::None,
            );
        }

        WM_SIZE => {
            set_bool(
                hwnd,
                &shared.maximized,
                wparam as u32 == SIZE_MAXIMIZED,
            );
        }

        WM_NCACTIVATE => {
            set_bool(
                hwnd,
                &shared.active,
                wparam != 0,
            );
        }

        _ => {}
    }

    unsafe {
        DefSubclassProc(hwnd, message, wparam, lparam)
    }
}

fn hit_test(
    hwnd: HWND,
    lparam: LPARAM,
    shared: &Shared,
) -> LRESULT {
    let mut point = POINT {
        x: signed_low_word(lparam),
        y: signed_high_word(lparam),
    };

    if unsafe { ScreenToClient(hwnd, &mut point) } == 0 {
        return HTCLIENT as LRESULT;
    }

    let mut client_rect: RECT = unsafe { mem::zeroed() };

    if unsafe { GetClientRect(hwnd, &mut client_rect) } == 0 {
        return HTCLIENT as LRESULT;
    }

    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
    let scale = dpi as f32 / 96.0;

    let logical_to_physical =
        |logical: f32| (logical * scale).round() as i32;

    let config = shared.config;

    let width = client_rect.right - client_rect.left;
    let height = client_rect.bottom - client_rect.top;

    let resize_border =
        logical_to_physical(config.resize_border).max(1);

    let title_bar_height =
        logical_to_physical(config.title_bar_height).max(1);

    let caption_button_width =
        logical_to_physical(config.caption_button_width).max(1);

    let system_menu_width =
        logical_to_physical(config.system_menu_width).max(0);

    let maximized = unsafe { IsZoomed(hwnd) != 0 };

    if config.resizable && !maximized {
        let left =
            point.x >= -resize_border && point.x < resize_border;

        let right = point.x >= width - resize_border
            && point.x < width + resize_border;

        let top =
            point.y >= -resize_border && point.y < resize_border;

        let bottom = point.y >= height - resize_border
            && point.y < height + resize_border;

        match (left, right, top, bottom) {
            (true, _, true, _) => {
                return HTTOPLEFT as LRESULT;
            }

            (_, true, true, _) => {
                return HTTOPRIGHT as LRESULT;
            }

            (true, _, _, true) => {
                return HTBOTTOMLEFT as LRESULT;
            }

            (_, true, _, true) => {
                return HTBOTTOMRIGHT as LRESULT;
            }

            (true, _, _, _) => {
                return HTLEFT as LRESULT;
            }

            (_, true, _, _) => {
                return HTRIGHT as LRESULT;
            }

            (_, _, true, _) => {
                return HTTOP as LRESULT;
            }

            (_, _, _, true) => {
                return HTBOTTOM as LRESULT;
            }

            _ => {}
        }
    }

    if point.y < 0 || point.y >= title_bar_height {
        return HTCLIENT as LRESULT;
    }

    let close_left = width - caption_button_width;
    let maximize_left = close_left - caption_button_width;
    let minimize_left = maximize_left - caption_button_width;

    if point.x >= close_left && point.x < width {
        return HTCLOSE as LRESULT;
    }

    if point.x >= maximize_left && point.x < close_left {
        return HTMAXBUTTON as LRESULT;
    }

    if point.x >= minimize_left && point.x < maximize_left {
        return HTMINBUTTON as LRESULT;
    }

    if system_menu_width > 0
        && point.x >= 0
        && point.x < system_menu_width
    {
        return HTSYSMENU as LRESULT;
    }

    HTCLIENT as LRESULT
}

fn execute_caption_command(
    hwnd: HWND,
    control: CaptionControl,
) {
    let command = match control {
        CaptionControl::Minimize => SC_MINIMIZE,

        CaptionControl::Maximize => {
            if unsafe { IsZoomed(hwnd) != 0 } {
                SC_RESTORE
            } else {
                SC_MAXIMIZE
            }
        }

        CaptionControl::Close => SC_CLOSE,
        CaptionControl::None => return,
    };

    unsafe {
        PostMessageW(
            hwnd,
            WM_SYSCOMMAND,
            command as WPARAM,
            0,
        );
    }
}

fn track_non_client_leave(hwnd: HWND) {
    let mut tracking = TRACKMOUSEEVENT {
        cbSize: mem::size_of::<TRACKMOUSEEVENT>() as u32,
        dwFlags: TME_LEAVE | TME_NONCLIENT,
        hwndTrack: hwnd,
        dwHoverTime: 0,
    };

    unsafe {
        TrackMouseEvent(&mut tracking);
    }
}

fn set_control(
    hwnd: HWND,
    atomic: &AtomicU8,
    control: CaptionControl,
) {
    let previous = atomic.swap(
        control as u8,
        Ordering::AcqRel,
    );

    if previous != control as u8 {
        redraw(hwnd);
    }
}

fn set_bool(
    hwnd: HWND,
    atomic: &AtomicBool,
    value: bool,
) {
    let previous = atomic.swap(
        value,
        Ordering::AcqRel,
    );

    if previous != value {
        redraw(hwnd);
    }
}

fn redraw(hwnd: HWND) {
    unsafe {
        InvalidateRect(hwnd, ptr::null(), 0);
    }
}

fn signed_low_word(value: LPARAM) -> i32 {
    let low = (value as u32 & 0xFFFF) as u16;
    low as i16 as i32
}

fn signed_high_word(value: LPARAM) -> i32 {
    let high = ((value as u32 >> 16) & 0xFFFF) as u16;
    high as i16 as i32
}

impl CaptionControl {
    fn from_hit_test(hit: LRESULT) -> Self {
        if hit == HTMINBUTTON as LRESULT {
            Self::Minimize
        } else if hit == HTMAXBUTTON as LRESULT {
            Self::Maximize
        } else if hit == HTCLOSE as LRESULT {
            Self::Close
        } else {
            Self::None
        }
    }
}
