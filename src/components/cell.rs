use crate::board::CellType;
use yew::prelude::*;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub cell_type: CellType,
    pub x: f64,
    pub y: f64,
    pub opacity: f64,
}

#[function_component(Cell)]
pub fn cell(props: &Props) -> Html {
    let Props {
        cell_type,
        x,
        y,
        opacity,
    } = props.clone();
    let opacity = opacity.to_string();
    match cell_type {
        CellType::Bomb => {
            let cx = (x + 0.5).to_string();
            let cy = (y + 0.5).to_string();
            html! {
                <circle cx={cx} cy={cy} r="0.5" opacity={opacity} class="fill" />
            }
        }
        CellType::Tile => {
            let x = x.to_string();
            let y = y.to_string();
            html! {
                <rect x={x} y={y} width="1" height="1" opacity={opacity} class="stroke" />
            }
        }
    }
}
