use crate::animation::*;
use crate::board::CellType;
use std::cell::RefCell;

#[derive(Clone, PartialEq)]
pub struct FloatingCell {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub cell_type: CellType,
    pub opacity: f64,
}

fn interpolation((from, to): (f64, f64), position: f64) -> f64 {
    (from * (1. - position)) + (to * position)
}

pub struct CellAnimator {
    id: usize,
    x: f64,
    y: (f64, f64),
    opacity: (f64, f64),
    delay: usize,
    duration: usize,
    elapsed: usize,
    cell_type: CellType,
}

impl CellAnimator {
    pub fn new(
        id: usize,
        x: f64,
        y: (f64, f64),
        opacity: (f64, f64),
        delay: usize,
        duration: usize,
        cell_type: CellType,
    ) -> Self {
        CellAnimator {
            id,
            x,
            y,
            opacity,
            delay,
            duration,
            cell_type,
            elapsed: 0,
        }
    }
}

impl Animation<FloatingCell> for CellAnimator {
    fn advance_frames(&mut self, frames: usize) {
        self.elapsed += frames;
    }

    fn current_frame(&self) -> FloatingCell {
        let relative_time = self.elapsed.saturating_sub(self.delay).min(self.duration) as f64
            / self.duration as f64;
        FloatingCell {
            id: self.id,
            x: self.x,
            y: interpolation(self.y, relative_time),
            cell_type: self.cell_type,
            opacity: interpolation(self.opacity, relative_time),
        }
    }

    fn is_over(&self) -> bool {
        self.duration + self.delay <= self.elapsed
    }
}

#[derive(Clone, PartialEq)]
pub struct FloatingParticle {
    pub id: usize,
    pub color: &'static str,
    pub cell_type: CellType,
    pub x: f64,
    pub y: f64,
    pub expansion: f64,
    pub opacity: f64,
}

pub struct ParticleAnimator {
    id: usize,
    color: &'static str,
    cell_type: CellType,
    x: f64,
    y: f64,
    expansion: (f64, f64),
    opacity: (f64, f64),
    delay: usize,
    duration: usize,
    elapsed: usize,
}

impl ParticleAnimator {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: usize,
        color: &'static str,
        cell_type: CellType,
        x: f64,
        y: f64,
        expansion: (f64, f64),
        opacity: (f64, f64),
        delay: usize,
        duration: usize,
    ) -> Self {
        ParticleAnimator {
            id,
            color,
            cell_type,
            x,
            y,
            expansion,
            opacity,
            delay,
            duration,
            elapsed: 0,
        }
    }
}

impl Animation<FloatingParticle> for ParticleAnimator {
    fn advance_frames(&mut self, frames: usize) {
        self.elapsed += frames;
    }

    fn current_frame(&self) -> FloatingParticle {
        let relative_time = self.elapsed.saturating_sub(self.delay).min(self.duration) as f64
            / self.duration as f64;
        FloatingParticle {
            id: self.id,
            color: self.color,
            cell_type: self.cell_type,
            x: self.x,
            y: self.y,
            expansion: interpolation(self.expansion, relative_time),
            opacity: interpolation(self.opacity, relative_time),
        }
    }

    fn is_over(&self) -> bool {
        self.duration + self.delay <= self.elapsed
    }
}

pub struct NumberAnimator {
    target: usize,
    current: usize,
}

impl NumberAnimator {
    pub fn new(target: usize) -> Self {
        NumberAnimator { target, current: 0 }
    }

    pub fn set_target(&mut self, target: usize) {
        self.target = target;
    }
}

impl Animation<usize> for NumberAnimator {
    fn advance_frames(&mut self, frames: usize) {
        for _ in 0..frames {
            self.current = (self.current * 3 + self.target + 3) / 4;
        }
    }

    fn current_frame(&self) -> usize {
        self.current
    }

    fn is_over(&self) -> bool {
        false
    }
}

pub enum Sound {
    Break,
    Fall,
    Feed,
    Stuck,
    LevelUp,
}

pub struct SoundPlayer {
    frames_elapsed: usize,
    events: Vec<(usize, Sound)>,
    current: RefCell<Vec<Sound>>,
}

impl SoundPlayer {
    pub fn new(mut events: Vec<(usize, Sound)>) -> Self {
        events.sort_by_key(|x| std::cmp::Reverse(x.0));
        let mut player = SoundPlayer {
            events,
            frames_elapsed: 0,
            current: RefCell::new(Vec::new()),
        };
        player.advance_frames(0);
        player
    }
}

impl Animation<Vec<Sound>> for SoundPlayer {
    fn current_frame(&self) -> Vec<Sound> {
        let mut current = self.current.borrow_mut();
        std::mem::take(current.as_mut())
    }

    fn advance_frames(&mut self, frames: usize) {
        self.frames_elapsed += frames;
        while !self.events.is_empty() {
            if self.events[self.events.len() - 1].0 <= self.frames_elapsed {
                self.current.borrow_mut().push(self.events.pop().unwrap().1);
            } else {
                break;
            }
        }
    }

    fn is_over(&self) -> bool {
        self.events.is_empty() && self.current.borrow().is_empty()
    }
}
