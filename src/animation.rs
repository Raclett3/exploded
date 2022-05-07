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

pub struct FloatAnimator<T, A: Animation<T>> {
    begin_at: f64,
    elapsed_frames: usize,
    pub animator: A,
    phantom: std::marker::PhantomData<T>,
}

impl<T, A: Animation<T>> FloatAnimator<T, A> {
    pub fn new(animator: A) -> Self {
        let now = js_sys::Date::now();
        FloatAnimator {
            begin_at: now,
            elapsed_frames: 0,
            animator,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn animate(&mut self) {
        let now = js_sys::Date::now();
        let elapsed = now - self.begin_at;
        let frames = (elapsed / 1000.0 * 60.0).floor() as usize;
        let frame_delta = frames - self.elapsed_frames;
        self.elapsed_frames = frames;

        if frame_delta > 0 {
            self.animator.advance_frames(frame_delta);
        }
    }

    pub fn frame(&self) -> T {
        self.animator.current_frame()
    }

    pub fn is_over(&self) -> bool {
        self.animator.is_over()
    }
}

pub struct EndlessAnimator<T> {
    animations: Vec<Box<dyn Animation<T>>>,
}

impl<T> EndlessAnimator<T> {
    pub fn new(animations: Vec<Box<dyn Animation<T>>>) -> Self {
        EndlessAnimator {
            animations: animations.into_iter().rev().collect(),
        }
    }

    pub fn push(&mut self, animation: impl Animation<T> + 'static) {
        self.animations.push(Box::new(animation));
    }
}

impl<T> Animation<Vec<T>> for EndlessAnimator<T> {
    fn advance_frames(&mut self, frames: usize) {
        for anim in self.animations.iter_mut() {
            anim.advance_frames(frames);
        }
        self.animations.retain(|x| !x.is_over());
    }

    fn current_frame(&self) -> Vec<T> {
        self.animations.iter().map(|x| x.current_frame()).collect()
    }

    fn is_over(&self) -> bool {
        true
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
