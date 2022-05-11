use super::cell::Cell;
use super::particle::Particle;
use crate::game::{FloatingCell, FloatingParticle};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props<const WIDTH: usize, const HEIGHT: usize> {
    pub cell_size: f64,
    pub floating_cells: Vec<FloatingCell>,
    pub is_animating: bool,
    pub particles: Vec<FloatingParticle>,
    pub score: usize,
    pub is_game_over: bool,
    pub numerator: usize,
    pub denominator: usize,
    pub rank: &'static str,
}

#[function_component(Board)]
pub fn board<const WIDTH: usize, const HEIGHT: usize>(props: &Props<WIDTH, HEIGHT>) -> Html {
    let Props {
        cell_size,
        floating_cells,
        is_animating,
        particles,
        score,
        is_game_over,
        numerator,
        denominator,
        rank,
    } = props;
    let cell_size = *cell_size;
    let width = (WIDTH as f64 * cell_size).to_string();
    let height = (HEIGHT as f64 * cell_size).to_string();
    let center_x = (WIDTH as f64 * cell_size / 2.).to_string();
    let center_y = (HEIGHT as f64 * cell_size / 2.).to_string();
    let upper_y = (HEIGHT as f64 * cell_size / 3.).to_string();
    let lower_y = (HEIGHT as f64 * cell_size / 3. * 2.).to_string();

    let cells = floating_cells.iter().map(|cell| {
        let &FloatingCell {
            id,
            x,
            y,
            cell_type,
            opacity,
        } = cell;
        html! {
            <Cell key={id} x={x} y={y} opacity={opacity} size={cell_size} cell_type={cell_type} />
        }
    });

    let particles = particles.iter().map(|x| {
        html! {
            <Particle key={x.id} cell_type={x.cell_type} x={x.x} y={x.y} color={x.color} opacity={x.opacity} expansion={x.expansion} size={cell_size} />
        }
    });

    let font_size = cell_size * 0.5;
    let font_size_large = font_size * 2.;

    html! {
        <svg width={width.clone()} height={height.clone()}>
            <text x={center_x.clone()} y={center_y.clone()} class="numerator" font-size={format!("{font_size_large}px")}>{format!("{numerator:03}")}</text>
            <text x={center_x.clone()} y={center_y} class="denominator" font-size={format!("{font_size_large}px")}>{format!("{denominator:03}")}</text>
            {for cells}
            {for particles}
            if *is_game_over && !is_animating {
                <rect x="0" y="0" width={width} height={height} fill="rgba(0, 0, 0, 0.5)" />
                <text x={center_x.clone()} y={upper_y} class="text-center" font-size={format!("{font_size_large}px")} dominant-baseline="hanging">{"GAME OVER"}</text>
                <text x={center_x} y={lower_y} class="text-center" dominant-baseline="baseline">
                    <tspan font-size={format!("{font_size}px")}>{"RANK:"}</tspan>
                    <tspan font-size={format!("{font_size_large}px")}>{rank}</tspan>
                </text>
            }
            <text x="0" y="0" class="text" font-size={format!("{font_size}px")}>{format!("SCORE: {}", score)}</text>
        </svg>
    }
}
