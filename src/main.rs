mod components;
#[allow(dead_code)]
mod game;

fn main() {
    yew::start_app::<components::app::App>();
}
