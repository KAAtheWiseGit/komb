# Komb

A primitive parser combinator library, inspired by [nom] and [winnow].
Unlike the latter, Komb operates on borrowed slices (i.e. `&str` or
`[u8]`) instead of iterators.  This slightly simplifies zero-copy
parsing and parser creation at the cost of lesser flexibility and
inability to do streaming.


[nom]: https://lib.rs/crates/nom
[winnow]: https://lib.rs/crates/winnow
[`map_err`]: khttps://docs.rs/komb/*/komb/trait.Parser.html#method.map_err
