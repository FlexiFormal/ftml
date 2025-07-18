use crate::errors::SegmentParseError;
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;

#[impl_tools::autoimpl(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Str<A>(strumbra::SharedString, PhantomData<A>);
crate::debugdisplay!(Str<A>);
impl<A> Str<A> {
    #[inline]
    fn new(s: &str) -> Result<Self, SegmentParseError> {
        if let Some(i) = s.find(super::errors::ILLEGAL_CHARS) {
            // SAFETY: i is defined, so s[i..].chars().next() is defined
            return unsafe {
                Err(SegmentParseError::IllegalChar(
                    s[i..].chars().next().unwrap_unchecked(),
                ))
            };
        }
        Ok(Self(s.try_into()?, PhantomData))
    }
}

impl<A> std::fmt::Display for Str<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<A> Borrow<str> for Str<A> {
    #[inline]
    fn borrow(&self) -> &str {
        &self.0
    }
}
impl<A> std::ops::Deref for Str<A> {
    type Target = str;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<A> FromStr for Str<A> {
    type Err = SegmentParseError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[impl_tools::autoimpl(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonEmptyStr<A>(Str<A>);
crate::debugdisplay!(NonEmptyStr<A>);

impl<A> NonEmptyStr<A> {
    #[inline]
    pub(crate) unsafe fn new_from_nonempty(s: Str<A>) -> Self {
        Self(s)
    }

    pub fn new(s: &str) -> Result<Self, SegmentParseError> {
        if s.is_empty() {
            Err(SegmentParseError::Empty)
        } else {
            Ok(Self(Str::new(s)?))
        }
    }
    pub fn new_with_sep<const SEP: char>(s: &str) -> Result<Self, SegmentParseError> {
        if s.is_empty() || s.split(SEP).any(str::is_empty) {
            return Err(SegmentParseError::Empty);
        }
        Ok(Self(Str::new(s)?))
    }

    #[inline]
    pub fn segmented<const SEP: char>(&self) -> std::str::Split<'_, char> {
        self.split(SEP)
    }

    #[inline]
    pub fn first_of<const SEP: char>(&self) -> &str {
        // SAFETY: NonEmptyStr guarantees the string is non-empty,
        // so split() will always yield at least one element
        unsafe { self.split(SEP).next().unwrap_unchecked() }
    }
    #[inline]
    pub fn last_of<const SEP: char>(&self) -> &str {
        // SAFETY: NonEmptyStr guarantees the string is non-empty,
        // so split() will always yield at least one element
        unsafe { self.split(SEP).next_back().unwrap_unchecked() }
    }

    pub fn up<const SEP: char>(&self) -> Option<Self> {
        if let Some((s, _)) = self.rsplit_once(SEP) {
            // SAFETY: rsplit_once with a non-empty string that was validated
            // to have no empty segments guarantees s is non-empty
            // and length <= u32::MAX
            Some(Self(Str(
                unsafe { s.try_into().unwrap_unchecked() },
                PhantomData,
            )))
        } else {
            None
        }
    }
}

impl<A> std::fmt::Display for NonEmptyStr<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<A> std::ops::Deref for NonEmptyStr<A> {
    type Target = Str<A>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<A> FromStr for NonEmptyStr<A> {
    type Err = SegmentParseError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}
