use std::borrow::Cow;

use leptos::{context::Provider, prelude::*};

#[component]
pub fn TableRow(children: Children) -> impl IntoView {
    leptos::html::tr().child(children())
}

#[component]
pub fn TableCell(
    children: Children,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] style: Option<&'static str>,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    let children = children();
    if InHeader::is() {
        Left(
            leptos::html::th()
                .class(class)
                .style(style)
                .child(leptos::html::button().role("presentation").child(children)),
        )
    } else {
        Right(
            leptos::html::td().class(class).style(style).child(
                leptos::html::div()
                    .child(leptos::html::div().child(leptos::html::span().child(children))),
            ),
        )
    }
}

#[slot]
pub struct TableHeader {
    children: leptos::prelude::Children,
}

#[derive(Copy, Clone)]
struct InHeader(bool);
impl InHeader {
    fn is() -> bool {
        use_context::<Self>().is_some_and(|b| b.0)
    }
}

#[component]
pub fn Table(
    #[prop(optional)] table_header: Option<TableHeader>,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    super::inject_css("ftml-viewer-table", include_str!("tables.css"));
    let class: Cow<'static, str> = if let Some(cls) = class
        && !cls.is_empty()
    {
        format!("ftml-viewer-table {cls}").into()
    } else {
        "ftml-viewer-table".into()
    };
    let children = view!(<Provider value=InHeader(false)><tbody>{children()}</tbody></Provider>);

    let children = if let Some(header) = table_header {
        Left(
            view!(<thead><tr><Provider value=InHeader(true)>{(header.children)()}</Provider></tr></thead>{children}),
        )
    } else {
        Right(children)
    };
    leptos::html::table().class(class).child(children)
}
