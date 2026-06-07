use crate::app::ui::agents::constants::{
    KEY_HINT_CLOSE, KEY_HINT_OPEN, STATUS_ICON_CANCEL, STATUS_ICON_CURSOR, STATUS_ICON_CURSOR_BLANK,
};

pub const ICON_ACTION_RETRY: &str = "\u{F0450}";
pub const ICON_ACTION_EXPORT_MD: &str = "\u{F0488}";
pub const ICON_ACTION_EXPORT_JSON: &str = "\u{F0626}";
pub const ICON_ACTION_FILTER: &str = "\u{F0349}";
pub const ICON_ACTION_COLLAPSE: &str = "\u{F0140}";
pub const ICON_ACTION_EXPAND: &str = "\u{F0142}";
pub const ICON_PROVIDER: &str = "\u{F0210}";
pub const ICON_LAUNCH: &str = "\u{F06A9}";
pub const ICON_PARALLEL: &str = "\u{F04C5}";
pub const ICON_CLEAR: &str = "\u{F0A48}";
pub const ICON_AGENTS_TITLE: &str = "\u{F1DA}";
pub const ICON_DETAIL_TITLE: &str = "\u{F0218}";
pub const ICON_ACTIONS_TITLE: &str = "\u{F0B12}";

pub const STATUS_IDLE: &str = "⏺";
pub const STATUS_QUEUED: &str = "⏳";
pub const STATUS_DONE: &str = "✔";
pub const STATUS_FAILED: &str = "✘";

pub fn cursor_for_frame(frame_count: u64) -> &'static str {
    if frame_count.is_multiple_of(2) {
        STATUS_ICON_CURSOR
    } else {
        STATUS_ICON_CURSOR_BLANK
    }
}

pub fn action_cancel_icon() -> &'static str {
    STATUS_ICON_CANCEL
}

pub fn key_hint(key: &str) -> String {
    format!("{}{}{}", KEY_HINT_OPEN, key, KEY_HINT_CLOSE)
}
