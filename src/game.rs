use std::collections::{BTreeMap, VecDeque};

fn adjacent_cells(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> impl Iterator<Item = (usize, usize)> {
    let x = x as isize;
    let y = y as isize;
    (-1..=1)
        .flat_map(|x| (-1..=1).map(move |y| (x, y)))
        .filter(|&x| x != (0, 0))
        .flat_map(move |(vx, vy)| {
            (x + vx)
                .try_into()
                .and_then(|x| (y + vy).try_into().map(|y| (x, y)))
        })
        .filter(move |&(x, y)| x < width && y < height)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellType {
    Tile,
    Bomb,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub id: usize,
    pub cell_type: CellType,
}

impl Cell {
    fn new(id: usize, cell_type: CellType) -> Self {
        Cell { id, cell_type }
    }

    pub fn is_bomb(&self) -> bool {
        self.cell_type == CellType::Bomb
    }
}

#[derive(Clone)]
pub struct Game<const WIDTH: usize, const HEIGHT: usize> {
    pub board: [[Option<Cell>; HEIGHT]; WIDTH],
    generated_cells: usize,
}

impl<const WIDTH: usize, const HEIGHT: usize> Game<WIDTH, HEIGHT> {
    pub fn new() -> Self {
        Game {
            board: [[None; HEIGHT]; WIDTH],
            generated_cells: 0,
        }
    }

    pub fn remove(&mut self, x: usize, y: usize) -> Vec<(usize, usize, usize)> {
        let mut queue = VecDeque::new();
        queue.push_back((x, y, 0));
        let mut dists = Vec::new();

        while let Some((x, y, dist)) = queue.pop_front() {
            let cell = self
                .board
                .get_mut(x)
                .and_then(|x| x.get_mut(y))
                .and_then(|x| x.take());

            if let Some(Cell { cell_type, .. }) = cell {
                dists.push((dist, x, y));
                if cell_type == CellType::Bomb {
                    for (x, y) in adjacent_cells(x, y, WIDTH, HEIGHT) {
                        queue.push_back((x, y, dist + 1));
                    }
                }
            }
        }

        dists
    }

    pub fn apply_gravity(&mut self) -> BTreeMap<usize, usize> {
        let mut fall_distance = BTreeMap::new();

        for x in 0..WIDTH {
            let mut blank_cells_below = 0;

            for y in (0..HEIGHT).rev() {
                if let Some(Cell { id, .. }) = self.board[x][y] {
                    if blank_cells_below > 0 {
                        fall_distance.insert(id, blank_cells_below);
                    }

                    self.board[x].swap(y, y + blank_cells_below);
                } else {
                    blank_cells_below += 1;
                }
            }
        }

        fall_distance
    }

    pub fn feed(&mut self, row: &[CellType; WIDTH]) -> [Cell; WIDTH] {
        let row = row.map(|cell| {
            let cell = Cell::new(self.generated_cells, cell);
            self.generated_cells += 1;
            cell
        });

        for (cell, column) in row.iter().cloned().zip(self.board.iter_mut()) {
            column.rotate_left(1);
            *column.last_mut().unwrap() = Some(cell);
        }

        row
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use CellType::*;

    fn from_board<const WIDTH: usize, const HEIGHT: usize>(
        board: [[Option<Cell>; HEIGHT]; WIDTH],
    ) -> Game<WIDTH, HEIGHT> {
        Game {
            board,
            generated_cells: 0,
        }
    }

    fn cell(id: usize, cell_type: CellType) -> Option<Cell> {
        Some(Cell { id, cell_type })
    }

    trait Sorted {
        fn sorted(&self) -> Self;
    }

    impl<T: Ord + Clone> Sorted for Vec<T> {
        fn sorted(&self) -> Vec<T> {
            let mut vec = self.to_vec();
            vec.sort();
            vec
        }
    }

    #[test]
    fn test_remove() {
        let mut game = from_board::<3, 4>([
            [None, cell(6, Tile), cell(3, Tile), cell(0, Tile)],
            [None, cell(7, Tile), cell(4, Bomb), cell(1, Tile)],
            [None, cell(8, Tile), cell(5, Tile), cell(2, Tile)],
        ]);

        assert_eq!(game.remove(0, 3), vec![(0, 0, 3)]);
        assert_eq!(game.remove(1, 2).sorted(), vec![
            (0, 1, 2),
            (1, 0, 1),
            (1, 0, 2),
            (1, 1, 1),
            (1, 1, 3),
            (1, 2, 1),
            (1, 2, 2),
            (1, 2, 3),
        ]);
        assert_eq!(game.board, [[None; 4]; 3]);
    }

    #[test]
    fn test_apply_gravity() {
        let mut game = from_board::<3, 4>([
            [cell(0, Tile), None, None, cell(3, Bomb)],
            [None, cell(1, Tile), cell(4, Bomb), None],
            [None, cell(2, Tile), None, cell(5, Bomb)],
        ]);

        let mut map = BTreeMap::<usize, usize>::new();
        map.insert(0, 2);
        map.insert(1, 1);
        map.insert(2, 1);
        map.insert(4, 1);
        assert_eq!(game.apply_gravity(), map);

        assert_eq!(
            game.board,
            [
                [None, None, cell(0, Tile), cell(3, Bomb)],
                [None, None, cell(1, Tile), cell(4, Bomb)],
                [None, None, cell(2, Tile), cell(5, Bomb)],
            ]
        );
    }

    #[test]
    fn test_feed() {
        let mut game = from_board::<4, 3>([
            [None, cell(0, Tile), cell(0, Bomb)],
            [None, None, cell(0, Tile)],
            [None, None, cell(0, Tile)],
            [None, None, cell(0, Tile)],
        ]);

        let row = [Tile, Bomb, Tile, Bomb];
        assert_eq!(
            game.feed(&row).as_slice(),
            row.iter()
                .cloned()
                .enumerate()
                .map(|(i, x)| Cell::new(i, x))
                .collect::<Vec<_>>()
                .as_slice(),
        );

        assert_eq!(
            game.board,
            [
                [cell(0, Tile), cell(0, Bomb), cell(0, Tile)],
                [None, cell(0, Tile), cell(1, Bomb)],
                [None, cell(0, Tile), cell(2, Tile)],
                [None, cell(0, Tile), cell(3, Bomb)],
            ]
        );
    }

    #[test]
    fn test_adjacent_cells() {
        assert_eq!(
            adjacent_cells(1, 1, 3, 3).collect::<Vec<_>>(),
            vec![
                (0, 0),
                (0, 1),
                (0, 2),
                (1, 0),
                (1, 2),
                (2, 0),
                (2, 1),
                (2, 2)
            ]
        );

        assert_eq!(
            adjacent_cells(0, 0, 3, 3).collect::<Vec<_>>(),
            vec![(0, 1), (1, 0), (1, 1)]
        );

        assert_eq!(
            adjacent_cells(2, 2, 3, 3).collect::<Vec<_>>(),
            vec![(1, 1), (1, 2), (2, 1)]
        );
    }
}
