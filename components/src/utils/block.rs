#![allow(clippy::must_use_candidate)]

use ftml_dom::utils::css::inject_css;
use leptos::prelude::*;

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
    #[prop(optional)] header: Option<super::Header>,
    #[prop(optional)] header_right: Option<HeaderRight>,
    #[prop(optional)] header_left: Option<HeaderLeft>,
    #[prop(optional)] footer: Option<Footer>,
    #[prop(optional)] show_separator: Option<bool>,
    children: Children,
) -> impl IntoView {
    use ftml_component_utils::{
        Card, CardFooter, CardHeader, CardHeaderAction, CardHeaderDescription, CardHeaderProps,
        CardPreview, Divider,
    };
    inject_css("ftml-block", include_str!("block.css"));
    let has_header = header.is_some() || header_right.is_some() || header_left.is_some();
    let has_separator = show_separator == Some(true);
    view! {
        <Card class="ftml-block-card">
            {if has_header {
                Some(CardHeader(CardHeaderProps{
                    class:Option::<String>::None.into(),
                    card_header_action:header_right.map(|c| CardHeaderAction{children:c.children}),
                    card_header_description:header_left.map(|c| CardHeaderDescription{children:c.children}),
                    children:header.map_or_else(
                      || Box::new(|| view!(<span/>).into_any()) as Children,
                      |c| Box::new(|| leptos::html::div().child((c.children)()).into_any())
                    )
                }))
            } else {None}}
            {if has_separator {
                Some(view!(<div style="margin:5px;"><Divider/></div>))
            } else {None}}
            <CardPreview class="ftml-block-card-inner">
              {children()}
            </CardPreview>
            {footer.map(|h| view!{
                <CardFooter>{(h.children)()}</CardFooter>
            })}
        </Card>
    }
}
