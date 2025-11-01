macro_rules! ts {
    ($name:ident$(<$($a:ident$(:$guard:ident)?),+>)? = $c:ident) => {
        #[cfg(feature = "typescript")]
        #[::wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
        const $c: &str = concat!("export type ", stringify!($name), " = string;");
        #[cfg(feature = "typescript")]
        impl$(<$($a$(:$guard)?),+>)? ::wasm_bindgen::convert::FromWasmAbi for $name$(<$($a),+>)? {
            type Abi = <String as ::wasm_bindgen::convert::FromWasmAbi>::Abi;
            unsafe fn from_abi(js: Self::Abi) -> Self {
                unsafe {
                    use ::wasm_bindgen::UnwrapThrowExt;
                    let s = <String as ::wasm_bindgen::convert::FromWasmAbi>::from_abi(js);
                    s.parse().unwrap_throw()
                }
            }
        }
        #[cfg(feature = "typescript")]
        impl$(<$($a$(:$guard)?),+>)? ::wasm_bindgen::convert::IntoWasmAbi for $name$(<$($a),+>)? {
            type Abi = <String as ::wasm_bindgen::convert::IntoWasmAbi>::Abi;
            fn into_abi(self) -> Self::Abi {
                <String as ::wasm_bindgen::convert::IntoWasmAbi>::into_abi(self.to_string())
            }
        }
        #[cfg(feature = "typescript")]
        impl<'ts$(,$($a$(:$guard)?),+)?> ::wasm_bindgen::convert::IntoWasmAbi for &'ts $name$(<$($a),+>)? {
            type Abi = <String as ::wasm_bindgen::convert::IntoWasmAbi>::Abi;
            fn into_abi(self) -> Self::Abi {
                <String as ::wasm_bindgen::convert::IntoWasmAbi>::into_abi(self.to_string())
            }
        }

        #[cfg(feature = "typescript")]
        impl$(<$($a$(:$guard)?),+>)? ::wasm_bindgen::describe::WasmDescribe for $name$(<$($a),+>)? {
            fn describe() {
                <String as ::wasm_bindgen::describe::WasmDescribe>::describe();
            }
        }

        #[cfg(feature = "js")]
        impl$(<$($a$(:$guard)?),+>)? ::wasm_bindgen::convert::TryFromJsValue for $name$(<$($a),+>)? {
            fn try_from_js_value(value: wasm_bindgen::JsValue) -> Result<Self, wasm_bindgen::JsValue> {
                if let Some(s) = value.as_string() {
                    s.parse().map_err(|_| value)
                } else {
                    Err(value)
                }
            }
            fn try_from_js_value_ref(value: &wasm_bindgen::JsValue) -> Option<Self> {
                value.as_string()?.parse().ok()
            }
        }

        #[cfg(feature = "js")]
        impl$(<$($a$(:$guard)?),+>)? ::ftml_js_utils::conversion::FromJs for $name$(<$($a),+>)? {
            type Error = $crate::errors::UriParseError;
            fn from_js(value: ::wasm_bindgen::JsValue) -> Result<Self,Self::Error> {
                if let Some(s) = value.as_string() {
                    s.parse().map_err($crate::errors::UriParseError::from)
                } else {
                    Err($crate::errors::UriParseError::NotAString)
                }
            }
        }

        #[cfg(feature = "js")]
        impl$(<$($a$(:$guard)?),+>)? ::ftml_js_utils::conversion::ToJs for $name$(<$($a),+>)? {
            type Error = ::std::convert::Infallible;
            fn to_js(&self) -> Result<::wasm_bindgen::JsValue, Self::Error> {
                Ok(::wasm_bindgen::JsValue::from_str(&self.to_string()))
            }
        }

        #[cfg(feature = "js")]
        impl$(<$($a$(:$guard)?),+>)? From<$name$(<$($a),+>)?> for wasm_bindgen::JsValue {
            fn from(uri: $name$(<$($a),+>)?) -> Self {
                Self::from_str(&uri.to_string())
            }
        }
    };
    ($name:ident$(<$($a:ident$(:$guard:ident)?),+>)?) => {
        $crate::ts!($name$(<$($a$(:$guard)?),+>)? = TS);
    };
}
macro_rules! tests {
    ($($(#[$meta:meta])* $name:ident $(($argn:ident:$argtp:ty))? $code:block);*) => {
        #[cfg(test)]
        mod tests {
            #![allow(unused_imports)]
            use super::*;
            use crate::trace;
            use rstest::{rstest,fixture};
            $(
                #[rstest]
                #[allow(unused_variables)]
                $(#[$meta])*
                fn $name(trace:()$($(,$argn:$argtp)*)?) $code
            )*

        }
    };
}

macro_rules! debugdisplay {
    ($s:ident$(<$($a:ident$(:$guard:ident)?),+>)?) => {
        impl$(<$($a$(:$guard)?),+>)? std::fmt::Debug for $s$(<$($a),+>)? {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(self, f)
            }
        }
    };
}

#[allow(clippy::redundant_pub_crate)]
macro_rules! intern {
    ($static:ident = $store:ident:$str_type:ident @ $num:expr ) => {
        #[cfg(feature = "interned")]//, not(feature = "api")))]
        #[allow(clippy::redundant_pub_crate)]
        pub(crate) static $static: std::sync::LazyLock<crate::utils::interned::InternMap> =
            std::sync::LazyLock::new(crate::utils::interned::InternMap::default);
        #[cfg(feature = "interned")]//, not(feature = "api")))]
        #[inline]
        unsafe fn get_ids() -> &'static crate::utils::interned::InternMap {
            &$static
        }
        /*#[cfg(all(feature = "interned", feature = "api"))]
        unsafe extern "C" {
            fn get_ids() -> &'static crate::utils::interned::InternMap;
        }*/

        #[allow(clippy::redundant_pub_crate)]
        pub(crate) struct $store;
        #[cfg(feature = "interned")]
        impl crate::utils::interned::InternStore for $store {
            const LIMIT: usize = $num;
            #[inline]
            fn get() -> &'static crate::utils::interned::InternMap {
                unsafe { get_ids() }
            }
        }
    };
    ($static:ident $type:ident = $store:ident:$str_type:ident|$noninterned:ident @ $num:expr ) => {
        crate::utils::macros::intern!($static = $store:$str_type @ $num);
        #[cfg(feature = "interned")]
        #[allow(clippy::redundant_pub_crate)]
        pub(crate) type $type = crate::utils::interned::$str_type<$store>;
        #[cfg(not(feature = "interned"))]
        #[allow(clippy::redundant_pub_crate)]
        pub(crate) type $type = crate::utils::$noninterned<$store>;
    };
}

pub(crate) use {debugdisplay, intern, tests, ts};
