mod gui;
mod platform;
mod tikz_export;
pub mod state_machine;
use gui::App;

pub fn main() -> iced::Result {
    platform::init();
    iced::application("Moccacino", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .font(include_bytes!("../assets/fonts/Nunito-Regular.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/Nunito-SemiBold.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/Nunito-Bold.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/DejaVuSans.ttf").as_slice())
        .default_font(gui::theme::UI)
        .exit_on_close_request(false)
        .window_size(iced::Size::new(1280.0, 820.0))
        .antialiasing(!platform::IS_WEB)
        .run_with(App::new)
}
