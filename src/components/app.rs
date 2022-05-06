use super::board::Board;
use crate::game::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
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
    let cell_size = use_state(|| 0.);
    let game = use_reducer(Game::new);
    let board_ref = use_node_ref();

    let cloned_cell_size = cell_size.clone();
    let cloned_board_ref = board_ref.clone();
    let cloned_game = game.clone();

    use_effect_with_deps(
        move |_| {
            let window = web_sys::window().unwrap();
            let game = cloned_game;
            let width = window.inner_width().unwrap().as_f64().unwrap();
            let height = window.inner_height().unwrap().as_f64().unwrap();
            let (resized_width, resized_height) =
                fit_with_aspect_ratio(width - 20., height - 20., 8., 9.);
            let cell_size = resized_width as f64 / 8.;
            cloned_cell_size.set(cell_size);

            let board = cloned_board_ref.cast::<HtmlElement>().unwrap();
            let top = (height - resized_height) / 2.;
            let left = (width - resized_width) / 2.;
            board
                .set_attribute("style", &format!("top: {}px; left: {}px;", top, left))
                .unwrap();
            game.dispatch(GameAction::Feed);
            let cloned_game = game.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                event.prevent_default();
                let x = ((event.client_x() as f64 - left) / cell_size)
                    .max(0.)
                    .min(7.) as usize;
                let y = ((event.client_y() as f64 - top) / cell_size)
                    .max(0.)
                    .min(8.) as usize;
                cloned_game.dispatch(GameAction::Remove(x, y));
            }) as Box<dyn FnMut(_)>);
            board
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();

            let f = Rc::new(RefCell::new(None));
            let g = f.clone();

            let request_animation_frame = |window: &web_sys::Window, f: &Closure<dyn FnMut()>| {
                window
                    .request_animation_frame(f.as_ref().unchecked_ref())
                    .unwrap();
            };

            let cloned_window = window.clone();
            let cloned_game = game.clone();
            *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                cloned_game.dispatch(GameAction::Animate);
                request_animation_frame(&cloned_window, f.borrow().as_ref().unwrap());
            }) as Box<dyn FnMut()>));
            request_animation_frame(&window, g.borrow().as_ref().unwrap());

            || ()
        },
        (),
    );

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
        <div class="app" ref={board_ref}>
            <Board<WIDTH, HEIGHT> board={game.board.cells} floating_cells={floating_cells} score={game.score} is_game_over={game.is_over()} cell_size={*cell_size} />
        </div>
    }
}
