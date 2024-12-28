# Komb

A primitive parser combinator library, inspired by [nom] and [winnow].
Unlike the latter, Komb operates on borrowed slices (i.e. `&str` or
`[u8]`) instead of iterators.  This slightly simplifies zero-copy
parsing and parser creation at the cost of lesser flexibility and
inability to do streaming.

Since Komb can't do streaming, it's fully generic over the error type.
This, combined with my design decisions regarding the primary `Parser`
trait, also means that the compiler error messages are abysmal.  If
compilation fails with several pages of error input, it's probably the
mismatch in error types, which can be fixed with [`map_err`].


[nom]: https://lib.rs/crates/nom
[winnow]: https://lib.rs/crates/winnow
[`map_err`]: khttps://docs.rs/komb/*/komb/trait.Parser.html#method.map_err
