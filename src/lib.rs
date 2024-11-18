#![allow(unused_imports)]

pub mod combinator;
pub mod str;

pub type PResult<I, O, E> = Result<(I, O), E>;
