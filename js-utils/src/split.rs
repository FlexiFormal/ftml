#[cfg(feature = "wasm-split")]
pub mod wsplit {
    pub use paste::paste;
    pub use wasm_split_helpers::*;
}
#[cfg(feature = "wasm-split")]
pub fn wait(
    f: impl Future<Output = leptos::prelude::AnyView> + Send + 'static,
) -> leptos::prelude::AnyView {
    use leptos::prelude::*;
    let ret = RwSignal::new(send_wrapper::SendWrapper::new(std::cell::Cell::new(None)));
    leptos::task::spawn(async move {
        let r = f.await;
        ret.update(|c| c.set(Some(r)));
    });
    (move || {
        ret.with(|v| (**v).take()).map_or_else(
            || leptos::either::Either::Right("..."),
            leptos::either::Either::Left,
        )
    })
    .into_any()
}

#[cfg(feature = "wasm-split")]
#[macro_export]
macro_rules! split {
    ($(#[$($m:meta),*])? $vis:vis fn $name:ident($($arg:ident:$t:ty),* $(,)?) -> $ret:ty { $($b:tt)* } ) => {
        $crate::split::wsplit::paste!{
            $(#[$($m),*])?
            $vis fn $name($($arg:$t),*) -> $ret {
                $crate::split::wait([<$name _async>]($($arg),*))
            }
            $(#[$($m),*])?
            #[$crate::split::wsplit::wasm_split($name,wasm_split_path = $crate::split::wsplit)]//([<$name _async>])]
            fn [<$name _async>]($($arg:$t),*) -> $ret {
                $($b)*
            }
        }
    };
}

#[cfg(not(feature = "wasm-split"))]
#[macro_export]
macro_rules! split {
    ($(#[$($m:meta),*])? $vis:vis fn $name:ident($($arg:ident:$t:ty),* $(,)?) -> $ret:ty { $($b:tt)* } ) => {
        $(#[$($m),*])?
        $vis fn $name($($arg:$t),*) -> $ret {
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
