use ftml_ontology::narrative::elements::SectionLevel;
use ftml_uris::DocumentElementUri;
use leptos::prelude::*;

macro_rules! callback {
    (@common $name:ident( $( $arg:ident:$argtp:ty),* ) ) => {

        #[cfg(feature = "typescript")]
        #[derive(Clone)]
        pub struct $name(callback!(@TYPE $($argtp),* ));

        #[cfg(feature = "typescript")]
        impl leptos_react::functions::JsRet for $name {
            type Error =
                <callback!(@TYPE $($argtp),* ) as leptos_react::functions::JsRet>::Error;
            #[inline]
            fn from_js(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
                leptos_react::functions::JsRet::from_js(value).map(Self)
            }
        }
    };
    (Insert $name:ident( $( $arg:ident:$argtp:ty),* ) ) => {
        callback!(@common $name( $($arg:$argtp),* ) );

        #[cfg(not(feature = "typescript"))]
        impl $name {
            pub fn from<V: IntoView>(
                f: impl Fn($(&$argtp,)*) -> V + 'static + Send + Sync,
            ) -> Self {
                Self(std::sync::Arc::new(move |$($arg),*| f($($arg),* ).into_any()))
            }
        }

        #[cfg(not(feature = "typescript"))]
        #[derive(Clone)]
        pub struct $name(std::sync::Arc<dyn Fn(  $(&$argtp),*) -> AnyView + Send + Sync>);
        impl $name {
            #[must_use]
            pub fn insert(&self, $( $arg: &$argtp, )*) -> impl IntoView + use<> {
                #[cfg(not(feature = "typescript"))]
                {
                    (&*self.0)($( $arg),* )
                }
                #[cfg(feature = "typescript")]
                {
                    self.0.insert($( $arg.clone() ),*)
                }
            }
        }
    };
    (Wrap $name:ident( $( $arg:ident:$argtp:ty),* ) ) => {
        callback!(@common $name( $($arg:$argtp),* ) );

        #[cfg(not(feature = "typescript"))]
        impl $name {
            pub fn from<V: IntoView>(
                f: impl Fn($(&$argtp,)* AnyView) -> V + 'static + Send + Sync,
            ) -> Self {
                Self(std::sync::Arc::new(move |$($arg,)* v| f($($arg,)* v).into_any()))
            }
        }

        #[cfg(not(feature = "typescript"))]
        #[derive(Clone)]
        pub struct $name(std::sync::Arc<dyn Fn(  $(&$argtp,)*  AnyView) -> AnyView + Send + Sync>);

        impl $name {
            pub fn wrap<V: IntoView, F: FnOnce() -> V>(
                &self,
                $( $arg: &$argtp, )*
                v: F,
            ) -> impl IntoView + use<V, F> {
                #[cfg(not(feature = "typescript"))]
                {
                    let any = view! { {v()}}.into_any();
                    (&*self.0)($( $arg, )* any)
                }
                #[cfg(feature = "typescript")]
                {
                    self.0.wrap($( $arg.clone(), )* v)
                }
            }
        }
    };
    (@TYPE ) => { leptos_react::ReactWrapper };
    (@TYPE $t:ty) => { leptos_react::ReactWrapper1<$t> };
    (@TYPE $ta:ty, $tb:ty) => { leptos_react::ReactWrapper2<$ta,$tb> };
    (@TYPE $ta:ty, $tb:ty, $tc:ty) => { leptos_react::ReactWrapper3<$ta,$tb,$tc> };
}

callback!(Wrap SectionWrap(u:DocumentElementUri));
callback!(Insert OnSectionTitle(u:DocumentElementUri,lvl: SectionLevel));
