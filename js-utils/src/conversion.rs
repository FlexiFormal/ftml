#![allow(clippy::trait_duplication_in_bounds)]

use crate::JsDisplay;
use send_wrapper::SendWrapper;
use std::{convert::Infallible, marker::PhantomData};
use wasm_bindgen::{JsCast, JsValue, convert::TryFromJsValue};

pub trait FromJs: Sized {
    type Error: std::fmt::Display;
    /// # Errors
    fn from_js(value: JsValue) -> Result<Self, Self::Error>;
    /// # Errors
    fn from_field(object: &JsValue, name: &str) -> Result<Option<Self>, Self::Error> {
        let Ok(v) = web_sys::js_sys::Reflect::get(object, &JsValue::from_str(name)) else {
            return Ok(None);
        };
        if v.is_undefined() {
            return Ok(None);
        }
        Self::from_js(v).map(Some)
    }
}
pub trait ToJs {
    type Error: std::fmt::Display;
    /// # Errors
    fn to_js(&self) -> Result<JsValue, Self::Error>;
}

pub trait SerdeToJs: serde::Serialize {}
impl<T: SerdeToJs> ToJs for T {
    type Error = serde_wasm_bindgen::Error;
    #[inline]
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        serde_wasm_bindgen::to_value(self)
    }
}

pub trait FromWasmBindgen: TryFromJsValue {}
impl<T: FromWasmBindgen> FromJs for T {
    type Error = JsDisplay;
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        T::try_from_js_value(value.clone()).map_err(|_| JsDisplay(value))
    }
}

//pub struct Ser<'a, T>(&'a T);
//pub struct SerDe<T>(PhantomData<T>);
//pub struct TFJSV<T: TryFromJsValue>(T);

impl<T: FromJs> FromJs for Option<T> {
    type Error = T::Error;
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        if value.is_null() {
            return Ok(None);
        }
        if value.is_undefined() {
            return Ok(None);
        }
        T::from_js(value).map(Some)
    }
}
impl<T: ToJs> ToJs for Option<T> {
    type Error = T::Error;
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        self.as_ref().map_or(Ok(JsValue::NULL), T::to_js)
    }
}
/*
impl<T: TryFromJsValue> FromJs for TFJSV<T>
where
    T::Error: std::fmt::Display,
{
    type Error = <T as TryFromJsValue>::Error;
    type Inner = T;
    #[inline]
    fn from_js(value: JsValue) -> Result<Self::Inner, Self::Error> {
        T::try_from_js_value(value)
    }
}

impl<T: serde::de::DeserializeOwned> FromJs for SerDe<T> {
    type Inner = T;
    type Error = serde_wasm_bindgen::Error;
    #[inline]
    fn from_js(value: JsValue) -> Result<Self::Inner, Self::Error> {
        serde_wasm_bindgen::from_value(value)
    }
}
impl<T: serde::Serialize> ToJs for Ser<'_, T> {
    type Error = serde_wasm_bindgen::Error;
    #[inline]
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        serde_wasm_bindgen::to_value(&self.0)
    }
}
 */

impl FromJs for () {
    type Error = crate::JsDisplay;
    fn from_js(value: JsValue) -> Result<(), Self::Error> {
        if value.is_null() || value.is_undefined() {
            Ok(())
        } else {
            Err(crate::JsDisplay(value))
        }
    }
}
impl ToJs for () {
    type Error = Infallible;
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        Ok(JsValue::NULL)
    }
}

impl FromJs for String {
    type Error = crate::JsDisplay;
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        value.as_string().ok_or(crate::JsDisplay(value))
    }
}
impl ToJs for str {
    type Error = Infallible;
    #[inline]
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        Ok(JsValue::from_str(self))
    }
}

impl FromJs for bool {
    type Error = crate::JsDisplay;
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        value.as_bool().ok_or(crate::JsDisplay(value))
    }
}
impl ToJs for bool {
    type Error = Infallible;
    #[inline]
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        Ok(JsValue::from_bool(*self))
    }
}

impl FromJs for f64 {
    type Error = crate::JsDisplay;
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        value.as_f64().ok_or(crate::JsDisplay(value))
    }
}
impl ToJs for f64 {
    type Error = Infallible;
    #[inline]
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        Ok(JsValue::from_f64(*self))
    }
}

impl ToJs for ::web_sys::Element {
    type Error = Infallible;
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        // SAFETY: infallible
        Ok(unsafe { self.clone().dyn_into().unwrap_unchecked() })
    }
}

impl ToJs for ::web_sys::HtmlDivElement {
    type Error = Infallible;
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        // SAFETY: infallible
        Ok(unsafe { self.clone().dyn_into().unwrap_unchecked() })
    }
}

impl ToJs for ::web_sys::HtmlElement {
    type Error = Infallible;
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        // SAFETY: infallible
        Ok(unsafe { self.clone().dyn_into().unwrap_unchecked() })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PairError<E1: std::fmt::Display, E2: std::fmt::Display> {
    #[error("error converting first component to js: {0}")]
    First(E1),
    #[error("error converting second component to js: {0}")]
    Second(E2),
    #[error("not a javascript array: {0}")]
    NotAJsArray(JsDisplay),
}

impl<T1: FromJs, T2: FromJs> FromJs for (T1, T2) {
    type Error = PairError<T1::Error, T2::Error>;
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        if !value.is_array() {
            return Err(PairError::NotAJsArray(value.into()));
        }
        let arr: web_sys::js_sys::Array = value
            .dyn_into()
            .map_err(|e| PairError::NotAJsArray(e.into()))?;
        let first = T1::from_js(arr.get(0)).map_err(PairError::First)?;
        let second = T2::from_js(arr.get(1)).map_err(PairError::Second)?;
        Ok((first, second))
    }
}

impl<T1: ToJs, T2: ToJs> ToJs for (T1, T2) {
    type Error = PairError<T1::Error, T2::Error>;
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        Ok(JsValue::from(web_sys::js_sys::Array::of2(
            &self.0.to_js().map_err(PairError::First)?,
            &self.1.to_js().map_err(PairError::Second)?,
        )))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TripleError<E1: std::fmt::Display, E2: std::fmt::Display, E3: std::fmt::Display> {
    #[error("error converting first component to js: {0}")]
    First(E1),
    #[error("error converting second component to js: {0}")]
    Second(E2),
    #[error("error converting third component to js: {0}")]
    Third(E3),
    #[error("not a javascript array: {0}")]
    NotAJsArray(JsDisplay),
}

impl<T1: FromJs, T2: FromJs, T3: FromJs> FromJs for (T1, T2, T3) {
    type Error = TripleError<T1::Error, T2::Error, T3::Error>;
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        if !value.is_array() {
            return Err(TripleError::NotAJsArray(value.into()));
        }
        let arr: web_sys::js_sys::Array = value
            .dyn_into()
            .map_err(|e| TripleError::NotAJsArray(e.into()))?;
        let first = T1::from_js(arr.get(0)).map_err(TripleError::First)?;
        let second = T2::from_js(arr.get(1)).map_err(TripleError::Second)?;
        let third = T3::from_js(arr.get(2)).map_err(TripleError::Third)?;
        Ok((first, second, third))
    }
}

impl<T1: ToJs, T2: ToJs, T3: ToJs> ToJs for (T1, T2, T3) {
    type Error = TripleError<T1::Error, T2::Error, T3::Error>;
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        Ok(JsValue::from(web_sys::js_sys::Array::of3(
            &self.0.to_js().map_err(TripleError::First)?,
            &self.1.to_js().map_err(TripleError::Second)?,
            &self.2.to_js().map_err(TripleError::Third)?,
        )))
    }
}
/*
impl<T1: FromJs, T2: FromJs, T3: FromJs, T4: FromJs> FromJs for (T1, T2, T3, T4) {
    type Error = either_of::EitherOf5<T1::Error, T2::Error, T3::Error, T4::Error, JsDisplay>;
    type Inner = (T1::Inner, T2::Inner, T3::Inner, T4::Inner);
    fn from_js(value: JsValue) -> Result<Self::Inner, Self::Error> {
        use either_of::EitherOf5::{A, B, C, D, E};
        if !value.is_array() {
            return Err(E(value.into()));
        }
        let arr: web_sys::js_sys::Array = value.dyn_into().map_err(|e| E(e.into()))?;
        let first = T1::from_js(arr.get(0)).map_err(A)?;
        let second = T2::from_js(arr.get(1)).map_err(B)?;
        let third = T3::from_js(arr.get(2)).map_err(C)?;
        let fourth = T4::from_js(arr.get(2)).map_err(D)?;
        Ok((first, second, third, fourth))
    }
}

impl<T1: ToJs, T2: ToJs, T3: ToJs, T4: ToJs> ToJs for (T1, T2, T3, T4) {
    type Error = either_of::EitherOf4<T1::Error, T2::Error, T3::Error, T4::Error>;
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        use either_of::EitherOf4::{A, B, C, D};
        Ok(JsValue::from(web_sys::js_sys::Array::of4(
            &self.0.to_js().map_err(A)?,
            &self.1.to_js().map_err(B)?,
            &self.2.to_js().map_err(C)?,
            &self.3.to_js().map_err(D)?,
        )))
    }
}
 */

#[derive(Debug, thiserror::Error)]
pub enum FunctionError0<E: std::fmt::Display> {
    #[error("error converting return value of js function call: {0}")]
    Ret(E),
    #[error("error calling js function: {0}")]
    Call(JsDisplay),
}

#[derive(Debug, Clone)]
pub struct JsFunction0<R: FromJs>(
    SendWrapper<web_sys::js_sys::Function>,
    PhantomData<SendWrapper<R>>,
);
impl<R: FromJs> JsFunction0<R> {
    /// # Errors
    pub fn call(&self) -> Result<R, FunctionError0<R::Error>> {
        let v = self
            .0
            .call0(&JsValue::UNDEFINED)
            .map_err(|e| FunctionError0::Call(JsDisplay(e)))?;
        R::from_js(v).map_err(FunctionError0::Ret)
    }
}
impl<R: FromJs> FromJs for JsFunction0<R> {
    type Error = JsDisplay;
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        if !value.is_function() {
            return Err(JsDisplay(value));
        }
        let f = web_sys::js_sys::Function::from(value);
        Ok(Self(SendWrapper::new(f), PhantomData))
    }
}
impl<R: FromJs> ToJs for JsFunction0<R> {
    type Error = Infallible;
    #[inline]
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        Ok(JsValue::from((*self.0).clone()))
    }
}

#[derive(Debug, Clone)]
pub struct JsFunction1<T: ToJs, R: FromJs>(
    SendWrapper<web_sys::js_sys::Function>,
    PhantomData<SendWrapper<(T, R)>>,
);
#[derive(Debug, thiserror::Error)]
pub enum FunctionError1<R: std::fmt::Display, T: std::fmt::Display> {
    #[error("error converting return value of js function call: {0}")]
    Ret(R),
    #[error("error converting js function argument call: {0}")]
    Arg(T),
    #[error("error calling js function: {0}")]
    Call(JsDisplay),
}

impl<T: ToJs, R: FromJs> JsFunction1<T, R> {
    /// # Errors
    pub fn call(&self, a: &T) -> Result<R, FunctionError1<R::Error, T::Error>> {
        let arg = a.to_js().map_err(FunctionError1::Arg)?;
        let v = self
            .0
            .call1(&JsValue::UNDEFINED, &arg)
            .map_err(|e| FunctionError1::Call(JsDisplay(e)))?;
        R::from_js(v).map_err(FunctionError1::Ret)
    }
}

impl<T: ToJs, R: FromJs> FromJs for JsFunction1<T, R> {
    type Error = JsDisplay;
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        if !value.is_function() {
            return Err(JsDisplay(value));
        }
        Ok(Self(
            SendWrapper::new(web_sys::js_sys::Function::from(value)),
            PhantomData,
        ))
    }
}

impl<T: ToJs, R: FromJs> ToJs for JsFunction1<T, R> {
    type Error = Infallible;
    #[inline]
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        Ok(JsValue::from((*self.0).clone()))
    }
}

#[derive(Debug, Clone)]
pub struct JsFunction2<T1: ToJs, T2: ToJs, R: FromJs>(
    SendWrapper<web_sys::js_sys::Function>,
    PhantomData<SendWrapper<(T1, T2, R)>>,
);
#[derive(Debug, thiserror::Error)]
pub enum FunctionError2<R: std::fmt::Display, T1: std::fmt::Display, T2: std::fmt::Display> {
    #[error("error converting return value of js function call: {0}")]
    Ret(R),
    #[error("error converting first js function argument call: {0}")]
    Arg1(T1),
    #[error("error converting second js function argument call: {0}")]
    Arg2(T2),
    #[error("error calling js function: {0}")]
    Call(JsDisplay),
}

impl<T1: ToJs, T2: ToJs, R: FromJs> JsFunction2<T1, T2, R> {
    /// # Errors
    #[allow(clippy::type_complexity)]
    pub fn call(
        &self,
        a: &T1,
        b: &T2,
    ) -> Result<R, FunctionError2<R::Error, T1::Error, T2::Error>> {
        let arg1 = a.to_js().map_err(FunctionError2::Arg1)?;
        let arg2 = b.to_js().map_err(FunctionError2::Arg2)?;
        let v = self
            .0
            .call2(&JsValue::UNDEFINED, &arg1, &arg2)
            .map_err(|e| FunctionError2::Call(JsDisplay(e)))?;
        R::from_js(v).map_err(FunctionError2::Ret)
    }
}

impl<T1: ToJs, T2: ToJs, R: FromJs> FromJs for JsFunction2<T1, T2, R> {
    type Error = JsDisplay;
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        if !value.is_function() {
            return Err(JsDisplay(value));
        }
        Ok(Self(
            SendWrapper::new(web_sys::js_sys::Function::from(value)),
            PhantomData,
        ))
    }
}
impl<T1: ToJs, T2: ToJs, R: FromJs> ToJs for JsFunction2<T1, T2, R> {
    type Error = Infallible;
    #[inline]
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        Ok(JsValue::from((*self.0).clone()))
    }
}

#[derive(Debug, Clone)]
pub struct JsFunction3<T1: ToJs, T2: ToJs, T3: ToJs, R: FromJs>(
    SendWrapper<web_sys::js_sys::Function>,
    PhantomData<SendWrapper<(T1, T2, T3, R)>>,
);
#[derive(Debug, thiserror::Error)]
pub enum FunctionError3<
    R: std::fmt::Display,
    T1: std::fmt::Display,
    T2: std::fmt::Display,
    T3: std::fmt::Display,
> {
    #[error("error converting return value of js function call: {0}")]
    Ret(R),
    #[error("error converting first js function argument call: {0}")]
    Arg1(T1),
    #[error("error converting second js function argument call: {0}")]
    Arg2(T2),
    #[error("error converting third js function argument call: {0}")]
    Arg3(T3),
    #[error("error calling js function: {0}")]
    Call(JsDisplay),
}

impl<T1: ToJs, T2: ToJs, T3: ToJs, R: FromJs> JsFunction3<T1, T2, T3, R> {
    /// # Errors
    #[allow(clippy::type_complexity)]
    pub fn call(
        &self,
        a: &T1,
        b: &T2,
        c: &T3,
    ) -> Result<R, FunctionError3<R::Error, T1::Error, T2::Error, T3::Error>> {
        let arg1 = a.to_js().map_err(FunctionError3::Arg1)?;
        let arg2 = b.to_js().map_err(FunctionError3::Arg2)?;
        let arg3 = c.to_js().map_err(FunctionError3::Arg3)?;
        let v = self
            .0
            .call3(&JsValue::UNDEFINED, &arg1, &arg2, &arg3)
            .map_err(|e| FunctionError3::Call(JsDisplay(e)))?;
        R::from_js(v).map_err(FunctionError3::Ret)
    }
}
impl<T1: ToJs, T2: ToJs, T3: ToJs, R: FromJs> FromJs for JsFunction3<T1, T2, T3, R> {
    type Error = JsDisplay;
    fn from_js(value: JsValue) -> Result<Self, Self::Error> {
        if !value.is_function() {
            return Err(JsDisplay(value));
        }
        Ok(Self(
            SendWrapper::new(web_sys::js_sys::Function::from(value)),
            PhantomData,
        ))
    }
}

impl<T1: ToJs, T2: ToJs, T3: ToJs, R: FromJs> ToJs for JsFunction3<T1, T2, T3, R> {
    type Error = Infallible;
    #[inline]
    fn to_js(&self) -> Result<JsValue, Self::Error> {
        Ok(JsValue::from((*self.0).clone()))
    }
}

/*

macro_rules! num {
    ($($tp:ty),*) => {
        $(
            impl FromJs for $tp {
                type Error = crate::JsDisplay;
                type Inner = Self;
                #[allow(clippy::cast_possible_truncation)]
                fn from_js(value: JsValue) -> Result<Self::Inner, Self::Error> {
                    value.as_f64().ok_or(crate::JsDisplay(value)).map(|v| v as _)
                }
            }

            impl ToJs for $tp {
                type Error = Infallible;
                #[inline]
                #[allow(clippy::cast_precision_loss,clippy::cast_lossless)]
                fn to_js(&self) -> Result<JsValue, Self::Error> {
                    Ok(JsValue::from_f64(*self as _))
                }
            }
        )*
    }
}
num!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, f32);
 */
