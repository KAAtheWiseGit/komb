pub mod str;

pub type PResult<I, O, E> = Result<(I, O), E>;

pub trait Parser<I, O, E>: Fn(I) -> PResult<I, O, E> {}

impl<I, O, E, T> Parser<I, O, E> for T where T: Fn(I) -> PResult<I, O, E> {}
