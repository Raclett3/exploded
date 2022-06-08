use super::{animation::*, board::AnimatedBoard, GameAction, WIDTH};
use crate::animation::*;
use common::board::CellType;
use rand::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use yew::Reducible;

fn generate_x_pairs() -> Vec<(usize, usize)> {
    (0..WIDTH)
        .flat_map(|first| (first + 1..WIDTH).map(move |second| (first, second)))
        .collect()
}

#[derive(Clone)]
struct BombGenerator<T> {
    shuffled: Vec<T>,
    rng: StdRng,
    generator: fn() -> Vec<T>,
}

impl<T> BombGenerator<T> {
    fn new(generator: fn() -> Vec<T>) -> Self {
        let random = js_sys::Math::random();
        let rng = StdRng::seed_from_u64(u64::from_be_bytes(random.to_be_bytes()));
        let mut generator = BombGenerator {
            shuffled: Vec::new(),
            rng,
            generator,
        };
        generator.shuffle();
        generator
    }

    fn shuffle(&mut self) {
        let mut shuffled = (self.generator)();
        shuffled.shuffle(&mut self.rng);
        self.shuffled = shuffled;
    }

    fn next(&mut self) -> T {
        self.shuffled.pop().unwrap_or_else(|| {
            self.shuffle();
            self.next()
        })
    }
}

#[derive(Clone)]
pub struct Game {
    pub board: AnimatedBoard,
    generator: BombGenerator<(usize, usize)>,
    pub score: usize,
    pub bombs_removed: usize,
    pub bombs_limit: usize,
    pub score_animator: Rc<RefCell<FloatAnimator<NumberAnimator>>>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            board: AnimatedBoard::new(),
            generator: BombGenerator::new(generate_x_pairs),
            score: 0,
            bombs_removed: 0,
            bombs_limit: 999,
            score_animator: Rc::new(RefCell::new(FloatAnimator::new(Box::new(
                NumberAnimator::new(0),
            )))),
        }
    }

    pub fn is_over(&self) -> bool {
        let reached_limit = self.bombs_limit <= self.bombs_removed;
        self.board.is_filled() || reached_limit
    }

    pub fn next_row(&mut self) -> [CellType; WIDTH] {
        let bombs = self.generator.next();
        let mut row = [CellType::Tile; WIDTH];
        row[bombs.0] = CellType::Bomb;
        row[bombs.1] = CellType::Bomb;
        row
    }
}

impl Reducible for Game {
    type Action = GameAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut game = (*self).clone();

        match action {
            GameAction::Remove(x, y) => {
                if game.is_over() {
                    return game.into();
                }

                let (removed_cells, removed_bombs) = game.board.remove(x, y);
                if removed_cells > 0 {
                    game.score += (removed_cells + 1) * removed_cells / 2;
                    game.score_animator
                        .borrow_mut()
                        .animation
                        .set_target(game.score);
                    game.bombs_removed += removed_bombs;

                    game.board.apply_gravity();
                    let row = game.next_row();
                    game.board.feed(&row);
                }
            }
            GameAction::Feed => {
                let row = game.next_row();
                game.board.feed(&row);
            }
            GameAction::Animate => {
                self.board.animate();
                self.score_animator.borrow_mut().animate();
            }

            GameAction::Retry => {
                let mut game = Game::new();
                let row = game.next_row();
                game.board.feed(&row);
                return Rc::new(game);
            }
        }

        game.into()
    }
}
