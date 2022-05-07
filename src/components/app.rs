use super::board::Board;
use crate::game::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
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

fn raf_loop(mut func: impl FnMut() + 'static) {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let request_animation_frame = |window: &web_sys::Window, f: &Closure<dyn FnMut()>| {
        window
            .request_animation_frame(f.as_ref().unchecked_ref())
            .unwrap();
    };

    let window = web_sys::window().unwrap();
    let cloned_window = window.clone();
    *f.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        func();
        request_animation_frame(&cloned_window, g.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    request_animation_frame(&window, f.borrow().as_ref().unwrap());
}

#[function_component(App)]
pub fn app() -> Html {
    let game = use_reducer(Game::new);

    let cloned_game = game.clone();

    use_effect_with_deps(
        move |_| {
            game.dispatch(GameAction::Feed);
            raf_loop(move || game.dispatch(GameAction::Animate));
            || ()
        },
        (),
    );

    let window = web_sys::window().unwrap();
    let game = cloned_game;
    let width = window.inner_width().unwrap().as_f64().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap();
    let (resized_width, resized_height) = fit_with_aspect_ratio(width - 20., height - 20., WIDTH as f64, HEIGHT as f64);
    let cell_size = resized_width as f64 / WIDTH as f64;

    let top = (height - resized_height) / 2.;
    let left = (width - resized_width) / 2.;

    let cloned_game = game.clone();
    let onmousedown = Callback::from(move |event: web_sys::MouseEvent| {
        event.prevent_default();
        let x = ((event.client_x() as f64 - left) / cell_size)
            .max(0.)
            .min(WIDTH as f64 - 1.) as usize;
        let y = ((event.client_y() as f64 - top) / cell_size)
            .max(0.)
            .min(HEIGHT as f64 - 1.) as usize;
        cloned_game.dispatch(GameAction::Remove(x, y));
    });

    let floating_cells = if let Some(animator) = &game.animator {
        let animator = animator.borrow();

        if !animator.is_over() {
            Some(animator.frame())
        } else {
            None
        }
    } else {
        None
    };

    html! {
        <div class="app" onmousedown={onmousedown} style={format!("top: {}px; left: {}px;", top, left)}>
            <Board<WIDTH, HEIGHT> board={game.board.cells} floating_cells={floating_cells} score={game.score} is_game_over={game.is_over()} cell_size={cell_size} />
        </div>
    }
}
