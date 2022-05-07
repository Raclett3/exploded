use yew::prelude::*;

const COLORS: [&str; 7] = [
    "#FF0000", "#FF8800", "#FFFF00", "#00FF00", "#00FFFF", "#0000FF", "#FF00FF",
];

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub color: usize,
    pub x: f64,
    pub y: f64,
    pub opacity: f64,
    pub expansion: f64,
    pub size: f64,
}

#[function_component(Particle)]
pub fn particle(props: &Props) -> Html {
    let Props {
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

    let cx = (x + size / 2.).to_string();
    let cy = (y + size / 2.).to_string();
    let r = (size / 2. * expansion).to_string();
    html! {
        <circle cx={cx} cy={cy} r={r} opacity={opacity} stroke={COLORS[color % 7]} stroke-width="1px" fill="none" />
    }
}
