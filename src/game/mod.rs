mod animation;

use crate::animation::*;
use crate::board::{Board, Cell, CellType};
use rand::prelude::*;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;
use yew::Reducible;

pub use animation::*;

pub const WIDTH: usize = 8;
pub const HEIGHT: usize = 9;

const PARTICLE_COLORS: [&str; 7] = [
    "#FF0000", "#FF8800", "#FFFF00", "#00FF00", "#00FFFF", "#0000FF", "#FF00FF",
];

pub const SINGLE_FREQUENCY: [usize; 10] = [9999, 9, 8, 7, 6, 5, 4, 3, 2, 3];

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

#[derive(Clone, PartialEq)]
enum VisibleState {
    Visible,
    Invisible,
    InvisibleWhileAnimation,
}

use VisibleState::*;

#[derive(Clone)]
pub struct AnimatedBoard {
    pub board: Board<WIDTH, HEIGHT>,
    visible: VisibleState,
    #[allow(clippy::type_complexity)]
    pub animator: Rc<
        RefCell<
            FloatAnimator<
                Option<(Vec<FloatingCell>, Vec<Sound>)>,
                AnimationStream<(Vec<FloatingCell>, Vec<Sound>)>,
            >,
        >,
    >,
    pub particles:
        Rc<RefCell<FloatAnimator<Vec<FloatingParticle>, EndlessAnimator<FloatingParticle>>>>,
}

impl AnimatedBoard {
    fn new() -> Self {
        AnimatedBoard {
            board: Board::new(),
            visible: Visible,
            animator: Rc::new(RefCell::new(FloatAnimator::new(Box::new(
                AnimationStream::new(),
            )))),
            particles: Rc::new(RefCell::new(FloatAnimator::new(Box::new(
                EndlessAnimator::new(Vec::new()),
            )))),
        }
    }

    fn feed(&mut self, row: &[CellType; WIDTH]) {
        self.board.feed(row);
        let visible = self.visible == Visible;

        let feed_animation = self
            .board
            .cells
            .iter()
            .enumerate()
            .flat_map(|(x, col)| {
                col.iter().enumerate().flat_map(move |(y, cell)| {
                    cell.map(|cell| {
                        let Cell { id, cell_type } = cell;
                        let opacity = if !visible && cell_type == CellType::Tile {
                            0.
                        } else {
                            1.
                        };
                        Box::new(CellAnimator::new(
                            id,
                            x as f64,
                            ((y + 1) as f64, y as f64),
                            (opacity, opacity),
                            0,
                            10,
                            cell_type,
                        )) as Box<dyn Animation<FloatingCell>>
                    })
                })
            })
            .collect();
        let feed_sounds = if self.is_filled() {
            vec![(3, Sound::Feed), (10, Sound::Stuck)]
        } else {
            vec![(3, Sound::Feed)]
        };

        self.animator
            .borrow_mut()
            .animation
            .push(Animator::new(feed_animation).zip(SoundPlayer::new(feed_sounds)));
    }

    fn remove(&mut self, x: usize, y: usize) -> (usize, usize) {
        let dists = self.board.remove(x, y);

        if dists.is_empty() {
            return (0, 0);
        }

        let visible = self.visible == Visible;

        let mut particle_animator = self.particles.borrow_mut();
        dists
            .iter()
            .map(|&(id, dist, x, y, cell_type)| {
                let (color, expansion, duration) = match cell_type {
                    CellType::Bomb => (PARTICLE_COLORS[dist % 7], (0., 3.), 40),
                    CellType::Tile => ("#FFFFFF", (0., 1.), 10),
                };
                ParticleAnimator::new(
                    id + 1_000_000,
                    color,
                    cell_type,
                    x as f64,
                    y as f64,
                    expansion,
                    (1., 0.),
                    dist * 3,
                    duration,
                )
            })
            .for_each(|x| particle_animator.animation.push(x));

        let remove_animation = self
            .board
            .cells
            .iter()
            .enumerate()
            .flat_map(|(x, col)| {
                col.iter().enumerate().flat_map(move |(y, cell)| {
                    cell.map(|cell| {
                        let Cell { cell_type, id } = cell;
                        let opacity = if !visible && cell_type == CellType::Tile {
                            0.
                        } else {
                            1.
                        };
                        Box::new(CellAnimator::new(
                            id,
                            x as f64,
                            (y as f64, y as f64),
                            (opacity, opacity),
                            0,
                            1,
                            cell_type,
                        )) as Box<dyn Animation<FloatingCell>>
                    })
                })
            })
            .chain(dists.iter().map(|&(id, dist, x, y, cell_type)| {
                Box::new(CellAnimator::new(
                    id,
                    x as f64,
                    (y as f64, y as f64),
                    (1., 0.),
                    dist * 3,
                    10,
                    cell_type,
                )) as Box<dyn Animation<FloatingCell>>
            }))
            .collect();
        let remove_sounds = dists
            .iter()
            .flat_map(|&(_, dist, _, _, cell_type)| {
                if cell_type == CellType::Bomb || dist == 0 {
                    Some((dist * 3, Sound::Break))
                } else {
                    None
                }
            })
            .collect();
        self.animator
            .borrow_mut()
            .animation
            .push(Animator::new(remove_animation).zip(SoundPlayer::new(remove_sounds)));
        let bombs = dists.iter().filter(|x| x.4 == CellType::Bomb).count();
        (dists.len(), bombs)
    }

    fn apply_gravity(&mut self) {
        let dists = self.board.apply_gravity();
        let visible = self.visible == Visible;
        let fall_animation = self
            .board
            .cells
            .iter()
            .enumerate()
            .flat_map(|(x, col)| {
                let dists = dists.clone();
                col.iter().enumerate().flat_map(move |(y, cell)| {
                    cell.map(|cell| {
                        let Cell { id, cell_type } = cell;
                        let dist = dists.get(&id).cloned().unwrap_or(0);
                        let opacity = if !visible && cell_type == CellType::Tile {
                            0.
                        } else {
                            1.
                        };
                        Box::new(CellAnimator::new(
                            id,
                            x as f64,
                            ((y - dist) as f64, y as f64),
                            (opacity, opacity),
                            0,
                            dist * 5 + 1,
                            cell_type,
                        )) as Box<dyn Animation<FloatingCell>>
                    })
                })
            })
            .collect();
        let mut dist_set = BTreeSet::new();
        for (_, &dist) in dists.iter() {
            dist_set.insert(dist);
        }
        let fall_sounds = dist_set
            .iter()
            .map(|dist| (dist * 5 + 1, Sound::Fall))
            .collect();
        self.animator
            .borrow_mut()
            .animation
            .push(Animator::new(fall_animation).zip(SoundPlayer::new(fall_sounds)));
    }

    fn is_filled(&self) -> bool {
        self.board
            .cells
            .iter()
            .any(|x| x.first().cloned().flatten().is_some())
    }

    fn animate(&self) {
        self.animator.borrow_mut().animate();
        self.particles.borrow_mut().animate();
    }

    pub fn frame(&self) -> (Vec<FloatingCell>, Vec<Sound>) {
        self.animator.borrow().frame().unwrap_or_else(|| {
            let cells = self
                .board
                .cells
                .iter()
                .enumerate()
                .flat_map(|(x, column)| {
                    column.iter().enumerate().flat_map(move |(y, cell)| {
                        cell.map(|cell| {
                            let (x, y) = (x as f64, y as f64);
                            let Cell { id, cell_type } = cell;
                            let opacity =
                                if self.visible == Invisible && cell_type == CellType::Tile {
                                    0.
                                } else {
                                    1.
                                };
                            FloatingCell {
                                x,
                                y,
                                id,
                                cell_type,
                                opacity,
                            }
                        })
                    })
                })
                .collect();
            (cells, Vec::new())
        })
    }

    pub fn particles(&self) -> Vec<FloatingParticle> {
        self.particles.borrow().frame()
    }

    pub fn is_animating(&self) -> bool {
        !self.animator.borrow().animation.is_over()
    }
}

fn generate_x_pairs() -> Vec<(usize, usize)> {
    (0..WIDTH)
        .flat_map(|first| (first + 1..WIDTH).map(move |second| (first, second)))
        .collect()
}

#[derive(Clone)]
pub struct Game {
    pub board: AnimatedBoard,
    generator: BombGenerator<(usize, usize)>,
    pub score: usize,
    pub bombs_removed: usize,
    pub bombs_limit: usize,
    pub score_animator: Rc<RefCell<FloatAnimator<usize, NumberAnimator>>>,
}

pub enum GameAction {
    Feed,
    Remove(usize, usize),
    Animate,
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
        let mut self_cloned = (*self).clone();

        match action {
            GameAction::Remove(x, y) => {
                if self_cloned.is_over() {
                    if !self_cloned.board.is_animating() {
                        let mut game = Game::new();
                        let row = game.next_row();
                        game.board.feed(&row);
                        return Rc::new(game);
                    } else {
                        return self_cloned.into();
                    }
                }

                let (removed_cells, removed_bombs) = self_cloned.board.remove(x, y);
                if removed_cells > 0 {
                    self_cloned.score += (removed_cells + 1) * removed_cells / 2;
                    self_cloned
                        .score_animator
                        .borrow_mut()
                        .animation
                        .set_target(self_cloned.score);
                    self_cloned.bombs_removed += removed_bombs;

                    self_cloned.board.apply_gravity();
                    let row = self_cloned.next_row();
                    self_cloned.board.feed(&row);
                }
            }
            GameAction::Feed => {
                let row = self_cloned.next_row();
                self_cloned.board.feed(&row);
            }
            GameAction::Animate => {
                self.board.animate();
                self.score_animator.borrow_mut().animate();
            }
        }

        self_cloned.into()
    }
}

fn cumulate(iter: impl Iterator<Item = usize>) -> impl Iterator<Item = usize> {
    iter.scan(0, |acc, x| {
        *acc += x;
        Some(*acc)
    })
}

#[derive(Clone)]
struct SpreadBombGenerator {
    rng: StdRng,
    generated: [usize; WIDTH],
}

impl SpreadBombGenerator {
    fn new() -> Self {
        let random = js_sys::Math::random();
        let rng = StdRng::seed_from_u64(u64::from_be_bytes(random.to_be_bytes()));
        SpreadBombGenerator {
            rng,
            generated: [0; WIDTH],
        }
    }

    fn possibility(&self) -> Vec<usize> {
        let max = self.generated.iter().copied().max().unwrap();
        self.generated.iter().map(|x| 1 << (max - x)).collect()
    }

    fn next_double(&mut self) -> (usize, usize) {
        let mut possibility = self.possibility();

        let mut sum = possibility.iter().sum::<usize>();
        let r = self.rng.gen_range(0..sum);

        let left = cumulate(possibility.iter().copied())
            .enumerate()
            .filter(|&(_, x)| r < x)
            .map(|(i, _)| i)
            .next()
            .unwrap();

        sum -= possibility[left];
        possibility[left] = 0;
        let r = self.rng.gen_range(0..sum);

        let right = cumulate(possibility.iter().copied())
            .enumerate()
            .filter(|&(_, x)| r < x)
            .map(|(i, _)| i)
            .next()
            .unwrap();

        self.generated[left] += 1;
        self.generated[right] += 1;
        (left, right)
    }

    fn next_single(&mut self) -> usize {
        let possibility = self.possibility();

        let sum = possibility.iter().sum::<usize>();
        let r = self.rng.gen_range(0..sum);

        let bomb = cumulate(possibility.iter().copied())
            .enumerate()
            .filter(|&(_, x)| r < x)
            .map(|(i, _)| i)
            .next()
            .unwrap();

        self.generated[bomb] += 1;
        bomb
    }
}

#[derive(Clone)]
pub struct GameHard {
    pub board: AnimatedBoard,
    generator: SpreadBombGenerator,
    pub until_single: usize,
    pub section: usize,
    pub level: usize,
    pub level_limit: usize,
    pub score_animator: Rc<RefCell<FloatAnimator<usize, NumberAnimator>>>,
}

impl GameHard {
    pub fn new() -> Self {
        GameHard {
            board: AnimatedBoard::new(),
            generator: SpreadBombGenerator::new(),
            until_single: 999,
            section: 0,
            level: 0,
            level_limit: 999,
            score_animator: Rc::new(RefCell::new(FloatAnimator::new(Box::new(
                NumberAnimator::new(0),
            )))),
        }
    }

    pub fn is_over(&self) -> bool {
        let reached_limit = self.level >= self.level_limit;
        self.board.is_filled() || reached_limit
    }

    pub fn next_row(&mut self) -> [CellType; WIDTH] {
        if self.level % 100 != 99 && self.level != 998 {
            self.level += 1;
        }

        if self.until_single == 0 {
            self.until_single = SINGLE_FREQUENCY[self.section] - 1;
            let bomb = self.generator.next_single();
            let mut row = [CellType::Tile; WIDTH];
            row[bomb] = CellType::Bomb;
            row
        } else {
            let bombs = self.generator.next_double();
            let mut row = [CellType::Tile; WIDTH];
            row[bombs.0] = CellType::Bomb;
            row[bombs.1] = CellType::Bomb;
            self.until_single -= 1;
            row
        }
    }
}

impl Reducible for GameHard {
    type Action = GameAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut self_cloned = (*self).clone();

        match action {
            GameAction::Remove(x, y) => {
                if self_cloned.is_over() {
                    if !self_cloned.board.is_animating() {
                        let mut game = GameHard::new();
                        let row = game.next_row();
                        game.board.feed(&row);
                        return Rc::new(game);
                    } else {
                        return self_cloned.into();
                    }
                }

                let (removed_cells, removed_bombs) = self_cloned.board.remove(x, y);
                if removed_cells > 0 {
                    self_cloned.level += removed_bombs;
                    if self_cloned.section < self_cloned.level / 100 {
                        self_cloned.section = (self_cloned.level / 100).min(9);
                        self_cloned.until_single = SINGLE_FREQUENCY[self_cloned.section];
                        if self_cloned.section == 9 {
                            self_cloned.board.visible = Invisible;
                        }
                    }

                    self_cloned.board.apply_gravity();
                    let row = self_cloned.next_row();
                    self_cloned.board.feed(&row);

                    if self_cloned.is_over() && self_cloned.board.visible == Invisible {
                        self_cloned.board.visible = InvisibleWhileAnimation;
                    }
                }
            }
            GameAction::Feed => {
                let row = self_cloned.next_row();
                self_cloned.board.feed(&row);
            }
            GameAction::Animate => {
                self.board.animate();
                self.score_animator.borrow_mut().animate();
            }
        }

        self_cloned.into()
    }
}
