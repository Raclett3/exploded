use super::game::Game;
use super::game_hard::GameHard;
use super::game_online::GameOnline;
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

#[derive(Clone, Copy)]
enum GameMode {
    Normal,
    Hard,
    Online,
}

#[function_component(App)]
pub fn app() -> Html {
    let difficulty: UseStateHandle<Option<GameMode>> = use_state(|| None);
    let window = web_sys::window().unwrap();
    let width = window.inner_width().unwrap().as_f64().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap();
    let (resized_width, _) =
        fit_with_aspect_ratio(width - 20., height - 20., WIDTH as f64, HEIGHT as f64);

    let cell_size = resized_width as f64 / WIDTH as f64;

    let cloned_difficulty = difficulty.clone();
    let select_difficulty = |diff: GameMode| {
        let cloned_difficulty = cloned_difficulty.clone();
        Callback::from(move |event: web_sys::MouseEvent| {
            event.prevent_default();
            cloned_difficulty.set(Some(diff));
        })
    };

    match *difficulty {
        None => html! {
            <div class="app">
                <h1>{"Exploded"}</h1>
                <h2>{"Select a game mode"}</h2>
                <h3><a href="#" onclick={select_difficulty(GameMode::Normal)}>{"NORMAL"}</a></h3>
                <h3><a href="#" onclick={select_difficulty(GameMode::Hard)}>{"MASTER"}</a></h3>
                <h3><a href="#" onclick={select_difficulty(GameMode::Online)}>{"ONLINE"}</a></h3>
            </div>
        },
        Some(GameMode::Normal) => html! {
            <Game cell_size={cell_size} />
        },
        Some(GameMode::Hard) => html! {
            <GameHard cell_size={cell_size} />
        },
        Some(GameMode::Online) => html! {
            <GameOnline cell_size={cell_size} />
        },
    }
}
