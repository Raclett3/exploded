use super::cell::{Cell, CellType::*};
use yew::prelude::*;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub cell_size: f64,
}

pub struct Board {}

impl Component for Board {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let cell_size = ctx.props().cell_size;
        let width = (8. * cell_size).to_string();
        let height = (9. * cell_size).to_string();
        html! {
            <svg width={width} height={height}>
                <Cell x={0.} y={0.} size={cell_size} cell_type={Tile} />
                <Cell x={1.} y={0.} size={cell_size} cell_type={Bomb} />
                <Cell x={0.} y={1.} size={cell_size} cell_type={Bomb} />
                <Cell x={1.} y={1.} size={cell_size} cell_type={Tile} />
            </svg>
        }
    }
}
