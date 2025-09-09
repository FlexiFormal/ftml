use std::marker::PhantomData;

use send_wrapper::SendWrapper;
use wasm_bindgen::{JsValue, convert::TryFromJsValue};
use web_sys::js_sys::Function;

use ftml_js_utils::JsDisplay;

#[derive(thiserror::Error, Debug)]
pub enum CallbackError<E> {
    #[error("Javascript error: {0}")]
    JsError(JsDisplay),
    #[error("Conversion error: {0}")]
    Convert(E),
}

#[derive(thiserror::Error, Debug)]
#[error("Not a javascript function: {0}")]
pub struct NotAJsFunction(JsDisplay);

macro_rules! fun {
    ( $name:ident:$tp:ident$(<$( $aname:ident = $arg:ident),+>)? = $f:ident) => {
        pub struct $name< $( $($arg: Into<JsValue>),* , )? R:JsRet > {
            js: ::send_wrapper::SendWrapper<::leptos::web_sys::js_sys::Function>,
            #[allow(unused_parens)]
            __phantom:PhantomData<SendWrapper<( $( $($arg),* , )? R )>>
        }
        impl< $( $($arg: Into<JsValue>),* , )? R:JsRet > $name< $( $($arg),* , )? R  > {
            pub fn apply(&self $(, $($aname:$arg),* )?  ) -> Result<R, CallbackError<R::Error>> {
                let r = self.js.$f(&JsValue::UNDEFINED $(,  $(&$aname.into()),*  )?  )
                    .map_err(|e| CallbackError::JsError(JsDisplay(e)))?;
                JsRet::from_js(r).map_err(CallbackError::Convert)
            }
            #[inline]
            pub fn into_impl(self) -> impl Fn( $( $($arg),* )? ) ->  Result<R, CallbackError<R::Error>> {
                move | $( $($aname),+  )? | self.apply( $( $($aname),* )? )
            }
        }
        impl< $( $($arg: Into<JsValue>),* , )? R:JsRet > Clone for $name< $( $($arg),* , )? R > {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    js: self.js.clone(),
                    __phantom: PhantomData,
                }
            }
        }
        impl< $( $($arg: Into<JsValue>),* , )? R:JsRet > From< Function > for $name< $( $($arg),* , )? R > {
            #[inline]
            fn from(f: Function) -> Self {
                Self {
                    js: SendWrapper::new(f),
                    __phantom: PhantomData,
                }
            }
        }
        impl< $( $($arg: Into<JsValue>),* , )? R:JsRet > From< $name< $( $($arg),* , )? R > > for JsValue {
            #[inline]
            fn from(f: $name< $( $($arg),* , )? R >) -> Self {
                (*f.js).clone().into()
            }
        }
        impl< $( $($arg: Into<JsValue>),* , )? R:JsRet > JsRet for $name< $( $($arg),* , )? R > {
            type Error = NotAJsFunction;
            fn from_js(value: JsValue) -> Result<Self, Self::Error> {
                if !value.is_function() {
                    return Err(NotAJsFunction(JsDisplay(value)));
                }
                Ok(Self {
                    js: SendWrapper::new(web_sys::js_sys::Function::from(value)),
                    __phantom: PhantomData,
                })
            }
        }
        impl< $( $($arg: Into<JsValue>),* , )? R:JsRet > wasm_bindgen::convert::FromWasmAbi for $name< $( $($arg),* , )? R > {
            type Abi = u32;
            unsafe fn from_abi(js: Self::Abi) -> Self {
                Self {
                    js: SendWrapper::new(unsafe { JsValue::from_abi(js) }.into()),
                    __phantom: PhantomData,
                }
            }
        }
        impl< $( $($arg: Into<JsValue>),* , )? R:JsRet > wasm_bindgen::convert::IntoWasmAbi for $name< $( $($arg),* , )? R > {
            type Abi = u32;
            fn into_abi(self) -> Self::Abi {
                self.js.take().into_abi()
            }
        }
        impl< $( $($arg: Into<JsValue>),* , )? R:JsRet > wasm_bindgen::describe::WasmDescribe for $name< $( $($arg),* , )? R > {
            fn describe() {
                <Function as wasm_bindgen::describe::WasmDescribe>::describe();
            }
        }

        pub trait $tp {
            type R:JsRet;
            $($(
            type $arg: Into<JsValue>;
            )*)?

            #[cfg(feature = "typescript")]
            fn get(self) -> $name< $( $(Self::$arg),* , )? Self::R >;
        }
    }
}

fun!(JsFunction0: TsFunctionType0 = call0);
fun!(JsFunction1: TsFunctionType1 <a = Arg> = call1);
fun!(JsFunction2: TsFunctionType2 <a = Arg1, b = Arg2> = call2);
fun!(JsFunction3: TsFunctionType3 <a = Arg1, b = Arg2, c = Arg3> = call3);

pub struct RetFromJs<R: TryFromJsValue>(pub R);

pub trait JsRet: Sized {
    type Error;
    fn from_js(value: JsValue) -> Result<Self, Self::Error>;
}

impl<T> JsRet for RetFromJs<T>
where
    T: TryFromJsValue,
{
    type Error = T::Error;
    #[inline]
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        T::try_from_js_value(value).map(RetFromJs)
    }
}

impl<R: JsRet> JsRet for Option<R> {
    type Error = R::Error;
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        if value.is_null() {
            return Ok(None);
        }
        if value.is_undefined() {
            return Ok(None);
        }
        R::from_js(value).map(Some)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Not a javascript function: {0}")]
pub struct NotAVoid(JsDisplay);

pub struct Void;
impl JsRet for Void {
    type Error = NotAVoid;
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        if value.is_null() || value.is_undefined() {
            Ok(Self)
        } else {
            Err(NotAVoid(JsDisplay(value)))
        }
    }
}

/*

#[cfg(feature = "typescript")]
#[macro_export]
macro_rules! new_ts_funtype {
    ($name:ident @ $ts:literal = $($tp:tt)* ) => {
        #[::wasm_bindgen::prelude::wasm_bindgen]
        extern "C" {
            #[::wasm_bindgen::prelude::wasm_bindgen(extends = ::leptos::web_sys::js_sys::Function)]
            #[::wasm_bindgen::prelude::wasm_bindgen(typescript_type = $ts)]
            pub type $name;
        }
        $crate::new_ts_funtype!(@TYPE $name $($tp)*);
    };
    (@IMPL $name:ident $tp:ident $struc:ident $($ret:ty)?; $($($arg:ident = $atp:ty;)+)? ) => {
        impl $crate::functions::$tp for $name {
            type R = $crate::new_ts_funtype!(@RET $($ret)?);
            $($(
                type $arg = $atp;
            )*)?
            #[inline]
            fn get(self) -> $crate::functions::$struc< $( $(Self::$arg),* , )? Self::R > {
                $crate::functions::$struc::from(::web_sys::js_sys::Function::from(self))
            }
        }
    };
    (@RET) => {$crate::functions::Void};
    (@RET $ret:ty) => {$ret};
    (@TYPE $name:ident () $(=> $ret:ty)? ) => {
        $crate::new_ts_funtype!(@IMPL $name TsFunctionType0 JsFunction0 $($ret)?;);
    };
    (@TYPE $name:ident ($ta:ty,$tb:ty,$tc:ty) $(=> $ret:ty)? ) => {
        $crate::new_ts_funtype!(@IMPL $name TsFunctionType3 JsFunction3 $($ret)?; Arg1 = $ta; Arg2=$tb; Arg3 = $tc;);
    };
    (@TYPE $name:ident ($ta:ty,$tb:ty) $(=> $ret:ty)? ) => {
        $crate::new_ts_funtype!(@IMPL $name TsFunctionType2 JsFunction2 $($ret)?; Arg1 = $ta; Arg2=$tb;);
    };
    (@TYPE $name:ident $ta:ty $(=> $ret:ty)? ) => {
        $crate::new_ts_funtype!(@IMPL $name TsFunctionType1 JsFunction1 $($ret)?; Arg = $ta;);
    };
}

#[cfg(not(feature = "typescript"))]
#[macro_export]
macro_rules! new_ts_funtype {}

new_ts_funtype!(Test @ "test => void;" = (String,i32) );

*/
