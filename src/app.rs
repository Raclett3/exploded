use crate::board::Board;
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
    link: ComponentLink<Self>,
    board: NodeRef,
    cell_size: f64,
}

pub enum Msg {
    Resize(f64),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            board: NodeRef::default(),
            cell_size: 100.,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Resize(cell_size) => {
                let should_render = self.cell_size != cell_size;
                self.cell_size = cell_size;
                should_render
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn rendered(&mut self, _first_render: bool) {
        let window = web_sys::window().unwrap();
        let width = window.inner_width().unwrap().as_f64().unwrap() - 20.;
        let height = window.inner_height().unwrap().as_f64().unwrap() - 20.;
        let (resized_width, _) = fit_with_aspect_ratio(width, height, 8., 9.);
        let cell_size = resized_width as f64 / 8.;

        let callback = self.link.callback(Msg::Resize);
        callback.emit(cell_size);
    }

    fn view(&self) -> Html {
        html! {
            <div class="app" ref=self.board.clone()>
                <Board cell_size={self.cell_size} />
            </div>
        }
    }
}
