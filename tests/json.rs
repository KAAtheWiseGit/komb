#![allow(unused)]

use std::collections::HashMap;
use std::convert::Infallible;

use komb::{string, PResult, Parser};

enum Value {
	Null,
	Bool(bool),
	Number(f64),
	String(String),
	Array(Vec<Value>),
	Object(HashMap<String, Value>),
}

fn whitespace(input: &str) -> PResult<str, (), Infallible> {
	string::one_of0(&[' ', '\n', '\r', '\t'])
		.parse(input)
		.map(|(_, rest)| ((), rest))
}

fn value(input: &str) -> PResult<str, Value, ()> {
	todo!()
}
