//! Window icon from embedded `src/assets/markdown-logo.svg`.
//!
//! Renders into a square pixmap (centered), tints opaque areas white for dark title bars,
//! and uses straight RGBA for the windowing stack.

use iced::window::Icon;
use resvg::tiny_skia::Transform;
use resvg::usvg;

const MARKDOWN_LOGO_SVG: &[u8] = include_bytes!("assets/markdown-logo.svg");

/// Minimum edge so small toolbar icons still have enough pixels after OS scaling.
const MIN_SQUARE: u32 = 64;

pub fn from_embedded_logo_svg() -> Option<Icon> {
    let tree = usvg::Tree::from_data(MARKDOWN_LOGO_SVG, &usvg::Options::default()).ok()?;
    let size = tree.size();
    let w = size.width().ceil().max(1.0) as u32;
    let h = size.height().ceil().max(1.0) as u32;
    let square = w.max(h).max(MIN_SQUARE);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(square, square)?;
    let tx = ((square - w) as f32 / 2.0).floor();
    let ty = ((square - h) as f32 / 2.0).floor();
    resvg::render(
        &tree,
        Transform::from_translate(tx, ty),
        &mut pixmap.as_mut(),
    );
    let mut rgba = pixmap.data().to_vec();
    unpremultiply_rgba(&mut rgba);
    tint_visible_white(&mut rgba);
    iced::window::icon::from_rgba(rgba, square, square).ok()
}

fn unpremultiply_rgba(buf: &mut [u8]) {
    for px in buf.chunks_exact_mut(4) {
        let a = px[3] as u32;
        if a == 0 {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
            continue;
        }
        px[0] = ((px[0] as u32 * 255 + a / 2) / a).min(255) as u8;
        px[1] = ((px[1] as u32 * 255 + a / 2) / a).min(255) as u8;
        px[2] = ((px[2] as u32 * 255 + a / 2) / a).min(255) as u8;
    }
}

/// Title-bar / taskbar icons on dark backgrounds read best as white; keep alpha for AA edges.
fn tint_visible_white(buf: &mut [u8]) {
    for px in buf.chunks_exact_mut(4) {
        if px[3] > 0 {
            px[0] = 255;
            px[1] = 255;
            px[2] = 255;
        }
    }
}
