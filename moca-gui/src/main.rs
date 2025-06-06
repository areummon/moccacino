mod gui;
pub mod state_machine;
use gui::App;

pub fn main() -> iced::Result {
    iced::application("State Machine Editor", App::update, App::view)
        .subscription(App::subscription)
        .run_with(App::new)
}