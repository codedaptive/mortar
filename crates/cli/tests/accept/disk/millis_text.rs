//! Shows a wall time on a check line.

use std::time::Duration;

/// "<milliseconds to one decimal> ms".
pub fn millis_text(wall: Duration) -> String {
    format!("{:.1} ms", wall.as_secs_f64() * 1000.0)
}
