use super::cell::Cell;
use super::particle::Particle;
use crate::board::Cell as GameCell;
use crate::game::{FloatingCell, FloatingParticle};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props<const WIDTH: usize, const HEIGHT: usize> {
    pub cell_size: f64,
    pub board: [[Option<GameCell>; HEIGHT]; WIDTH],
    pub floating_cells: Option<Vec<FloatingCell>>,
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
        board,
        floating_cells,
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

    let cells = if let Some(floating_cells) = floating_cells {
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
        html! {
            {for cells}
        }
    } else {
        let cells = board.iter().enumerate().flat_map(|(x, column)| {
            column.iter().enumerate().flat_map(move |(y, cell)| {
                cell.map(|cell| {
                    let (x, y) = (x as f64, y as f64);
                    html! {
                        <Cell key={cell.id} x={x} y={y} opacity={1.0} size={cell_size} cell_type={cell.cell_type} />
                    }
                })
            })
        });

        html! {
            {for cells}
        }
    };

    let particles = particles.iter().map(|x| {
        html! {
            <Particle key={x.id} cell_type={x.cell_type} x={x.x} y={x.y} color={x.color} opacity={x.opacity} expansion={x.expansion} size={cell_size} />
        }
    });

    let font_size = cell_size * 0.5;
    let font_size_large = font_size * 2.;

    html! {
        <svg width={width} height={height}>
            <text x={center_x.clone()} y={center_y.clone()} class="numerator" font-size={format!("{font_size_large}px")}>{format!("{numerator:03}")}</text>
            <text x={center_x.clone()} y={center_y} class="denominator" font-size={format!("{font_size_large}px")}>{format!("{denominator:03}")}</text>
            <text x="0" y="0" class="text" font-size={format!("{font_size}px")}>{format!("SCORE: {}", score)}</text>
            {cells}
            {for particles}
            if *is_game_over {
                <text x={center_x.clone()} y={upper_y} class="text-center" font-size={format!("{font_size_large}px")} alignment-baseline="hanging">{"GAME OVER"}</text>
                <text x={center_x} y={lower_y} class="text-center" alignment-baseline="baseline">
                    <tspan font-size={format!("{font_size}px")}>{"RANK:"}</tspan>
                    <tspan font-size={format!("{font_size_large}px")}>{rank}</tspan>
                </text>
            }
        </svg>
    }
}
