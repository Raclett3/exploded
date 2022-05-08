use crate::board::CellType;
use yew::prelude::*;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub cell_type: CellType,
    pub color: &'static str,
    pub x: f64,
    pub y: f64,
    pub opacity: f64,
    pub expansion: f64,
    pub size: f64,
}

#[function_component(Particle)]
pub fn particle(props: &Props) -> Html {
    let Props {
        cell_type,
        color,
        x,
        y,
        opacity,
        size,
        expansion,
    } = props.clone();
    let x = x as f64 * size;
    let y = y as f64 * size;
    let opacity = opacity.to_string();
    let cx = x + size / 2.;
    let cy = y + size / 2.;
    let r = size / 2. * expansion;

    match cell_type {
        CellType::Bomb => {
            let cx = cx.to_string();
            let cy = cy.to_string();
            let r = r.to_string();
            html! {
                <circle cx={cx} cy={cy} r={r} opacity={opacity} stroke={color} stroke-width="1px" fill="none" />
            }
        }
        CellType::Tile => {
            let x = (cx - r).to_string();
            let y = (cy - r).to_string();
            let width = (r * 2.).to_string();
            let height = width.clone();
            let rotate = format!("rotate({})", (expansion * 90.) as isize);
            html! {
                <rect x={x} y={y} width={width} height={height} opacity={opacity} transform={rotate} class="rotate-center stroke" />
            }
        }
    }
}
