pub mod errors;
pub mod infix;
pub mod macros;

#[cfg(feature = "interned")]
pub mod interned;
#[cfg(feature = "interned")]
pub type NonEmptyStr<Store> = interned::NonEmptyInternedStr<Store>;

#[cfg(not(feature = "interned"))]
mod uninterned;
#[cfg(not(feature = "interned"))]
pub use uninterned::NonEmptyStr;

macros::intern!(IDS IdStr = IdStore:NonEmptyInternedStr|NonEmptyStr @ crate::aux::interned::ID_MAX);

#[cfg(feature = "typescript")]
#[::wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_CUSTOM: &str = "export type Id = string;";

/// An arbitrary Identifier; not part of a URI,
/// but similarly implemented, e.g. (if `interned`-feature is active)
/// interned, equality-checkable etc.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(
        serde_with::DeserializeFromStr,
        serde_with::SerializeDisplay,
        bincode::Decode,
        bincode::Encode
    )
)]
pub struct Id(IdStr);
impl Id {
    /// Creates a new [`Id`] from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string:
    /// - Is empty
    /// - Contains empty segments (consecutive forward slashes)
    /// - Contains illegal characters (backslash, curly braces)
    /// - Exceeds the maximum length supported by the interning system (`u32::MAX`)
    #[inline]
    pub fn new(s: &str) -> Result<Self, errors::SegmentParseError> {
        IdStr::new(s).map(Self)
    }
}
impl std::str::FromStr for Id {
    type Err = errors::SegmentParseError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}
impl std::fmt::Display for Id {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl AsRef<str> for Id {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
crate::debugdisplay!(Id);
