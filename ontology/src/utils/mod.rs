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

use crate::domain::declarations::symbols::ArgumentSpec;
pub mod awaitable;
pub mod regex;
pub mod time;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct SourcePos {
    pub line: u32,
    pub col: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct SourceRange {
    pub start: SourcePos,
    pub end: SourcePos,
}
impl SourceRange {
    pub const DEFAULT: Self = Self {
        start: SourcePos { line: 0, col: 0 },
        end: SourcePos { line: 0, col: 0 },
    };
    #[inline]
    #[must_use]
    pub const fn is_defined(&self) -> bool {
        // in lieu of const traits
        self.start.line as usize
            + self.start.col as usize
            + self.end.line as usize
            + self.end.col as usize
            != 0
    }
    #[inline]
    #[must_use]
    pub const fn is_position(&self) -> bool {
        self.start.line == self.end.line && self.start.col == self.end.col
    }
}

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
impl<T, const N: usize> Default for SVec<T, N> {
    #[inline]
    fn default() -> Self {
        Self(SmallVec::default())
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

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Permutation(Box<[u8]>);
impl Permutation {
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// ### Errors
    /// (obviously)
    pub fn parse(spec: &ArgumentSpec, s: &str) -> Result<Self, ()> {
        use std::str::FromStr;
        let mut ret = s
            .split(',')
            .map(|s| u8::from_str(s).map_or(Err(()), |u| if u > 0 { Ok(u - 1) } else { Err(()) }))
            .collect::<Result<Vec<u8>, ()>>()?;
        // completeness + no duplicates:
        for j in 0..spec.num() {
            let num = bytecount::naive_count_32(&ret, j);
            if num > 1 {
                return Err(());
            }
            if num == 0 {
                ret.push(j);
            }
        }
        if ret.len() == spec.num() as usize {
            Ok(Self(ret.into_boxed_slice()))
        } else {
            Err(())
        }
    }

    /// ### Errors
    /// (obviously)
    pub fn apply<T: Clone + std::fmt::Debug>(&self, arguments: &[T]) -> Result<Vec<T>, ()> {
        if arguments.len() != self.0.len() {
            return Err(());
        }
        Ok(unsafe { self.apply_unchecked(arguments) })
    }

    /// ### Safety
    /// `arguments.len() == self.len()`
    pub unsafe fn apply_unchecked<T: Clone + std::fmt::Debug>(&self, arguments: &[T]) -> Vec<T> {
        let mut ret = Vec::with_capacity(arguments.len());
        for i in &self.0 {
            ret.push(arguments[*i as usize].clone());
        }
        ret
    }

    /// ### Errors
    /// (obviously)
    pub fn revert<T: Clone>(&self, arguments: &[T]) -> Result<Vec<T>, ()> {
        if arguments.len() != self.0.len() {
            return Err(());
        }
        Ok(unsafe { self.revert_unchecked(arguments) })
    }

    /// ### Safety
    /// `arguments.len() == self.len()`
    pub unsafe fn revert_unchecked<T: Clone>(&self, arguments: &[T]) -> Vec<T> {
        let mut ret = Vec::with_capacity(arguments.len());
        for _ in 0..arguments.len() {
            ret.push(std::mem::MaybeUninit::uninit());
        }
        for (idx, v) in self.0.iter().zip(arguments.iter()) {
            ret[*idx as usize].write(v.clone());
        }
        // SAFETY:
        // - by construction, `self.0` holds every index `0..self.0.len()` exactly once
        // - `self.len() == arguments.len()`
        unsafe { std::mem::transmute(ret) }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn validate(arr: Box<[u8]>) -> Option<Self> {
        if arr.len() > u8::MAX as usize {
            return None;
        }
        let mut max = 0;
        for j in 0..arr.len() as u8 {
            let num = bytecount::naive_count_32(&arr, j);
            if num > 1 {
                return None;
            }
            max = max.max(arr[j as usize]);
        }
        if max as usize == arr.len() - 1 {
            Some(Self(arr))
        } else {
            None
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    impl serde::Serialize for super::Permutation {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.serialize(serializer)
        }
    }
    impl<'de> serde::Deserialize<'de> for super::Permutation {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Self::validate(Box::<[u8]>::deserialize(deserializer)?)
                .ok_or_else(|| serde::de::Error::custom("invalid"))
        }
    }
    impl bincode::Encode for super::Permutation {
        fn encode<E: bincode::enc::Encoder>(
            &self,
            encoder: &mut E,
        ) -> Result<(), bincode::error::EncodeError> {
            self.0.encode(encoder)
        }
    }
    impl<Context> bincode::Decode<Context> for super::Permutation {
        fn decode<D: bincode::de::Decoder<Context = Context>>(
            decoder: &mut D,
        ) -> Result<Self, bincode::error::DecodeError> {
            Self::validate(Box::<[u8]>::decode(decoder)?)
                .ok_or(bincode::error::DecodeError::Other("invalid"))
        }
    }
    impl<'de, Context> bincode::BorrowDecode<'de, Context> for super::Permutation {
        fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
            decoder: &mut D,
        ) -> Result<Self, bincode::error::DecodeError> {
            Self::validate(Box::<[u8]>::borrow_decode(decoder)?)
                .ok_or(bincode::error::DecodeError::Other("invalid"))
        }
    }
}

#[cfg(feature = "serde-lite")]
mod serde_lite_impl {
    impl serde_lite::Serialize for super::Permutation {
        #[inline]
        fn serialize(&self) -> Result<serde_lite::Intermediate, serde_lite::Error> {
            self.0.serialize()
        }
    }
    impl serde_lite::Deserialize for super::Permutation {
        fn deserialize(val: &serde_lite::Intermediate) -> Result<Self, serde_lite::Error>
        where
            Self: Sized,
        {
            Self::validate(Box::<[u8]>::deserialize(val)?)
                .ok_or_else(|| serde_lite::Error::custom("invalid"))
        }
    }
}
/*
#[cfg_attr(
    feature = "serde",
    derive(bincode::Decode, bincode::Encode)
)]
 */
