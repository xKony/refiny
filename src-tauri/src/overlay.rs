//! The waiting pill: where it lands, and how it stays out of the way.
//!
//! The overlay window is transparent, undecorated, and deliberately larger than
//! the pill by `SHADOW_MARGIN` on every side, so the pill's shadow has room to
//! fall. The placement maths is a pure function on physical pixels, so it is
//! tested without a desktop session.

use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow};

/// Distance from the pointer to the nearest visible edge of the pill.
const POINTER_GAP: i32 = 16;

/// Transparent margin between the window edge and the pill.
const SHADOW_MARGIN: i32 = 8;

/// A monitor's usable area in physical pixels. The taskbar is excluded, which
/// is the point: a pill clamped to the screen would still sit under it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkArea {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Picks the window origin for a pill that belongs just past the pointer.
///
/// Preference is below-right, because that is where the eye is not looking and
/// where the pointer is not sitting. When that side runs out of room the pill
/// flips to the other, and only when neither side fits does it clamp, which is
/// the one case where it can cover the pointer. That case is a work area
/// smaller than the window, which no real monitor has.
pub fn place(pointer: (i32, i32), area: WorkArea, size: (u32, u32)) -> (i32, i32) {
    let width = size.0 as i32;
    let height = size.1 as i32;
    let right = area.x + area.width as i32;
    let bottom = area.y + area.height as i32;

    // The window origin sits one margin before the pill, so the pill's own edge
    // keeps the full gap from the pointer.
    let after_x = pointer.0 + POINTER_GAP - SHADOW_MARGIN;
    let after_y = pointer.1 + POINTER_GAP - SHADOW_MARGIN;

    let x = if after_x + width <= right {
        after_x
    } else {
        pointer.0 - POINTER_GAP + SHADOW_MARGIN - width
    };
    let y = if after_y + height <= bottom {
        after_y
    } else {
        pointer.1 - POINTER_GAP + SHADOW_MARGIN - height
    };

    (
        x.clamp(area.x, (right - width).max(area.x)),
        y.clamp(area.y, (bottom - height).max(area.y)),
    )
}

/// Moves the pill next to the pointer, then shows it.
///
/// Positioning comes first on purpose: showing the window and then moving it
/// makes it flash at wherever it was last seen. When the pointer cannot be
/// read, the window keeps its last position and the action still reports its
/// state - a pill out of place is better than a silent failure.
pub fn show(app: &AppHandle) {
    let Some(window) = app.get_webview_window("overlay") else {
        return;
    };
    let _ = place_at_pointer(app, &window);
    let _ = window.show();
}

fn place_at_pointer(app: &AppHandle, window: &WebviewWindow) -> tauri::Result<()> {
    let pointer = app.cursor_position()?;
    let monitor = app
        .monitor_from_point(pointer.x, pointer.y)?
        .or(window.current_monitor()?);
    let Some(monitor) = monitor else {
        return Ok(());
    };

    let work = monitor.work_area();
    let size = window.outer_size()?;
    let (x, y) = place(
        (pointer.x.round() as i32, pointer.y.round() as i32),
        WorkArea {
            x: work.position.x,
            y: work.position.y,
            width: work.size.width,
            height: work.size.height,
        },
        (size.width, size.height),
    );

    window.set_position(PhysicalPosition::new(x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: WorkArea = WorkArea {
        x: 0,
        y: 0,
        width: 1920,
        height: 1040,
    };
    const PILL: (u32, u32) = (216, 60);

    #[test]
    fn the_visible_pill_keeps_the_pointer_gap() {
        let (x, y) = place((100, 100), SCREEN, PILL);
        assert_eq!(x + SHADOW_MARGIN - 100, POINTER_GAP);
        assert_eq!(y + SHADOW_MARGIN - 100, POINTER_GAP);
    }

    #[test]
    fn the_default_is_below_and_to_the_right() {
        assert_eq!(place((100, 100), SCREEN, PILL), (108, 108));
    }

    #[test]
    fn flips_to_the_left_when_the_right_side_runs_out() {
        let (x, _) = place((1900, 100), SCREEN, PILL);
        // The pill's right edge lands one gap left of the pointer.
        assert_eq!(x + PILL.0 as i32 - SHADOW_MARGIN, 1900 - POINTER_GAP);
    }

    #[test]
    fn flips_above_when_the_bottom_runs_out() {
        let (_, y) = place((100, 1030), SCREEN, PILL);
        assert_eq!(y + PILL.1 as i32 - SHADOW_MARGIN, 1030 - POINTER_GAP);
    }

    #[test]
    fn both_sides_flip_in_the_bottom_right_corner() {
        let (x, y) = place((1910, 1035), SCREEN, PILL);
        assert_eq!(x + PILL.0 as i32 - SHADOW_MARGIN, 1910 - POINTER_GAP);
        assert_eq!(y + PILL.1 as i32 - SHADOW_MARGIN, 1035 - POINTER_GAP);
    }

    #[test]
    fn a_work_area_smaller_than_the_window_clamps_to_its_origin() {
        let tiny = WorkArea {
            x: 40,
            y: 30,
            width: 120,
            height: 40,
        };
        assert_eq!(place((60, 20), tiny, PILL), (40, 30));
    }

    #[test]
    fn follows_the_pointer_onto_a_monitor_left_of_the_primary() {
        let left = WorkArea {
            x: -1920,
            y: 0,
            width: 1920,
            height: 1040,
        };
        assert_eq!(place((-1900, 100), left, PILL), (-1892, 108));
    }
}
