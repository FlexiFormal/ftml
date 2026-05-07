use std::borrow::Cow;

pub use ftml_component_utils::inject_css;
use ftml_component_utils::inject_css_after;
use ftml_ontology::utils::Css;

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
            inject_css_after(id, s.to_string());
        }
        Css::Class { name, css } => {
            inject_css_after(name.to_string(), css.to_string());
        }
        Css::Link(s) => {
            let id = hashstr("id_", &s);
            #[cfg(feature = "ssr")]
            {
                use leptos::prelude::expect_context;
                use leptos_meta::Stylesheet;
                let ids = expect_context::<ftml_component_utils::ssr::CssIds>();
                let mut ids = ids.0.lock().expect("poisoned lock");
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
                _ = head.append_with_node_1(&style);
            }
        }
    }
}
