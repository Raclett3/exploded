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
    pub opacity: f64,
    pub size: f64,
}

#[function_component(Cell)]
pub fn cell(props: &Props) -> Html {
    let Props {
        cell_type,
        x,
        y,
        opacity,
        size,
    } = props.clone();
    let x = x as f64 * size;
    let y = y as f64 * size;
    let opacity = opacity.to_string();
    match cell_type {
        CellType::Bomb => {
            let cx = (x + size / 2.).to_string();
            let cy = (y + size / 2.).to_string();
            let r = (size / 2.).to_string();
            html! {
                <circle cx={cx} cy={cy} r={r} opacity={opacity} class="fill" />
            }
        }
        CellType::Tile => {
            let x = x.to_string();
            let y = y.to_string();
            let width = size.to_string();
            let height = width.clone();
            html! {
                <rect x={x} y={y} width={width} height={height} opacity={opacity} class="stroke" />
            }
        }
    }
}
