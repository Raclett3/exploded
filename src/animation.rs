pub trait Animation<T> {
    fn advance_frames(&mut self, frames: usize);
    fn current_frame(&self) -> T;
    fn is_over(&self) -> bool;
}

pub struct ConstantAnimation<T, F: Fn() -> T>(F);

impl<T, F: Fn() -> T> Animation<T> for ConstantAnimation<T, F> {
    fn advance_frames(&mut self, _frames: usize) {}

    fn current_frame(&self) -> T {
        self.0()
    }

    fn is_over(&self) -> bool {
        true
    }
}

pub struct Animator<T> {
    animations: Vec<Box<dyn Animation<T>>>,
}

impl<T> Animator<T> {
    pub fn new(animations: Vec<Box<dyn Animation<T>>>) -> Self {
        Animator { animations }
    }
}

impl<T> Animation<Vec<T>> for Animator<T> {
    fn advance_frames(&mut self, frames: usize) {
        for anim in self.animations.iter_mut() {
            anim.advance_frames(frames);
        }
    }

    fn current_frame(&self) -> Vec<T> {
        let frame = self
            .animations
            .iter()
            .map(|anim| anim.current_frame())
            .collect();
        frame
    }

    fn is_over(&self) -> bool {
        self.animations.iter().all(|anim| anim.is_over())
    }
}

pub struct AnimationChain<T> {
    animations: Vec<Box<dyn Animation<T>>>,
}

impl<T> AnimationChain<T> {
    pub fn new(animations: Vec<Box<dyn Animation<T>>>) -> Self {
        AnimationChain {
            animations: animations.into_iter().rev().collect(),
        }
    }
}

impl<T> Animation<T> for AnimationChain<T> {
    fn advance_frames(&mut self, frames: usize) {
        for _ in 0..frames {
            if let Some(anim) = self.animations.last_mut() {
                anim.advance_frames(1);
            } else {
                break;
            }

            while self.animations.len() >= 2 && self.animations.last().unwrap().is_over() {
                self.animations.pop();
            }
        }
    }

    fn current_frame(&self) -> T {
        self.animations.last().unwrap().current_frame()
    }

    fn is_over(&self) -> bool {
        self.animations.last().unwrap().is_over()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Double {
        elapsed: usize,
        duration: usize,
    }

    impl Double {
        fn new(duration: usize) -> Self {
            Double {
                elapsed: 0,
                duration,
            }
        }
    }

    impl Animation<usize> for Double {
        fn advance_frames(&mut self, frames: usize) {
            self.elapsed += frames;
        }

        fn current_frame(&self) -> usize {
            self.elapsed * 2
        }

        fn is_over(&self) -> bool {
            self.elapsed >= self.duration
        }
    }

    #[test]
    fn test_animator() {
        let mut animator = Animator::new(vec![
            Box::new(ConstantAnimation(|| 2)),
            Box::new(Double::new(5)),
        ]);

        let mut frames = Vec::new();

        loop {
            frames.push(animator.current_frame());
            animator.advance_frames(1);
            if animator.is_over() {
                break;
            }
        }

        assert_eq!(
            frames,
            vec![vec![2, 0], vec![2, 2], vec![2, 4], vec![2, 6], vec![2, 8]]
        );
    }
}
