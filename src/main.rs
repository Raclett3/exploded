#[allow(dead_code)]
mod animation;
mod board;
mod components;

fn main() {
    yew::start_app::<components::app::App>();
}
