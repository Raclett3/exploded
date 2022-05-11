use super::cell::Cell;
use super::particle::Particle;
use crate::game::{FloatingCell, FloatingParticle};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props<const WIDTH: usize, const HEIGHT: usize> {
    pub cell_size: f64,
    pub floating_cells: Vec<FloatingCell>,
    pub particles: Vec<FloatingParticle>,
}

#[function_component(Board)]
pub fn board<const WIDTH: usize, const HEIGHT: usize>(props: &Props<WIDTH, HEIGHT>) -> Html {
    let Props {
        cell_size,
        floating_cells,
        particles,
    } = props;
    let cell_size = *cell_size;

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

    html! {
        <>
            {for cells}
            {for particles}
        </>
    }
}
