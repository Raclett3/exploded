use super::{animation::*, HEIGHT, WIDTH};
use crate::animation::*;
use crate::board::{Board, Cell, CellType};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

const PARTICLE_COLORS: [&str; 7] = [
    "#FF0000", "#FF8800", "#FFFF00", "#00FF00", "#00FFFF", "#0000FF", "#FF00FF",
];

#[derive(Clone, PartialEq)]
pub enum VisibleState {
    Visible,
    Invisible,
    InvisibleWhileAnimation,
}

use VisibleState::*;

pub struct SingleAnimation<T> {
    target: RefCell<Vec<T>>,
}

impl<T> SingleAnimation<T> {
    pub fn new(target: Vec<T>) -> Self {
        SingleAnimation {
            target: RefCell::new(target),
        }
    }
}

impl<T> Animation<Vec<T>> for SingleAnimation<T> {
    fn current_frame(&self) -> Vec<T> {
        std::mem::take(self.target.borrow_mut().as_mut())
    }

    fn advance_frames(&mut self, _frames: usize) {}

    fn is_over(&self) -> bool {
        self.target.borrow().is_empty()
    }
}

#[derive(Clone)]
pub struct AnimatedBoard {
    pub board: Board<WIDTH, HEIGHT>,
    pub visible: VisibleState,
    #[allow(clippy::type_complexity)]
    pub animator: Rc<
        RefCell<
            FloatAnimator<
                Option<((Vec<FloatingCell>, Vec<Sound>), Vec<ParticleAnimator>)>,
                AnimationStream<((Vec<FloatingCell>, Vec<Sound>), Vec<ParticleAnimator>)>,
            >,
        >,
    >,
    pub particles:
        Rc<RefCell<FloatAnimator<Vec<FloatingParticle>, EndlessAnimator<FloatingParticle>>>>,
}

impl AnimatedBoard {
    pub fn new() -> Self {
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

    pub fn feed(&mut self, row: &[CellType; WIDTH]) {
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
                        let opacity = if visible || y == HEIGHT - 1 {
                            (1., 1.)
                        } else if y == HEIGHT - 2 {
                            (1., 0.)
                        } else {
                            (0., 0.)
                        };
                        Box::new(CellAnimator::new(
                            id,
                            x as f64,
                            ((y + 1) as f64, y as f64),
                            opacity,
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

        self.animator.borrow_mut().animation.push(
            Animator::new(feed_animation)
                .zip(SoundPlayer::new(feed_sounds))
                .zip(SingleAnimation::new(Vec::new())),
        );
    }

    pub fn remove(&mut self, x: usize, y: usize) -> (usize, usize) {
        let dists = self.board.remove(x, y);

        if dists.is_empty() {
            return (0, 0);
        }

        let visible = self.visible == Visible;

        let particles = dists
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
            .collect();

        let remove_animation = self
            .board
            .cells
            .iter()
            .enumerate()
            .flat_map(|(x, col)| {
                col.iter().enumerate().flat_map(move |(y, cell)| {
                    cell.map(|cell| {
                        let Cell { cell_type, id } = cell;
                        let opacity = if !visible && y != HEIGHT - 1 { 0. } else { 1. };
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
        let mut remove_sounds = dists
            .iter()
            .flat_map(|&(_, dist, _, _, cell_type)| {
                if cell_type == CellType::Bomb || dist == 0 {
                    Some((dist * 3, Sound::Break))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        remove_sounds.sort_by_key(|x| x.0);
        remove_sounds.dedup_by_key(|x| x.0);
        self.animator.borrow_mut().animation.push(
            Animator::new(remove_animation)
                .zip(SoundPlayer::new(remove_sounds))
                .zip(SingleAnimation::new(particles)),
        );
        let bombs = dists.iter().filter(|x| x.4 == CellType::Bomb).count();
        (dists.len(), bombs)
    }

    pub fn apply_gravity(&mut self) {
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
                        let opacity = if visible || y - dist == HEIGHT - 1 {
                            (1., 1.)
                        } else if y == HEIGHT - 1 {
                            (0., 1.)
                        } else {
                            (0., 0.)
                        };
                        Box::new(CellAnimator::new(
                            id,
                            x as f64,
                            ((y - dist) as f64, y as f64),
                            opacity,
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
        self.animator.borrow_mut().animation.push(
            Animator::new(fall_animation)
                .zip(SoundPlayer::new(fall_sounds))
                .zip(SingleAnimation::new(Vec::new())),
        );
    }

    pub fn reset(&mut self) {
        let particles = self
            .board
            .cells
            .iter()
            .enumerate()
            .flat_map(|(x, col)| {
                col.iter().enumerate().flat_map(move |(y, cell)| {
                    cell.map(|cell| {
                        let Cell { cell_type, id } = cell;
                        let (color, expansion, duration) = match cell_type {
                            CellType::Bomb => (PARTICLE_COLORS[HEIGHT - y - 1], (0., 3.), 40),
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
                            (HEIGHT - y - 1) * 10,
                            duration,
                        )
                    })
                })
            })
            .collect();

        let reset_animation = self
            .board
            .cells
            .iter()
            .enumerate()
            .flat_map(|(x, col)| {
                col.iter().enumerate().flat_map(move |(y, cell)| {
                    cell.map(|cell| {
                        let Cell { cell_type, id } = cell;
                        Box::new(CellAnimator::new(
                            id,
                            x as f64,
                            (y as f64, y as f64),
                            (1., 0.),
                            (HEIGHT - y - 1) * 10,
                            10,
                            cell_type,
                        )) as Box<dyn Animation<FloatingCell>>
                    })
                })
            })
            .collect();
        let mut animator = self.animator.borrow_mut();
        animator.animation.push(
            Animator::new(reset_animation)
                .zip(SoundPlayer::new(Vec::new()))
                .zip(SingleAnimation::new(particles)),
        );
        self.board.cells = [[None; HEIGHT]; WIDTH];
    }

    pub fn is_filled(&self) -> bool {
        self.board
            .cells
            .iter()
            .any(|x| x.first().cloned().flatten().is_some())
    }

    pub fn animate(&self) {
        self.animator.borrow_mut().animate();
        self.particles.borrow_mut().animate();
    }

    pub fn frame(&self) -> (Vec<FloatingCell>, Vec<Sound>) {
        let (frame, particles) = self.animator.borrow().frame().unwrap_or_else(|| {
            let cells = self
                .board
                .cells
                .iter()
                .enumerate()
                .flat_map(|(x, column)| {
                    column.iter().enumerate().flat_map(move |(y, cell)| {
                        cell.map(|cell| {
                            let Cell { id, cell_type } = cell;
                            let opacity = if self.visible == Invisible && y != HEIGHT - 1 {
                                0.
                            } else {
                                1.
                            };
                            let (x, y) = (x as f64, y as f64);
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
            ((cells, Vec::new()), Vec::new())
        });
        let mut particles_animator = self.particles.borrow_mut();
        particles
            .into_iter()
            .for_each(|x| particles_animator.animation.push(x));
        frame
    }

    pub fn particles(&self) -> Vec<FloatingParticle> {
        self.particles.borrow().frame()
    }

    pub fn is_animating(&self) -> bool {
        !self.animator.borrow().animation.is_over()
    }
}
