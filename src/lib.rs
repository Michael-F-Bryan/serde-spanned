#![no_std]

mod de;
mod spanned;

pub use de::Deserializer;
pub use spanned::Spanned;

pub(crate) const NAME: &str = "$__private_serde_spanned";
pub(crate) const START: &str = "$__private_serde_spanned_start";
pub(crate) const END: &str = "$__private_serde_spanned_end";
pub(crate) const VALUE: &str = "$__private_serde_spanned_value";
pub(crate) const FIELDS: &[&str] = &[START, END, VALUE];

pub trait SpannedDeserializer<'de>: serde::Deserializer<'de> {
    /// The offset into the current stream.
    fn offset(&self) -> usize;
}
