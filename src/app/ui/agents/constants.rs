pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub const RUNNING_PHASE_INTERVAL_FRAMES: u64 = 8;
pub const RUNNING_PHASE_COUNT: u64 = 4;
pub const RUNNING_PROGRESS_MIN_INITIAL: usize = 5;
pub const RUNNING_PROGRESS_MIN_WITH_MSG: usize = 20;
pub const RUNNING_PROGRESS_MAX: usize = 95;
pub const RUNNING_PROGRESS_DIVISOR: usize = 10;
pub const RUNNING_PROGRESS_TIME_DIVISOR: u64 = 4;
pub const RUNNING_ETA_BASE_SECS: usize = 180;
pub const STATUS_ICON_CANCEL: &str = "⏹";
pub const STATUS_ICON_CURSOR: &str = "█";
pub const STATUS_ICON_CURSOR_BLANK: &str = " ";

pub const MD_TRUNCATION_SUFFIX: &str = "...";
pub const MD_BOLD_DELIMITER: &str = "**";
pub const MD_CODE_DELIMITER: char = '`';
pub const MD_BULLET_CHAR: &str = " • ";
pub const MD_HR_CHAR: &str = "─";
pub const MD_HR_MAX_WIDTH: usize = 60;
pub const MD_TABLE_SEP_MAX_WIDTH: usize = 40;
pub const MD_TABLE_INNER_SEP: &str = " ┃ ";
pub const MD_CODE_BLOCK_OPEN: &str = "┌─ code ───────────────────────────";
pub const MD_CODE_BLOCK_CLOSE: &str = "└───────────────────────────────────";
pub const MD_CODE_LINE_PREFIX: &str = "│ ";
pub const MD_INLINE_CODE_FALLBACK: &str = "`";
pub const MD_BOLD_FALLBACK: &str = "**";
pub const MD_H1_PREFIX: &str = "# ";
pub const MD_H2_PREFIX: &str = "## ";
pub const MD_H3_PREFIX: &str = "### ";
pub const MD_HR_LINE: &str = "---";
pub const MD_HR_STAR: &str = "***";
pub const MD_HR_UNDER: &str = "___";
pub const MD_HR_WIDTH_REDUCTION: usize = 4;
pub const MD_BULLET_DASH: &str = "- ";
pub const MD_BULLET_STAR: &str = "* ";
pub const MD_INLINE_PADDING: usize = 2;
pub const MD_INDENT_PADDING: usize = 4;
pub const MD_NUMERIC_PADDING: usize = 6;
pub const MD_COLLAPSE_MARKER: &str = "  [+]";
pub const MD_SCROLL_DOWN_INDICATOR_PREFIX: &str = "▼ (+";
pub const MD_SCROLL_DOWN_INDICATOR_SUFFIX: &str = " lines)";

pub const FAILED_STATUS_MAX_CHARS: usize = 45;

pub const ACTION_BTN_PADDING: &str = " ";
pub const SELECTED_ROW_INDICATOR: &str = " ▎";
pub const UNSELECTED_ROW_INDICATOR: &str = "  ";
pub const SCROLL_INDICATOR_UP: &str = "↑";
pub const SCROLL_INDICATOR_DOWN: &str = "↓";
pub const SCROLL_INDICATOR_BOTH: &str = "↕";

pub const PROCESS_SELECTOR_CHECKED: &str = "[X]";
pub const PROCESS_SELECTOR_UNCHECKED: &str = "[ ]";
pub const NETWORK_SELECTOR_CHECKED: &str = "[X]";
pub const NETWORK_SELECTOR_UNCHECKED: &str = "[ ]";
pub const SELECTOR_HINT_TEXT: &str = "  [Space] \u{2713} / \u{2717}   \u{2191}\u{2193} ";
pub const SELECTOR_TOGGLE_HINT: &str = "Space: toggle";

pub const KEY_HINT_OPEN: &str = "[ ";
pub const KEY_HINT_CLOSE: &str = " ]";

pub const PROVIDER_MODAL_WIDTH_DIVISOR: u16 = 5;
pub const PROVIDER_MODAL_WIDTH_NUMERATOR: u16 = 3;
pub const PROVIDER_MODAL_WIDTH_MIN: u16 = 56;
pub const PROVIDER_MODAL_HEIGHT_DIVISOR: u16 = 7;
pub const PROVIDER_MODAL_HEIGHT_NUMERATOR: u16 = 3;
pub const PROVIDER_MODAL_HEIGHT_MIN: u16 = 20;

pub const PROVIDER_MODAL_FOCUS_PROVIDER: usize = 0;
pub const PROVIDER_MODAL_FOCUS_URL: usize = 1;
pub const PROVIDER_MODAL_FOCUS_MODEL_INPUT: usize = 2;
pub const PROVIDER_MODAL_FOCUS_API_KEY: usize = 3;
pub const PROVIDER_MODAL_FOCUS_MODELS: usize = 4;
pub const PROVIDER_MODAL_FOCUS_FETCH: usize = 5;
pub const PROVIDER_MODAL_FOCUS_SAVE: usize = 6;
pub const PROVIDER_MODAL_FOCUS_CANCEL: usize = 7;
pub const PROVIDER_MODAL_MODEL_LIST_RESERVED_ROWS: usize = 17;
pub const PROVIDER_MODAL_API_KEY_MASK_CHAR: &str = "•";
pub const PROVIDER_MODAL_API_KEY_MAX_MASK: usize = 24;

pub const AGENT_TYPE_SELECTOR_WIDTH: u16 = 76;
pub const AGENT_TYPE_SELECTOR_HEIGHT: u16 = 18;

pub const PROCESS_SELECTOR_HEIGHT_DIVISOR: u16 = 5;
pub const PROCESS_SELECTOR_WIDTH_DIVISOR: u16 = 5;
pub const PROCESS_SELECTOR_RESERVED_ROWS: usize = 8;
pub const NETWORK_SELECTOR_HEIGHT_DIVISOR: u16 = 5;
pub const NETWORK_SELECTOR_WIDTH_DIVISOR: u16 = 5;
pub const NETWORK_SELECTOR_RESERVED_ROWS: usize = 8;

pub const FRAME_COUNT_MS: u64 = 250;
pub const ELAPSED_TIME_DECIMAL_DIVISOR: u64 = 100;

pub const MIN_WRAP_WIDTH: usize = 10;

pub const STATUS_BADGE_PADDING: &str = " ";

pub const PROGRESS_BAR_FULL: &str = "█";
pub const PROGRESS_BAR_EMPTY: &str = "░";
pub const PROGRESS_BAR_TOTAL: usize = 10;

pub const KEY_PROVIDER_CYCLE: &str = "C";
pub const KEY_LAUNCH: &str = "A";
pub const KEY_CANCEL: &str = "S";
pub const KEY_RETRY: &str = "R";
pub const KEY_EXPORT_MD: &str = "E";
pub const KEY_EXPORT_JSON: &str = "J";
pub const KEY_FILTER: &str = "F";
pub const KEY_COLLAPSE: &str = "Z";
pub const KEY_PARALLEL: &str = "+/-";
pub const KEY_CLEAR: &str = "X";
