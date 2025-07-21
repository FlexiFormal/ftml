use std::borrow::Cow;

#[cfg(feature = "ssr")]
#[derive(Default, Clone)]
pub(crate) struct CssIds(
    std::sync::Arc<parking_lot::Mutex<rustc_hash::FxHashSet<Cow<'static, str>>>>,
);

#[inline]
pub fn inject_css(id: impl Into<Cow<'static, str>>, css: impl Into<Cow<'static, str>>) {
    do_inject_css(id.into(), css.into());
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
