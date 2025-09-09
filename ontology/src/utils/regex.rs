#[cfg(all(target_family = "wasm", feature = "wasm"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Regex(String);

#[cfg(not(all(target_family = "wasm", feature = "wasm")))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Regex(regex::Regex);

impl std::fmt::Display for Regex {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

macro_rules! switch {
    ($rs:block $js:block) => {{
        #[cfg(not(all(target_family = "wasm", feature = "wasm")))]
        $rs
        #[cfg(all(target_family = "wasm", feature = "wasm"))]
        $js
    }}
}

#[derive(Debug, thiserror::Error)]
#[error("invalid regex string")]
pub struct InvalidRegex;

impl Regex {
    #[must_use]
    pub fn is_match(&self, text: &str) -> bool {
        switch!({
            self.0.is_match(text)
        }{
            js_regexp::RegExp::new(&self.0, js_regexp::flags!("")).expect("illegal regex")
                .exec(text)
                .is_some()
        })
    }
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        switch!({self.0.as_str()}{&self.0})
    }
    /// ### Errors
    pub fn new(s: &str) -> Result<Self, InvalidRegex> {
        switch!({
            ::regex::Regex::new(s).map(Self).map_err(|_| InvalidRegex)
        }{
            // https://docs.rs/js-regexp/0.2.1/js_regexp/struct.FlagSets.html
            js_regexp::RegExp::new(s, js_regexp::flags!(""))
                .map(|_| Self(s.to_string()))
                .map_err(|_| InvalidRegex)
        })
    }
}

#[cfg(feature = "serde")]
mod regex_serde {
    use super::Regex;
    impl serde::Serialize for Regex {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(self.as_str())
        }
    }

    impl<'de> serde::Deserialize<'de> for Regex {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Self::new(&s).map_err(serde::de::Error::custom)
        }
    }
    impl bincode::Encode for Regex {
        fn encode<E: bincode::enc::Encoder>(
            &self,
            encoder: &mut E,
        ) -> Result<(), bincode::error::EncodeError> {
            self.as_str().encode(encoder)
        }
    }
    impl<'de, Context> bincode::BorrowDecode<'de, Context> for Regex {
        fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
            decoder: &mut D,
        ) -> Result<Self, bincode::error::DecodeError> {
            Self::new(&String::borrow_decode(decoder)?)
                .map_err(|e| bincode::error::DecodeError::OtherString(e.to_string()))
        }
    }
    impl<Context> bincode::Decode<Context> for Regex {
        fn decode<D: bincode::de::Decoder<Context = Context>>(
            decoder: &mut D,
        ) -> Result<Self, bincode::error::DecodeError> {
            Self::new(&String::decode(decoder)?)
                .map_err(|e| bincode::error::DecodeError::OtherString(e.to_string()))
        }
    }
}
