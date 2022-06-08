pub trait Animation {
    type Frame;

    fn advance_frames(&mut self, frames: usize);
    fn current_frame(&self) -> Self::Frame;
    fn is_over(&self) -> bool;

    fn zip<A: Animation>(self, right: A) -> AnimationZip<Self, A>
    where
        Self: Sized,
    {
        AnimationZip::new(self, right)
    }
}

pub struct ConstantAnimation<T, F: Fn() -> T>(F);

impl<T, F: Fn() -> T> Animation for ConstantAnimation<T, F> {
    type Frame = T;

    fn advance_frames(&mut self, _frames: usize) {}

    fn current_frame(&self) -> T {
        self.0()
    }

    fn is_over(&self) -> bool {
        true
    }
}

pub struct Animator<T> {
    animations: Vec<Box<dyn Animation<Frame = T>>>,
}

impl<T> Animator<T> {
    pub fn new(animations: Vec<Box<dyn Animation<Frame = T>>>) -> Self {
        Animator { animations }
    }
}

impl<T> Animation for Animator<T> {
    type Frame = Vec<T>;

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

pub struct FloatAnimator<A: Animation + ?Sized> {
    begin_at: f64,
    elapsed_frames: usize,
    pub animation: Box<A>,
}

impl<A: Animation + ?Sized> FloatAnimator<A> {
    pub fn new(animation: Box<A>) -> Self {
        let now = js_sys::Date::now();
        FloatAnimator {
            begin_at: now,
            elapsed_frames: 0,
            animation,
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

    pub fn frame(&self) -> A::Frame {
        self.animation.current_frame()
    }
}

pub struct EndlessAnimator<T> {
    animations: Vec<Box<dyn Animation<Frame = T>>>,
}

impl<T> EndlessAnimator<T> {
    pub fn new(animations: Vec<Box<dyn Animation<Frame = T>>>) -> Self {
        EndlessAnimator {
            animations: animations.into_iter().rev().collect(),
        }
    }

    pub fn push(&mut self, animation: impl Animation<Frame = T> + 'static) {
        self.animations.push(Box::new(animation));
    }
}

impl<T> Animation for EndlessAnimator<T> {
    type Frame = Vec<T>;

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

pub struct AnimationZip<A1: Animation, A2: Animation> {
    animation_1: A1,
    animation_2: A2,
}

impl<A1: Animation, A2: Animation> AnimationZip<A1, A2> {
    fn new(animation_1: A1, animation_2: A2) -> Self {
        AnimationZip {
            animation_1,
            animation_2,
        }
    }
}

impl<A1: Animation, A2: Animation> Animation for AnimationZip<A1, A2> {
    type Frame = (A1::Frame, A2::Frame);
    fn advance_frames(&mut self, frames: usize) {
        self.animation_1.advance_frames(frames);
        self.animation_2.advance_frames(frames);
    }

    fn current_frame(&self) -> Self::Frame {
        (
            self.animation_1.current_frame(),
            self.animation_2.current_frame(),
        )
    }

    fn is_over(&self) -> bool {
        self.animation_1.is_over() && self.animation_2.is_over()
    }
}

pub struct AnimationStream<T> {
    animations: Vec<Box<dyn Animation<Frame = T>>>,
}

impl<T> AnimationStream<T> {
    pub fn new() -> Self {
        AnimationStream {
            animations: Vec::new(),
        }
    }

    pub fn push(&mut self, animation: impl Animation<Frame = T> + 'static) {
        if !animation.is_over() {
            self.animations.push(Box::new(animation));
        }
    }
}

impl<T> Animation for AnimationStream<T> {
    type Frame = Option<T>;

    fn advance_frames(&mut self, frames: usize) {
        for _ in 0..frames {
            if let Some(animation) = self.animations.first_mut() {
                animation.advance_frames(1);
            } else {
                break;
            }

            if self.animations[0].is_over() {
                self.animations.remove(0);
            }
        }
    }

    fn current_frame(&self) -> Option<T> {
        self.animations.first().map(|x| x.current_frame())
    }

    fn is_over(&self) -> bool {
        self.animations.is_empty()
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

    impl Animation for Double {
        type Frame = usize;

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
