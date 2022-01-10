use super::board::Board;
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

pub struct App {
    board: NodeRef,
    cell_size: f64,
}

pub enum Msg {
    Resize(f64),
    Click((usize, usize)),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            board: NodeRef::default(),
            cell_size: 100.,
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
                web_sys::console::log_2(&JsValue::from_f64(x as f64), &JsValue::from_f64(y as f64));
                false
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
                <Board cell_size={self.cell_size} />
            </div>
        }
    }
}
