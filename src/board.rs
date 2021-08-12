use crate::cell::{Cell, CellType::*};
use yew::prelude::*;

pub struct Board;

impl Component for Board {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <svg width="800" height="900">
                <Cell x={0} y={0} cell_type={Tile} />
                <Cell x={100} y={0} cell_type={Bomb} />
                <Cell x={0} y={100} cell_type={Bomb} />
                <Cell x={100} y={100} cell_type={Tile} />
            </svg>
        }
    }
}
