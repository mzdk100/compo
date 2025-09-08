use {crate::runtime::Runtime, std::rc::Weak};

pub trait Component<'a> {
    fn new(rt: Weak<Runtime<'a, ()>>) -> Self;

    fn get_rt(&self) -> Weak<Runtime<'a, ()>>;

    fn spawn<Fut>(&self, fut: Fut)
    where
        Fut: Future<Output=()> + 'a,
    {
        if let Some(rt) = self.get_rt().upgrade() {
            rt.spawn(fut);
        }
    }
}
