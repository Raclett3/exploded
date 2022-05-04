use super::board::Board;
use crate::game::{CellType, Game};
use rand::prelude::*;
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

pub struct App {
    board: NodeRef,
    cell_size: f64,
    game: Game<WIDTH, HEIGHT>,
    generator: BombGenerator,
}

pub enum Msg {
    Resize(f64),
    Click((usize, usize)),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let mut game = Game::<WIDTH, HEIGHT>::new();
        let mut generator = BombGenerator::new();
        let bombs = generator.next();
        let mut row = [CellType::Tile; WIDTH];
        row[bombs.0] = CellType::Bomb;
        row[bombs.1] = CellType::Bomb;
        game.feed(&row);

        Self {
            board: NodeRef::default(),
            cell_size: 100.,
            game,
            generator,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Resize(cell_size) => {
                #[allow(clippy::float_cmp)]
                let should_render = self.cell_size != cell_size;
                self.cell_size = cell_size;
                should_render
            }
            Msg::Click((x, y)) => {
                let ids = self.game.remove(x, y);

                if !ids.is_empty() {
                    let bombs = self.generator.next();
                    let mut row = [CellType::Tile; WIDTH];
                    row[bombs.0] = CellType::Bomb;
                    row[bombs.1] = CellType::Bomb;
                    self.game.apply_gravity();
                    self.game.feed(&row);
                    true
                } else {
                    false
                }
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let window = web_sys::window().unwrap();
        let width = window.inner_width().unwrap().as_f64().unwrap();
        let height = window.inner_height().unwrap().as_f64().unwrap();
        let (resized_width, resized_height) =
            fit_with_aspect_ratio(width - 20., height - 20., 8., 9.);
        let cell_size = resized_width as f64 / 8.;

        let resize_callback = ctx.link().callback(Msg::Resize);
        resize_callback.emit(cell_size);

        let board = self.board.cast::<HtmlElement>().unwrap();
        let top = (height - resized_height) / 2.;
        let left = (width - resized_width) / 2.;
        board
            .set_attribute("style", &format!("top: {}px; left: {}px;", top, left))
            .unwrap();

        if first_render {
            let click_callback = ctx.link().callback(Msg::Click);
            let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                let x = ((event.client_x() as f64 - left) / cell_size)
                    .max(0.)
                    .min(7.) as usize;
                let y = ((event.client_y() as f64 - top) / cell_size)
                    .max(0.)
                    .min(8.) as usize;
                click_callback.emit((x, y));
            }) as Box<dyn FnMut(_)>);
            board
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="app" ref={self.board.clone()}>
                <Board<WIDTH, HEIGHT> board={self.game.board} cell_size={self.cell_size} />
            </div>
        }
    }
}
