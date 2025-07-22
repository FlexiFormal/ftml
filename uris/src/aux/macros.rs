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
        #[cfg(feature = "typescript")]
        impl$(<$($a$(:$guard)?),+>)? ::wasm_bindgen::convert::TryFromJsValue for $name$(<$($a),+>)? {
            type Error = Option<$crate::errors::UriParseError>;
            fn try_from_js_value(value: ::wasm_bindgen::JsValue) -> Result<Self,Self::Error> {
                if let Some(s) = value.as_string() {
                    s.parse().map_err(|e| Some($crate::errors::UriParseError::from(e)))
                } else {
                    Err(None)
                }
            }
        }

        #[cfg(feature = "typescript")]
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
    ($static:ident = $store:ident:$str_type:ident @ $num:literal ) => {
        #[cfg(all(feature = "interned", not(feature = "api")))]
        static $static: std::sync::LazyLock<crate::aux::interned::InternMap> =
            std::sync::LazyLock::new(crate::aux::interned::InternMap::default);
        #[cfg(all(feature = "interned", not(feature = "api")))]
        #[inline]
        unsafe fn get_ids() -> &'static crate::aux::interned::InternMap {
            &$static
        }
        #[cfg(all(feature = "interned", feature = "api"))]
        unsafe extern "C" {
            fn get_ids() -> &'static crate::aux::interned::InternMap;
        }

        #[allow(clippy::redundant_pub_crate)]
        pub(crate) struct $store;
        #[cfg(feature = "interned")]
        impl crate::aux::interned::InternStore for $store {
            const LIMIT: usize = $num;
            #[inline]
            fn get() -> &'static crate::aux::interned::InternMap {
                unsafe { get_ids() }
            }
        }
    };
    ($static:ident $type:ident = $store:ident:$str_type:ident|$noninterned:ident @ $num:literal ) => {
        crate::aux::macros::intern!($static = $store:$str_type @ $num);
        #[cfg(feature = "interned")]
        #[allow(clippy::redundant_pub_crate)]
        pub(crate) type $type = crate::aux::interned::$str_type<$store>;
        #[cfg(not(feature = "interned"))]
        #[allow(clippy::redundant_pub_crate)]
        pub(crate) type $type = crate::aux::$noninterned<$store>;
    };
}

pub(crate) use {debugdisplay, intern, tests, ts};
