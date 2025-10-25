mod shared_arc;
use std::ops::{Deref, DerefMut};

use ordered_float::OrderedFloat;
pub use shared_arc::SharedArc;
mod tree;
use smallvec::SmallVec;
pub use tree::*;
#[cfg(feature = "serde")]
mod hexable;
#[cfg(feature = "serde")]
pub use hexable::*;
mod css;
pub use css::*;
pub mod awaitable;
pub mod regex;
pub mod time;

/// Wrapper for [`OrderedFloat`] for serialization reasons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "typescript",
    tsify(into_wasm_abi, from_wasm_abi, type = "number")
)]
pub struct Float(#[cfg_attr(feature = "serde", bincode(with_serde))] OrderedFloat<f32>);
impl Deref for Float {
    type Target = f32;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Float {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<f32> for Float {
    #[inline]
    fn from(value: f32) -> Self {
        Self(OrderedFloat(value))
    }
}
impl From<Float> for f32 {
    #[inline]
    fn from(value: Float) -> Self {
        value.0.into()
    }
}

#[cfg(feature = "serde-lite")]
impl serde_lite::Serialize for Float {
    #[inline]
    fn serialize(&self) -> Result<serde_lite::Intermediate, serde_lite::Error> {
        self.0.deref().serialize()
    }
}

#[cfg(feature = "serde-lite")]
impl serde_lite::Deserialize for Float {
    #[inline]
    fn deserialize(val: &serde_lite::Intermediate) -> Result<Self, serde_lite::Error>
    where
        Self: Sized,
    {
        Ok(Self(OrderedFloat(f32::deserialize(val)?)))
    }
}

/// Wrapper for [`SmallVec`] for serialization reasons
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SVec<T, const N: usize>(pub SmallVec<T, N>);
impl<T, const N: usize> Deref for SVec<T, N> {
    type Target = SmallVec<T, N>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T, const N: usize> DerefMut for SVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T, const N: usize> FromIterator<T> for SVec<T, N> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(SmallVec::from_iter(iter))
    }
}

#[cfg(feature = "serde-lite")]
impl<T: serde_lite::Serialize, const N: usize> serde_lite::Serialize for SVec<T, N> {
    #[inline]
    fn serialize(&self) -> Result<serde_lite::Intermediate, serde_lite::Error> {
        self.0.as_slice().serialize()
    }
}

#[cfg(feature = "serde-lite")]
impl<T: serde_lite::Deserialize, const N: usize> serde_lite::Deserialize for SVec<T, N> {
    fn deserialize(val: &serde_lite::Intermediate) -> Result<Self, serde_lite::Error>
    where
        Self: Sized,
    {
        if let serde_lite::Intermediate::Array(v) = val {
            let mut ret = SmallVec::with_capacity(v.len());
            for e in v {
                ret.push(T::deserialize(e)?);
            }
            Ok(Self(ret))
        } else {
            Err(serde_lite::Error::custom("array expected"))
        }
    }
}
