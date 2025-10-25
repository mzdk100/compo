use {
    crate::runtime::{Cancellable, Runtime},
    std::rc::{Rc, Weak},
};

pub trait Component<'a> {
    fn new(rt: Weak<Runtime<'a, ()>>) -> Self;

    fn get_rt(&self) -> Weak<Runtime<'a, ()>>;

    fn spawn<Fut>(&self, fut: Fut) -> Cancellable
    where
        Fut: Future<Output=()> + 'a,
    {
        if let Some(rt) = self.get_rt().upgrade() {
            rt.spawn(fut)
        } else {
            Default::default()
        }
    }

    fn update(self: &Rc<Self>);
}
