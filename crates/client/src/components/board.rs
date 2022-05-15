use super::cell::Cell;
use super::particle::Particle;
use crate::game::{FloatingCell, FloatingParticle};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props<const WIDTH: usize, const HEIGHT: usize> {
    pub floating_cells: Vec<FloatingCell>,
    pub particles: Vec<FloatingParticle>,
}

#[function_component(Board)]
pub fn board<const WIDTH: usize, const HEIGHT: usize>(props: &Props<WIDTH, HEIGHT>) -> Html {
    let Props {
        floating_cells,
        particles,
    } = props;

    let cells = floating_cells.iter().map(|cell| {
        let &FloatingCell {
            id,
            x,
            y,
            cell_type,
            opacity,
        } = cell;
        html! {
            <Cell key={id} x={x} y={y} opacity={opacity} cell_type={cell_type} />
        }
    });

    let particles = particles.iter().map(|x| {
        html! {
            <Particle key={x.id} cell_type={x.cell_type} x={x.x} y={x.y} color={x.color} opacity={x.opacity} expansion={x.expansion} />
        }
    });

    html! {
        <>
            <rect width={WIDTH.to_string()} height={HEIGHT.to_string()} class="stroke" />
            {for cells}
            {for particles}
        </>
    }
}
