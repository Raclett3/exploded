use std::marker::PhantomData;

pub trait Animation<T> {
    fn advance_frames(&mut self, frames: usize);
    fn current_frame(&self) -> T;
    fn is_over(&self) -> bool;

    fn chain<A: Animation<T>>(self, right: A) -> AnimationChain<T, Self, A>
    where
        Self: Sized,
    {
        AnimationChain::new(self, right)
    }

    fn zip<U, A: Animation<U>>(self, right: A) -> AnimationZip<T, U, Self, A>
    where
        Self: Sized,
    {
        AnimationZip::new(self, right)
    }
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

pub struct AnimationChain<T, A1: Animation<T>, A2: Animation<T>> {
    animation_1: A1,
    animation_2: A2,
    phantom: PhantomData<T>,
}

impl<T, A1: Animation<T>, A2: Animation<T>> AnimationChain<T, A1, A2> {
    fn new(animation_1: A1, animation_2: A2) -> Self {
        AnimationChain {
            animation_1,
            animation_2,
            phantom: PhantomData,
        }
    }
}

impl<T, A1: Animation<T>, A2: Animation<T>> Animation<T> for AnimationChain<T, A1, A2> {
    fn advance_frames(&mut self, frames: usize) {
        for _ in 0..frames {
            if self.animation_1.is_over() {
                self.animation_2.advance_frames(1)
            } else {
                self.animation_1.advance_frames(1)
            }
        }
    }

    fn current_frame(&self) -> T {
        if self.animation_1.is_over() {
            self.animation_2.current_frame()
        } else {
            self.animation_1.current_frame()
        }
    }

    fn is_over(&self) -> bool {
        self.animation_1.is_over() && self.animation_2.is_over()
    }
}

pub struct FloatAnimator<T, A: Animation<T> + ?Sized> {
    begin_at: f64,
    elapsed_frames: usize,
    pub animation: Box<A>,
    phantom: PhantomData<T>,
}

impl<T, A: Animation<T> + ?Sized> FloatAnimator<T, A> {
    pub fn new(animation: Box<A>) -> Self {
        let now = js_sys::Date::now();
        FloatAnimator {
            begin_at: now,
            elapsed_frames: 0,
            animation,
            phantom: PhantomData,
        }
    }

    pub fn animate(&mut self) {
        let now = js_sys::Date::now();
        let elapsed = now - self.begin_at;
        let frames = (elapsed / 1000.0 * 60.0).floor() as usize;
        let frame_delta = frames - self.elapsed_frames;
        self.elapsed_frames = frames;

        if frame_delta > 0 {
            self.animation.advance_frames(frame_delta);
        }
    }

    pub fn frame(&self) -> T {
        self.animation.current_frame()
    }

    pub fn is_over(&self) -> bool {
        self.animation.is_over()
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

pub struct AnimationZip<T, U, A1: Animation<T>, A2: Animation<U>> {
    animation_1: A1,
    animation_2: A2,
    phantom: PhantomData<(T, U)>,
}

impl<T, U, A1: Animation<T>, A2: Animation<U>> AnimationZip<T, U, A1, A2> {
    fn new(animation_1: A1, animation_2: A2) -> Self {
        AnimationZip {
            animation_1,
            animation_2,
            phantom: PhantomData,
        }
    }
}

impl<T, U, A1: Animation<T>, A2: Animation<U>> Animation<(T, U)> for AnimationZip<T, U, A1, A2> {
    fn advance_frames(&mut self, frames: usize) {
        self.animation_1.advance_frames(frames);
        self.animation_2.advance_frames(frames);
    }

    fn current_frame(&self) -> (T, U) {
        (
            self.animation_1.current_frame(),
            self.animation_2.current_frame(),
        )
    }

    fn is_over(&self) -> bool {
        self.animation_1.is_over() && self.animation_2.is_over()
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
