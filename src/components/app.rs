use super::board::Board;
use crate::game::{CellType, Game};
use rand::prelude::*;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;

const WIDTH: usize = 8;
const HEIGHT: usize = 9;

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

#[derive(Clone)]
struct BombGenerator {
    shuffled: Vec<(usize, usize)>,
    rng: StdRng,
}

impl BombGenerator {
    fn new() -> Self {
        let random = js_sys::Math::random();
        let rng = StdRng::seed_from_u64(u64::from_be_bytes(random.to_be_bytes()));
        let mut generator = BombGenerator {
            shuffled: Vec::new(),
            rng,
        };
        generator.shuffle();
        generator
    }

    fn shuffle(&mut self) {
        let mut shuffled = (0..WIDTH)
            .flat_map(|first| (first + 1..WIDTH).map(move |second| (first, second)))
            .collect::<Vec<_>>();
        shuffled.shuffle(&mut self.rng);
        self.shuffled = shuffled;
    }

    fn next(&mut self) -> (usize, usize) {
        self.shuffled.pop().unwrap_or_else(|| {
            self.shuffle();
            self.next()
        })
    }
}

#[derive(Clone)]
struct ReducibleGame {
    game: Game<WIDTH, HEIGHT>,
    generator: BombGenerator,
}

pub enum GameAction {
    Feed,
    Remove(usize, usize),
}

impl ReducibleGame {
    fn new() -> Self {
        ReducibleGame {
            game: Game::new(),
            generator: BombGenerator::new(),
        }
    }

    fn feed(&mut self) {
        let bombs = self.generator.next();
        let mut row = [CellType::Tile; WIDTH];
        row[bombs.0] = CellType::Bomb;
        row[bombs.1] = CellType::Bomb;
        self.game.apply_gravity();
        self.game.feed(&row);
    }
}

impl Reducible for ReducibleGame {
    type Action = GameAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut self_cloned = (*self).clone();

        match action {
            GameAction::Remove(x, y) => {
                let ids = self_cloned.game.remove(x, y);
                if !ids.is_empty() {
                    self_cloned.feed();
                }
            }
            GameAction::Feed => {
                self_cloned.feed();
            }
        }

        self_cloned.into()
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let cell_size = use_state(|| 0.);
    let game = use_reducer(ReducibleGame::new);
    let board_ref = use_node_ref();

    let cloned_cell_size = cell_size.clone();
    let cloned_board_ref = board_ref.clone();
    let cloned_game = game.clone();

    use_effect_with_deps(
        move |_| {
            let window = web_sys::window().unwrap();
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
            cloned_game.dispatch(GameAction::Feed);
            let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
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

            || ()
        },
        (),
    );

    html! {
        <div class="app" ref={board_ref}>
            <Board<WIDTH, HEIGHT> board={game.game.board} cell_size={*cell_size} />
        </div>
    }
}
