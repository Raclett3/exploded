use yew::prelude::*;

const SIZE: isize = 100;

#[derive(Clone, Copy)]
pub enum CellType {
    Tile,
    Bomb,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub cell_type: CellType,
    pub x: isize,
    pub y: isize,
}

pub struct Cell {
    props: Props,
}

impl Component for Cell {
    type Message = ();
    type Properties = Props;

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
        let Props { cell_type, x, y } = self.props;
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
