#[derive(Debug, thiserror::Error)]
pub enum FromHexError {
    #[error("incompatible string length")]
    IncompatibleStringLength,
    #[error("error decoding: {0}")]
    Bincode(#[from] bincode::error::DecodeError),
    #[error("invalid integer value: {0}")]
    Int(#[from] std::num::ParseIntError),
}
#[derive(Debug, thiserror::Error)]
pub enum AsHexError {
    #[error("error enconding: {0}")]
    Bincode(#[from] bincode::error::EncodeError),
    #[error("formatting error: {0}")]
    Fmt(#[from] std::fmt::Error),
}

pub trait Hexable: Sized {
    /// #### Errors
    fn as_hex_string(&self) -> Result<String, AsHexError>;
    /// #### Errors
    fn from_hex(s: &str) -> Result<Self, FromHexError>;
}

impl<T: Sized + serde::Serialize + for<'de> serde::Deserialize<'de>> Hexable for T {
    fn as_hex_string(&self) -> Result<String, AsHexError> {
        use std::fmt::Write;
        let bc = bincode::serde::encode_to_vec(self, bincode::config::standard())?;
        let mut ret = String::with_capacity(bc.len() * 2);
        for b in bc {
            write!(ret, "{b:02X}")?;
        }
        Ok(ret)
    }
    fn from_hex(s: &str) -> Result<Self, FromHexError> {
        let bytes: Result<Vec<_>, _> = if s.len() % 2 == 0 {
            (0..s.len())
                .step_by(2)
                .filter_map(|i| s.get(i..i + 2))
                .map(|sub| u8::from_str_radix(sub, 16))
                .collect()
        } else {
            return Err(FromHexError::IncompatibleStringLength);
        };
        bincode::serde::decode_from_slice(&bytes?, bincode::config::standard())
            .map(|(r, _)| r)
            .map_err(Into::into)
    }
}
