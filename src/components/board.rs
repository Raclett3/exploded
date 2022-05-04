use super::cell::{Cell, CellType::*};
use crate::game::Cell as GameCell;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props<const WIDTH: usize, const HEIGHT: usize> {
    pub cell_size: f64,
    pub board: [[Option<GameCell>; HEIGHT]; WIDTH],
}

#[function_component(Board)]
pub fn board<const WIDTH: usize, const HEIGHT: usize>(props: &Props<WIDTH, HEIGHT>) -> Html {
    let Props { cell_size, board } = props;
    let cell_size = *cell_size;
    let width = (WIDTH as f64 * cell_size).to_string();
    let height = (HEIGHT as f64 * cell_size).to_string();

    let cells = board.iter().enumerate().flat_map(|(x, column)| {
        column.iter().enumerate().flat_map(move |(y, cell)| {
            cell.map(|cell| {
                let (x, y) = (x as f64, y as f64);
                let cell_type = if cell.is_bomb() {
                    Bomb
                } else {
                    Tile
                };
                html! {
                    <Cell x={x} y={y} size={cell_size} cell_type={cell_type} />
                }
            })
        })
    });

    html! {
        <svg width={width} height={height}>
            {for cells}
        </svg>
    }
}
