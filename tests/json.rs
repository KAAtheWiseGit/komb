#![allow(unused)]

use std::collections::HashMap;
use std::convert::Infallible;

use komb::{
	combinator::{choice, delimited, fold, option},
	string::{char, literal, one_of0},
	PResult, Parser,
};

#[derive(Clone)]
enum Value {
	Null,
	Bool(bool),
	Number(f64),
	String(String),
	Array(Vec<Value>),
	Object(HashMap<String, Value>),
}

fn whitespace(input: &str) -> PResult<str, (), ()> {
	let out = one_of0(&[' ', '\n', '\r', '\t'])
		.map_out(|_| ())
		.map_err(|_| ())
		.parse(input);
	out
}

fn bool(input: &str) -> PResult<str, bool, ()> {
	delimited(
		whitespace,
		choice((
			literal("true").map_out(|_| true).map_err(|_| ()),
			literal("false").map_out(|_| false).map_err(|_| ()),
		)),
		whitespace,
	)
	.parse(input)
}

fn array(input: &str) -> PResult<str, Vec<Value>, ()> {
	let p = value.before(option(char(',')).map_err(|_| ()));
	let folded = fold(p, Vec::new(), |acc, value| acc.push(value));
	let delimited = delimited(
		char('[').map_err(|_| ()),
		folded,
		char(']').map_err(|_| ()),
	);

	delimited.parse(input)
}

fn value(input: &str) -> PResult<str, Value, ()> {
	todo!()
}

#[test]
fn test_bool() {
	let s = "   true  ";
	assert_eq!(Ok((true, "")), bool(s));
}
