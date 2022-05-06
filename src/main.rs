#[allow(dead_code)]
mod animation;
mod board;
mod components;
mod game;

fn main() {
    yew::start_app::<components::app::App>();
}
