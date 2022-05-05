#[allow(dead_code)]
mod animation;
mod components;
mod game;

fn main() {
    yew::start_app::<components::app::App>();
}
