use crate::cell::{Cell, CellType::*};
use yew::prelude::*;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub cell_size: f64,
}

pub struct Board {
    props: Props,
}

impl Component for Board {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if props == self.props {
            return false;
        }
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let cell_size = self.props.cell_size;
        html! {
            <svg width={8. * cell_size} height={9. * cell_size}>
                <Cell x={0.} y={0.} size={cell_size} cell_type={Tile} />
                <Cell x={1.} y={0.} size={cell_size} cell_type={Bomb} />
                <Cell x={0.} y={1.} size={cell_size} cell_type={Bomb} />
                <Cell x={1.} y={1.} size={cell_size} cell_type={Tile} />
            </svg>
        }
    }
}
