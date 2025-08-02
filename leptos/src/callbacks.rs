use ftml_ontology::narrative::elements::SectionLevel;
use ftml_uris::DocumentElementUri;
use leptos::prelude::*;

macro_rules! callback {
    (@common $name:ident( $( $arg:ident:$argtp:ty),* ) ) => {

        #[derive(Clone)]
        pub struct $name(callback!(@TYPE $($argtp),* ));

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
        impl $name {
            #[must_use]
            pub fn insert(&self, $( $arg: &$argtp, )*) -> impl IntoView + use<> {
                self.0.insert($( $arg.clone() ),*)
            }
        }
    };
    (Wrap $name:ident( $( $arg:ident:$argtp:ty),* ) ) => {
        callback!(@common $name( $($arg:$argtp),* ) );

        impl $name {
            pub fn wrap<V: IntoView, F: FnOnce() -> V>(
                &self,
                $( $arg: &$argtp, )*
                v: F,
            ) -> impl IntoView + use<V, F> {
                self.0.wrap($( $arg.clone(), )* v)
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
