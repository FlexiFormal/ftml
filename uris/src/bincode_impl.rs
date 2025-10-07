pub use crate::prelude::*;
use crate::utils::{
    NonEmptyStr,
    errors::{SegmentParseError, UriParseError},
};

impl<Context> bincode::Decode<Context> for BaseUri {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        String::decode(decoder)?
            .parse()
            .map_err(|e: UriParseError| bincode::error::DecodeError::OtherString(e.to_string()))
    }
}
impl<'de, Context> bincode::BorrowDecode<'de, Context> for BaseUri {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        String::borrow_decode(decoder)?
            .parse()
            .map_err(|e: UriParseError| bincode::error::DecodeError::OtherString(e.to_string()))
    }
}
impl bincode::Encode for BaseUri {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.as_str().encode(encoder)
    }
}
#[cfg(feature = "interned")]
impl<Store: crate::utils::interned::InternStore, Context> bincode::Decode<Context>
    for NonEmptyStr<Store>
{
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Self::new(&String::decode(decoder)?)
            .map_err(|e: SegmentParseError| bincode::error::DecodeError::OtherString(e.to_string()))
    }
}
#[cfg(feature = "interned")]
impl<'de, Store: crate::utils::interned::InternStore, Context> bincode::BorrowDecode<'de, Context>
    for NonEmptyStr<Store>
{
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Self::new(&String::borrow_decode(decoder)?)
            .map_err(|e: SegmentParseError| bincode::error::DecodeError::OtherString(e.to_string()))
    }
}

#[cfg(not(feature = "interned"))]
impl<Store, Context> bincode::Decode<Context> for NonEmptyStr<Store> {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Self::new(&String::decode(decoder)?)
            .map_err(|e: SegmentParseError| bincode::error::DecodeError::OtherString(e.to_string()))
    }
}
#[cfg(not(feature = "interned"))]
impl<'de, Store, Context> bincode::BorrowDecode<'de, Context> for NonEmptyStr<Store> {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Self::new(&String::borrow_decode(decoder)?)
            .map_err(|e: SegmentParseError| bincode::error::DecodeError::OtherString(e.to_string()))
    }
}

#[cfg(feature = "interned")]
impl<Store: crate::utils::interned::InternStore> bincode::Encode for NonEmptyStr<Store> {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        (**self).encode(encoder)
    }
}

#[cfg(not(feature = "interned"))]
impl<Store> bincode::Encode for NonEmptyStr<Store> {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        (**self).encode(encoder)
    }
}
