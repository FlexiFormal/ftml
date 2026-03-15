#[cfg(feature = "wasm-split")]
pub use paste::paste;
#[cfg(feature = "wasm-split")]
pub use wasm_split_helpers::wasm_split;

#[cfg(feature = "wasm-split")]
#[macro_export]
macro_rules! split {
    ($name:ident$(<$($p:ident $(: $pb:path)? ),+>)? ($($arg:ident:$t:ty),*) -> $ret:ty { $($b:tt)* } ) => {
        fn $name$(< $($p $(:$pb)? ),+ >)? ($($arg:$t),*) -> $ret {
            todo!()
        }
        $crate::split::paste!{
            #[$crate::split::wasm_split(foo)]//([<$name _async>])]
            fn [<$name _async>]$(< $($p $(:$pb)? ),+ >)? ($($arg:$t),*) -> $ret {
                $($b)*
            }
        }
    };
}

#[cfg(not(feature = "wasm-split"))]
#[macro_export]
macro_rules! split {
    ($name:ident$(<$($p:ident $(: $pb:path)? ),+>)? ($($arg:ident:$t:ty),*) -> $ret:ty { $($b:tt)* } ) => {
        fn $name$(< $($p $(:$pb)? ),+ >)? ($($arg:$t),*) -> $ret {
            $($b)*
        }
    };
}

/*
#[cfg_attr(feature = "wasm-split", wasm_split_helpers::wasm_split(test))]
fn test<S: AsRef<str>>(s: S) -> impl leptos::IntoView {
    s.as_ref().to_string()
}
 */
