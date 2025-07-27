use std::borrow::Cow;

use ftml_ontology::utils::Css;

#[cfg(feature = "ssr")]
#[derive(Default, Clone)]
pub(crate) struct CssIds(
    std::sync::Arc<parking_lot::Mutex<rustc_hash::FxHashSet<Cow<'static, str>>>>,
);

#[inline]
pub fn inject_css(id: impl Into<Cow<'static, str>>, css: impl Into<Cow<'static, str>>) {
    do_inject_css(id.into(), css.into());
}

fn hashstr<A: std::hash::Hash>(prefix: &str, a: &A) -> String {
    use std::hash::BuildHasher;
    let h = rustc_hash::FxBuildHasher.hash_one(a);
    format!("{prefix}{h:02x}")
}

pub trait CssExt {
    fn inject(self);
}
impl CssExt for Css {
    #[inline]
    fn inject(self) {
        do_css(self);
    }
}

fn do_css(css: Css) {
    match css {
        Css::Inline(s) => {
            let id = hashstr("id_", &s);
            do_inject_css(id.into(), s.to_string().into());
        }
        Css::Class { name, css } => {
            do_inject_css(name.to_string().into(), css.to_string().into());
        }
        Css::Link(s) => {
            let id = hashstr("id_", &s);
            #[cfg(feature = "ssr")]
            {
                use leptos::prelude::expect_context;
                use leptos_meta::Stylesheet;
                let ids = expect_context::<CssIds>();
                let mut ids = ids.0.lock();
                if !ids.contains(&*id) {
                    ids.insert(id.clone().into());
                    let _ = leptos::view! {
                        <Stylesheet id=id href=s.to_string()/>
                    };
                }
                drop(ids);
            }
            #[cfg(not(feature = "ssr"))]
            {
                use leptos::prelude::document;
                let Some(head) = document().head() else {
                    leptos::logging::log!("ERROR: head does not exist");
                    return;
                };
                match head.query_selector(&format!("link#{id}")) {
                    Ok(Some(_)) => return,
                    Err(e) => {
                        leptos::logging::log!("ERROR: query link element error: {e:?}");
                        return;
                    }
                    Ok(None) => (),
                }
                let Ok(style) = document().create_element("link") else {
                    leptos::logging::log!("ERROR: error creating style element");
                    return;
                };
                _ = style.set_attribute("id", &id);
                _ = style.set_attribute("rel", "stylesheet");
                _ = style.set_attribute("href", &s);
                _ = head.prepend_with_node_1(&style);
            }
        }
    }
}

#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::needless_pass_by_value)]
fn do_inject_css(id: Cow<'static, str>, content: Cow<'static, str>) {
    #[cfg(feature = "ssr")]
    {
        use leptos_meta::Style;

        use leptos::prelude::expect_context;
        let ids = expect_context::<CssIds>();
        let mut ids = ids.0.lock();
        if !ids.contains(&id) {
            ids.insert(id.clone());
            let _ = leptos::view!(<Style id=id>{content}</Style>);
        }
        drop(ids);
    }
    #[cfg(not(feature = "ssr"))]
    {
        use leptos::prelude::document;
        let Some(head) = document().head() else {
            tracing::error!("head does not exist");
            return;
        };
        let Ok(style) = head.query_selector(&format!("style#{id}")) else {
            tracing::error!("query style element error");
            return;
        };
        if style.is_some() {
            return;
        }

        let Ok(style) = document().create_element("style") else {
            tracing::error!("error creating style element");
            return;
        };
        _ = style.set_attribute("id", &id);
        style.set_text_content(Some(&content));
        _ = head.prepend_with_node_1(&style);
    }
}
