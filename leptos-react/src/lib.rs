pub mod context;
//pub mod functions;

use crate::{
    context::LeptosContext,
    //functions::{JsFunction0, JsFunction1, JsFunction2, JsFunction3, Void},
};
use ftml_js_utils::conversion::{JsFunction0, JsFunction1, JsFunction2, JsFunction3};
use wasm_bindgen::prelude::*;
use web_sys::HtmlDivElement;
pub mod __reexp {
    pub use paste::*;
}
pub mod functions {
    pub use ftml_js_utils::conversion::*;
}

#[cfg(feature = "react")]
#[wasm_bindgen(typescript_custom_section)]
const REACT_IMPORTS: &str = include_str!("react.ts");

pub type LeptosContinuation = JsFunction2<HtmlDivElement, LeptosContext, ()>;
#[cfg(feature = "typescript")]
#[wasm_bindgen(typescript_custom_section)]
const LEPTOS_CONT: &str = r#"
export type LeptosContinuation = (e:HTMLDivElement,o:LeptosContext) => void;
"#;

macro_rules! callback {
    ($tp:ident$(<$($a:ident: $atp:ident),*>)? @ $trt:ident = $js:ident) => {
        pub type $tp$( < $($atp),* > )? = $js< $($($atp),* ,)? Option<LeptosContinuation> >;
    }
}

callback!(ReactWrapper @ Wrappable = JsFunction0);
callback!(ReactWrapper1<a:Arg1> @ Wrappable1 = JsFunction1);
callback!(ReactWrapper2<a:Arg1,b:Arg2> @ Wrappable2 = JsFunction2);
callback!(ReactWrapper3<a:Arg1,b:Arg2,c:Arg3> @ Wrappable3 = JsFunction3);

#[macro_export]
macro_rules! wrapper {
    ($name:ident( $( $arg:ident:$argtp:ident),* )) => {
        //pub type $name = $crate::wrapper!(@TYPE $($(@$w)? $argtp;)* );
        #[derive(Debug,Clone)]
        pub struct $name($crate::wrapper!(@TYPE $($argtp;)* ));
        $crate::__ts__!($name $( $arg:$argtp),*);

        impl $name {
            pub fn wrap<V: IntoView, F: FnOnce() -> V>(
                &self,
                $( $arg: &$argtp, )*
                children: F,
            ) -> impl IntoView + use<V, F> {
                use ::leptos::{either::Either::{Left, Right},prelude::*};
                match self.0.call($($arg),*) {
                    Ok(Some(cont)) => {
                        let owner = Owner::current()
                            .expect("Not in a leptos reactive context!")
                            .into();
                        let rf = NodeRef::new();
                        rf.on_load(move |elem| {
                            if let Err(err) = cont.call(&elem, &owner) {
                                tracing::error!("Error calling continuation: {err}");
                            }
                        });
                        Left(view! {<div node_ref=rf>{children()}</div>})
                    }
                    Ok(None) => Right(children()),
                    Err(e) => {
                        tracing::error!("Error calling continuation: {e}");
                        Right(children())
                    }
                }
            }
        }
        impl $crate::functions::FromJs for $name {
            type Error = <$crate::wrapper!(@TYPE $($argtp;)* ) as $crate::functions::FromJs>::Error;
            #[inline]
            fn from_js(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
                $crate::functions::FromJs::from_js(value).map(Self)
            }
        }

        /*
        $crate::__ts__!($name $( $arg:$argtp),*);

        impl $crate::functions::JsRet for $name {
            type Error =
                <$crate::wrapper!(@TYPE $($argtp),* ) as $crate::functions::JsRet>::Error;
            #[inline]
            fn from_js(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
                $crate::functions::JsRet::from_js(value).map(Self)
            }
        }
         */
    };

    (@TYPE) => { $crate::ReactWrapper };
    (@TYPE $(@$w:ident)? $t:ident;) => { $crate::ReactWrapper1<$crate::wrapper!(@TP $(@$w)? $t )> };
    (@TYPE $(@$wa:ident)? $ta:ident; $(@$wb:ident)? $tb:ident; ) => {
        $crate::ReactWrapper2<$crate::wrapper!(@TP $(@$wa)? $ta),$crate::wrapper!(@TP $(@$wb)? $tb)>
    };
    (@TYPE $(@$wa:ident)? $ta:ident; $(@$wb:ident)? $tb:ident; $(@$wc:ident)? $tc:ident;) => {
        $crate::ReactWrapper3<$crate::wrapper!(@TP $(@$wa)? $ta),$crate::wrapper!(@TP $(@$wb)? $tb),$crate::wrapper!(@TP $(@$wc)? $tc)>
    };
    (@TYPE $($rest:tt)+) => { ::std::compile_error!("Too many arguments for wrapper macro")};

    (@TP @Ser $t:ident) => { ::ftml_js_utils::conversion::Ser<'_,$t> };
    (@TP @$w:ident $t:ident) => { ::ftml_js_utils::conversion::$w<$t> };
    (@TP $t:ident) => { $t };
}

#[macro_export]
macro_rules! insertion {
    ($name:ident( $( $arg:ident:$(@$w:ident)? $argtp:ident),* )) => {
        #[derive(Debug,Clone)]
        pub struct $name($crate::wrapper!(@TYPE $($argtp;)* ));
        //pub type $name = $crate::wrapper!(@TYPE $($(@$w)? $argtp;)* );
        $crate::__ts__!($name $( $arg:$argtp),*);

        impl $name {
            #[must_use]
            pub fn insert(&self, $( $arg: &$argtp, )*) -> impl IntoView + use<> {
                use ::leptos::{either::Either::{Left, Right},prelude::*};
                match self.0.call($($arg),*) {
                    Ok(None) => None,
                    Err(e) => {
                        tracing::error!("Error calling insertion callback: {e}");
                        None
                    }
                    Ok(Some(cont)) => {
                        let ret = NodeRef::new();
                        let owner = Owner::current().expect("Not in a leptos reactive context!").into();
                        ret.on_load(move |e| {
                            if let Err(e) = cont.call(&e, &owner) {
                                tracing::error!("Error calling continuation: {e}");
                            }
                        });
                        Some(view!(<div node_ref = ret/>))
                    }
                }
            }
        }
        impl $crate::functions::FromJs for $name {
            type Error = <$crate::wrapper!(@TYPE $($argtp;)* ) as $crate::functions::FromJs>::Error;
            #[inline]
            fn from_js(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
                $crate::functions::FromJs::from_js(value).map(Self)
            }
        }

        /*
        $crate::__ts__!($name $( $arg:$argtp),*);

        impl $crate::functions::JsRet for $name {
            type Error =
                <$crate::wrapper!(@TYPE $($argtp),* ) as $crate::functions::JsRet>::Error;
            #[inline]
            fn from_js(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
                $crate::functions::JsRet::from_js(value).map(Self)
            }
        }

        */
    };
}

#[cfg(feature = "typescript")]
#[macro_export]
macro_rules! __ts__ {
    ($name:ident $($arg:ident:$argtp:ident),*) => {
        $crate::__reexp::paste! {
            #[::wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
            const [<$name:snake:upper _TS>]: &str = concat!(
                "export type ",
                stringify!($name),
                " = (",
                $crate::__ts__!(@ARG $($arg:$argtp;)*),
                ") => (LeptosContinuation | undefined);"
            );
        }
    };
    (@ARG) => {""};
    (@ARG $arg:ident:$argtp:ident; $($rest:tt)+) => {
        concat!(
            $crate::__ts__!(
                @ARG $arg:$argtp;),
                ", ",
                $crate::__ts__!(@ARG$($rest)*)
        )
    };
    (@ARG $arg:ident:String;) => { concat!(
        stringify!($arg),
        ":string"
    )};
    (@ARG $arg:ident:bool;) => { concat!(
        stringify!($arg),
        ":boolean"
    )};
    (@ARG $arg:ident:$argtp:ident;) => { concat!(
        stringify!($arg),
        ":",
        stringify!($argtp)
    )};
}

#[cfg(not(feature = "typescript"))]
#[macro_export]
macro_rules! __ts__ {
    ($name:ident $($arg:ident:$argtp:ident),*) => {};
}
