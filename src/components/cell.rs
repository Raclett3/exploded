use yew::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum CellType {
    Tile,
    Bomb,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub cell_type: CellType,
    pub x: f64,
    pub y: f64,
    pub size: f64,
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

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props == props {
            return false;
        }
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let Props {
            cell_type,
            x,
            y,
            size,
        } = self.props;
        let x = x as f64 * size;
        let y = y as f64 * size;
        match cell_type {
            CellType::Bomb => {
                html! {
                    <circle cx={x + size / 2.} cy={y + size / 2.} r={size / 2.} class="fill" />
                }
            }
            CellType::Tile => {
                html! {
                    <rect x={x} y={y} width={size} height={size} class="stroke" />
                }
            }
        }
    }
}
