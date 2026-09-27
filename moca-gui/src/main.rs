mod gui;
mod tikz_export;
pub mod state_machine;
use gui::App;

pub fn main() -> iced::Result {
    iced::application("Moccacino", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .font(include_bytes!("../assets/fonts/Nunito-Regular.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/Nunito-SemiBold.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/Nunito-Bold.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf").as_slice())
        // Glyph fallback (ε, arrows, check marks) for text the UI fonts lack;
        // cosmic-text falls back to "DejaVu Sans" by name.
        .font(include_bytes!("../assets/fonts/DejaVuSans.ttf").as_slice())
        .default_font(gui::theme::UI)
        // Closing is intercepted so unsaved tabs can be confirmed first.
        .exit_on_close_request(false)
        .window_size(iced::Size::new(1280.0, 820.0))
        .antialiasing(true)
        .run_with(App::new)
}
