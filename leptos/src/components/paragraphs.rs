use ftml_dom::utils::css::inject_css;
use leptos::prelude::*;

use crate::config::FtmlConfigState;

pub fn paragraph<V: IntoView>(
    info: ftml_dom::markers::ParagraphInfo,
    then: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    inject_css("ftml-sections", include_str!("sections.css"));
    view! {
        <div class=info.class style=info.style>{
            FtmlConfigState::wrap_paragraph(&info.uri,info.kind,then)
        }</div>
    }
}
