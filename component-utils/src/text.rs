use std::borrow::Cow;

use leptos::prelude::*;

//pub use thaw::Text;

static TEXT_CSS: &str = include_str!("text.css");

fn make_text(
    children: AnyView,
    node: &'static str,
    class: Cow<'static, str>,
    style: Option<&'static str>,
) -> impl IntoView {
    super::inject_css("ftml-viewer-text", TEXT_CSS);
    leptos::html::custom(node)
        .child(children)
        .class(class)
        .style(style)
}

#[component]
pub fn Text(
    children: Children,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] style: Option<&'static str>,
    #[prop(default = false)] bold: bool,
    #[prop(default = false)] italic: bool,
) -> impl IntoView {
    make_text(
        children(),
        if bold {
            "b"
        } else if italic {
            "i"
        } else {
            "span"
        },
        if let Some(cls) = class
            && !cls.is_empty()
        {
            format!("ftml-viewer-text {cls}").into()
        } else {
            "ftml-viewer-text".into()
        },
        style,
    )
}

#[component]
pub fn Code(children: Children) -> impl IntoView {
    make_text(children(), "code", "ftml-viewer-code".into(), None)
}

#[component]
pub fn Caption(children: Children) -> impl IntoView {
    make_text(children(), "div", "ftml-viewer-caption".into(), None)
}

#[component]
pub fn BoldCaption(children: Children) -> impl IntoView {
    make_text(children(), "div", "ftml-viewer-bold-caption".into(), None)
}
