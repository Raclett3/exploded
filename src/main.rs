const SIZE: isize = 100;

use yew::prelude::*;

#[derive(Clone, Copy)]
enum CellType {
    Tile,
    Bomb,
}

use CellType::*;

#[derive(Clone, Properties)]
struct CellProps {
    pub cell_type: CellType,
    pub x: isize,
    pub y: isize,
}

struct Cell {
    props: CellProps,
}

impl Component for Cell {
    type Message = ();
    type Properties = CellProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let CellProps { cell_type, x, y } = self.props;
        match cell_type {
            CellType::Bomb => {
                html! {
                    <circle cx={x + SIZE / 2} cy={y + SIZE / 2} r={SIZE / 2} class="fill" />
                }
            }
            CellType::Tile => {
                html! {
                    <rect x={x} y={y} width={SIZE} height={SIZE} class="stroke" />
                }
            }
        }
    }
}

struct Board;

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

fn main() {
    yew::start_app::<Board>();
}
