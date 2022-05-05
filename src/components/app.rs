use super::board::Board;
use super::cell::CellType as ComponentCellType;
use crate::animation::*;
use crate::game::{Cell, CellType, Game};
use rand::prelude::*;
use std::cell::RefCell;
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

struct FloatAnimator<T> {
    begin_at: f64,
    elapsed_frames: usize,
    animator: Animator<T>,
}

impl<T> FloatAnimator<T> {
    fn new(animations: Vec<Box<dyn Animation<T>>>) -> Self {
        let now = js_sys::Date::now();
        FloatAnimator {
            begin_at: now,
            elapsed_frames: 0,
            animator: Animator::new(animations),
        }
    }

    fn animate(&mut self) {
        let now = js_sys::Date::now();
        let elapsed = now - self.begin_at;
        let frames = (elapsed / 60.0).floor() as usize;
        let frame_delta = frames - self.elapsed_frames;

        if frame_delta > 0 {
            self.animator.advance_frames(frame_delta);
        }
    }

    fn frame(&self) -> Vec<T> {
        self.animator.current_frame()
    }

    fn is_over(&self) -> bool {
        self.animator.is_over()
    }
}

#[derive(Clone, PartialEq)]
pub struct FloatingCell {
    pub x: f64,
    pub y: f64,
    pub cell_type: ComponentCellType,
}

struct CellAnimator {
    x: f64,
    y_from: f64,
    y_to: f64,
    duration: usize,
    elapsed: usize,
    cell_type: ComponentCellType,
}

impl CellAnimator {
    fn new(x: f64, y_from: f64, y_to: f64, duration: usize, cell_type: ComponentCellType) -> Self {
        CellAnimator {
            x,
            y_from,
            y_to,
            duration,
            cell_type,
            elapsed: 0,
        }
    }
}

impl Animation<FloatingCell> for CellAnimator {
    fn advance_frames(&mut self, frames: usize) {
        self.elapsed += frames;
    }

    fn current_frame(&self) -> FloatingCell {
        let relative_time = self.elapsed.min(self.duration) as f64 / self.duration as f64;
        let y = (self.y_from * (1. - relative_time)) + (self.y_to * relative_time);
        FloatingCell {
            x: self.x,
            y,
            cell_type: self.cell_type,
        }
    }

    fn is_over(&self) -> bool {
        self.duration <= self.elapsed
    }
}

#[derive(Clone)]
struct ReducibleGame {
    game: Game<WIDTH, HEIGHT>,
    generator: BombGenerator,
    animator: Option<Rc<RefCell<FloatAnimator<FloatingCell>>>>,
}

pub enum GameAction {
    Feed,
    Remove(usize, usize),
    Animate,
}

impl ReducibleGame {
    fn new() -> Self {
        ReducibleGame {
            game: Game::new(),
            generator: BombGenerator::new(),
            animator: None,
        }
    }

    fn feed(&mut self) {
        let bombs = self.generator.next();
        let mut row = [CellType::Tile; WIDTH];
        row[bombs.0] = CellType::Bomb;
        row[bombs.1] = CellType::Bomb;
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
                    let dists = self_cloned.game.apply_gravity();
                    let cells = self_cloned
                        .game
                        .board
                        .iter()
                        .enumerate()
                        .flat_map(|(x, col)| {
                            let dists = dists.clone();
                            col.iter().enumerate().flat_map(move |(y, cell)| {
                                cell.map(|cell| {
                                    let Cell { id, cell_type } = cell;
                                    let cell_type = match cell_type {
                                        CellType::Bomb => ComponentCellType::Bomb,
                                        CellType::Tile => ComponentCellType::Tile,
                                    };
                                    let dist = dists.get(&id).cloned().unwrap_or(0);
                                    Box::new(CellAnimator::new(
                                        x as f64,
                                        (y - dist) as f64,
                                        y as f64,
                                        dist * 6 + 1,
                                        cell_type,
                                    ))
                                        as Box<dyn Animation<FloatingCell>>
                                })
                            })
                        })
                        .collect();
                    self_cloned.animator = Some(Rc::new(RefCell::new(FloatAnimator::new(cells))));
                    self_cloned.feed();
                }
            }
            GameAction::Feed => {
                self_cloned.feed();
            }
            GameAction::Animate => {
                if let Some(animator) = &self.animator {
                    animator.borrow_mut().animate();
                }
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
            <Board<WIDTH, HEIGHT> board={game.game.board} floating_cells={floating_cells} cell_size={*cell_size} />
        </div>
    }
}
