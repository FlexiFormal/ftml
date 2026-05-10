#![allow(clippy::must_use_candidate)]

use std::borrow::Cow;

use leptos::prelude::*;

#[leptos::prelude::slot]
pub struct Header {
    children: leptos::prelude::Children,
}

#[leptos::prelude::slot]
pub struct HeaderLeft {
    children: leptos::prelude::Children,
}

#[leptos::prelude::slot]
pub struct HeaderRight {
    children: leptos::prelude::Children,
}

#[leptos::prelude::slot]
pub struct Footer {
    children: leptos::prelude::Children,
}

#[component]
pub fn Block(
    #[prop(optional)] header: Option<Header>,
    #[prop(optional)] header_right: Option<HeaderRight>,
    #[prop(optional)] header_left: Option<HeaderLeft>,
    #[prop(optional)] footer: Option<Footer>,
    #[prop(optional)] show_separator: Option<bool>,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    use crate::Divider;
    super::inject_css("ftml-viewer-block", include_str!("block.css"));
    let has_header = header.is_some() || header_right.is_some() || header_left.is_some();
    let has_separator = show_separator == Some(true);
    let header_cls = if header_left.is_some() {
        "ftml-viewer-block-complex-header"
    } else {
        "ftml-viewer-block-header"
    };
    let main_class: Cow<str> = class.map_or_else(
        || "ftml-viewer-block".into(),
        |cls| format!("ftml-viewer-block {cls}").into(),
    );
    view! {
        <div class=main_class role="group">
            {if has_header {
                Some(view!{
                    <div class=header_cls>
                        <div class="ftml-viewer-block-header__inner">
                            {header.map(|h| (h.children)())}
                        </div>
                        {header_left.map(|ch| view!{
                            <div class="ftml-viewer-block-header-left">{(ch.children)()}</div>
                        })}
                        {header_right.map(|ch| view!{
                            <div class="ftml-viewer-block-header-right">{(ch.children)()}</div>
                        })}

                    </div>
                })
            } else {
                None
            }}
            {if has_separator {
                Some(view!(<div style="margin:5px;"><Divider/></div>))
            } else {None}}
            <div class="ftml-viewer-block-inner">
              {children()}
            </div>
            {footer.map(|h| view!{
                <div class="ftml-viewer-block-footer">{(h.children)()}</div>
            })}
        </div>
    }
}
