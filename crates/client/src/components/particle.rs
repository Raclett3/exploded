use common::board::CellType;
use yew::prelude::*;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub cell_type: CellType,
    pub color: &'static str,
    pub x: f64,
    pub y: f64,
    pub opacity: f64,
    pub expansion: f64,
}

#[function_component(Particle)]
pub fn particle(props: &Props) -> Html {
    let Props {
        cell_type,
        color,
        x,
        y,
        opacity,
        expansion,
    } = props.clone();
    let opacity = opacity.to_string();
    let cx = x + 0.5;
    let cy = y + 0.5;
    let r = 0.5 * expansion;

    match cell_type {
        CellType::Bomb => {
            let cx = cx.to_string();
            let cy = cy.to_string();
            let r = r.to_string();
            html! {
                <circle cx={cx} cy={cy} r={r} opacity={opacity} stroke={color} stroke-width="0.01" fill="none" />
            }
        }
        CellType::Tile => {
            let x = (cx - r).to_string();
            let y = (cy - r).to_string();
            let width = (r * 2.).to_string();
            let height = width.clone();
            let style = format!("transform:rotate({}deg);", (expansion * 90.) as isize);
            html! {
                <rect x={x} y={y} width={width} height={height} opacity={opacity} style={style} class="rotate-center stroke" />
            }
        }
    }
}
