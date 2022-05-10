use super::game::Game;
use crate::game::{HEIGHT, WIDTH};
use yew::prelude::*;

fn fit_with_aspect_ratio(
    width: f64,
    height: f64,
    aspect_width: f64,
    aspect_height: f64,
) -> (f64, f64) {
    if width * aspect_height > height * aspect_width {
        (height * aspect_width / aspect_height, height)
    } else {
        (width, width * aspect_height / aspect_width)
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let window = web_sys::window().unwrap();
    let width = window.inner_width().unwrap().as_f64().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap();
    let (resized_width, resized_height) =
        fit_with_aspect_ratio(width - 20., height - 20., WIDTH as f64, HEIGHT as f64);

    let top = (height - resized_height) / 2.;
    let left = (width - resized_width) / 2.;
    let cell_size = resized_width as f64 / WIDTH as f64;

    html! {
        <Game cell_size={cell_size} left={left} top={top} />
    }
}
