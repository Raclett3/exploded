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

pub struct Game {
    board: Vec<Vec<Option<CellType>>>,
    width: usize,
    height: usize,
}

impl Game {
    pub fn new(width: usize, height: usize) -> Self {
        Game {
            board: vec![vec![None; height]; width],
            width,
            height,
        }
    }

    pub fn remove(&mut self, x: usize, y: usize) -> usize {
        let cell = self
            .board
            .get_mut(x)
            .and_then(|x| x.get_mut(y))
            .and_then(|x| x.take());
        match cell {
            None => 0,
            Some(CellType::Tile) => 1,
            Some(CellType::Bomb) => {
                adjacent_cells(x, y, self.width, self.height)
                    .map(|(x, y)| self.remove(x, y))
                    .sum::<usize>()
                    + 1
            }
        }
    }

    pub fn apply_gravity(&mut self) {
        for x in 0..self.width {
            let mut blank_cells_below = 0;

            for y in (0..self.height).rev() {
                if self.board[x][y].is_some() {
                    self.board[x].swap(y, y + blank_cells_below);
                } else {
                    blank_cells_below += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use CellType::*;

    fn from_board(width: usize, height: usize, board: Vec<Vec<Option<CellType>>>) -> Game {
        Game {
            board,
            width,
            height,
        }
    }

    #[test]
    fn test_remove() {
        let mut game = from_board(
            3,
            6,
            vec![
                vec![None, None, None, Some(Tile), Some(Tile), Some(Tile)],
                vec![None, None, None, Some(Tile), Some(Bomb), Some(Tile)],
                vec![None, None, None, Some(Tile), Some(Tile), Some(Tile)],
            ],
        );
        assert_eq!(game.remove(1, 3), 1);
        assert_eq!(game.remove(1, 4), 8);
        assert_eq!(game.board, vec![vec![None; 6]; 3]);
    }

    #[test]
    fn test_apply_gravity() {
        let mut game = from_board(
            3,
            4,
            vec![
                vec![Some(Tile), None, None, Some(Bomb)],
                vec![None, Some(Tile), Some(Bomb), None],
                vec![None, Some(Tile), None, Some(Bomb)],
            ],
        );

        game.apply_gravity();

        assert_eq!(
            game.board,
            vec![vec![None, None, Some(Tile), Some(Bomb)]; 3]
        )
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
