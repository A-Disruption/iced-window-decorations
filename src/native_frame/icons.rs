use iced::widget::svg;

use super::CaptionControl;

/// Lucide "minus".
const MINUS_SVG: &[u8] = br#"
<svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
>
    <path d="M5 12h14"/>
</svg>
"#;

/// Lucide "square".
const SQUARE_SVG: &[u8] = br#"
<svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2.5"
    stroke-linecap="round"
    stroke-linejoin="round"
>
    <rect
        width="18"
        height="18"
        x="3"
        y="3"
        rx="2"
    />
</svg>
"#;

/// Lucide "x".
const X_SVG: &[u8] = br#"
<svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
>
    <path d="M18 6 6 18"/>
    <path d="m6 6 12 12"/>
</svg>
"#;

/// Returns the embedded Lucide SVG for a caption control.
///
/// `CaptionControl::None` should never be rendered as a button, but returning
/// an empty SVG keeps this function total and avoids panicking.
pub(crate) fn handle(
    control: CaptionControl,
) -> svg::Handle {
    let bytes: &'static [u8] = match control {
        CaptionControl::Minimize => MINUS_SVG,
        CaptionControl::Maximize => SQUARE_SVG,
        CaptionControl::Close => X_SVG,
        CaptionControl::None => EMPTY_SVG,
    };

    svg::Handle::from_memory(bytes)
}

const EMPTY_SVG: &[u8] = br#"
<svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
>
</svg>
"#;