use iced::Size;

pub(crate) const DEFAULT_WINDOW: Size = Size::new(1280.0, 820.0);

pub(crate) const BRAND_TEXT_MIN_WIDTH: f32 = 1000.0;
pub(crate) const STACKED_HEADER_MAX_WIDTH: f32 = 800.0;
pub(crate) const DOCK_COMPACT_MAX_WIDTH: f32 = 1150.0;
pub(crate) const PALETTE_ICONS_ONLY_MAX_WIDTH: f32 = 760.0;
pub(crate) const STATUS_HINT_MIN_WIDTH: f32 = 1000.0;
pub(crate) const STATUS_COUNTS_MIN_WIDTH: f32 = 700.0;
pub(crate) const SHORTCUTS_TWO_COLUMN_MIN_WIDTH: f32 = 820.0;

pub(crate) const HEADER_HEIGHT: f32 = 52.0;
pub(crate) const TAB_ROW_HEIGHT: f32 = 44.0;
/// Fixed so the run dock does not shift when the status chips come and go.
pub(crate) const STATUS_BAR_HEIGHT: f32 = 32.0;
/// Window height taken by the header, run dock and status bar before the canvas reports its size.
pub(crate) const CANVAS_CHROME_RESERVE: f32 = 260.0;
/// Window height kept free around the shortcut sheet so its card never touches the window edge.
pub(crate) const DIALOG_CHROME_RESERVE: f32 = 220.0;
