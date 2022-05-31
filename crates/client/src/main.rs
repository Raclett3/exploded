mod animation;
mod components;
mod game;
mod websocket;

fn main() {
    yew::start_app::<components::app::App>();
}
