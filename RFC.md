## Description

This is a request for comments on a mechanism for letting a
`serde::Deserializer` implementation communicate the source location of an item
with the end user without either party knowing the existence of the other.

This will be implemented by using a convention known by deserializer
implementations and some common items exposed from the `serde` crate. This has
been phrased as RFC instead of a simple GitHub issue as it would require
coordination with multiple high-profile projects across the Rust ecosystem.

## Motivation

Serde is often used to serialize and deserialize data that was originally
written manually by humans. Often, as well as needing to be syntactically
correct (e.g. valid JSON), there will be semantic invariants that must be
upheld.

When one of these invariants is broken there is no easy way to indicate *where*
the error was to the user. Often the best a library will be able to do is say
*"Error: `foo.bar` must be equal to the length of the `foo.baz[]` array"*,
leaving the actual location of the `foo.bar` element up to the end user.

By including a mechanism for emitting span locations, the error message could be
improved to *"Error: `foo.bar` on line 5 must be equal to the length of the
`foo.baz[]` array on line 42"*. For working with textual formats like TOML,
JSON, and YAML, crates like [`codespan`][codespan] could even be used to provide
diagnostics on par with `rustc`.

To avoid the situation where multiple deserializer crates create their own
incompatible mechanisms for retrieving span information, the mechanism should be
hoisted into a common crate. The end goal would be for the common infrastructure
to live in the `serde` crate, with initial experimentation done in a temporary
`serde-spanned` crate. Downstream serializers can then hide this functionality
behind a `spanned` feature flag until it is deemed suitably stable.

## Proposed Implementation

The proposed implementation for this RFC takes a lot of inspiration from similar
functionality in [the `toml` crate][toml].

The core feature of this protocol is the `Spanned` type, something which keeps
track of a `value` and its `start` and `end` locations.

```rust
struct Spanned<T, Loc=usize> {
    start: Loc,
    end: Loc,
    value: T,
}
```

The `Loc` type parameter could be anything that is deserializable, however for
maximum compatibility between implementations it is hoped that the most common
interpretation will be an integer byte offset into the underlying source (e.g. a
byte string or `std::io::Read`er).

To let a specific `serde::Deserializer` implementation know that it needs to
provide source implementation, the `Spanned<T>` type will ask to deserialize a
struct ([`serde::Deserializer::deserialize_struct()`][deserialize_struct])
using a well-known, uncommon struct name.

```rust
const NAME: &str = "$__serde_private_Spanned";
const START: &str = "$__serde_private_start";
const END: &str = "$__serde_private_end";
const VALUE: &str = "$__serde_private_value";

impl<'de, T, Loc> Deserialize<'de> for Spanned<T, Loc>
where
    T: Deserialize<'de>,
    Loc: Deserialize<'de>,
{
    fn deserialize<D>(de: D) -> Result<Spanned<T, Loc>, D::Error>
    where D: Deserializer<'de>,
    {
        let visitor = ...;
        de.deserialize_struct(NAME, &[START, END, VALUE], visitor)
    }
}
```

From there, the `Deserializer` would perform a check against the name and fields
and invoke `visit_map()` with an appropriate `MapAccess` implementation.

```rust
impl<'de> Deserializer<'de> for MyDeserializer<'de> {
    ...

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        if name == spanned::NAME && fields == [spanned::START, spanned::END, spanned::VALUE] {
            return visitor.visit_map(MapAccessWithSpan { ... });
        }

        self.deserialize_any(visitor)
    }
```

(see the [corresponding implementation in `toml-rs`][spanned-impl] for more)

## See Also

- *Support emitting spans when deserializing structures* - [`alexcrichton/toml-rs#236`](https://github.com/alexcrichton/toml-rs/issues/236)
- *Store span information/line & column numbers in deserialization* - [`serde-rs/serde#1811`](https://github.com/serde-rs/serde/issues/1811)
- *Get line and column number on valid JSON?* - [`serde-rs/json#637`](https://github.com/serde-rs/json/issues/637)
-  *Support emitting spans when deserializing structures* - [`dtolnay/serde-yaml#181`](https://github.com/dtolnay/serde-yaml/issues/181)

[deserialize_struct]: https://docs.rs/serde/1.0.125/serde/trait.Deserializer.html#tymethod.deserialize_struct
[toml]: https://crates.io/crates/toml
[codespan]: https://crates.io/crates/codespan
[spanned-impl]: https://github.com/alexcrichton/toml-rs/blob/c2a069f5debe8e361d8f0d4e4cd179aa6b9e4b47/src/spanned.rs#L7-L10
