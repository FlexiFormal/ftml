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

#[leptos::prelude::slot]
pub struct Separator {
    children: leptos::prelude::Children,
}

#[component]
pub fn Block(
    #[prop(optional)] header: Option<super::Header>,
    #[prop(optional)] header_right: Option<HeaderRight>,
    #[prop(optional)] header_left: Option<HeaderLeft>,
    #[prop(optional)] footer: Option<Footer>,
    #[prop(optional)] separator: Option<Separator>,
    #[prop(optional)] show_separator: Option<bool>,
    children: Children,
) -> impl IntoView {
    use thaw::{
        Card, CardFooter, CardHeader, CardHeaderAction, CardHeaderDescription, CardHeaderProps,
        CardPreview, Divider,
    };
    inject_css("ftml-block", include_str!("block.css"));
    let has_header = header.is_some() || header_right.is_some() || header_left.is_some();
    let has_separator = separator.is_some()
        || show_separator == Some(true)
        || (show_separator.is_none() && has_header);
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
                Some(separator.map_or_else(
                  || view!(<div style="margin:5px;"><Divider/></div>),
                  |c| view!(<div style="margin:5px;"><Divider>{(c.children)()}</Divider></div>)
                ))
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
/*
pub fn block<H1: IntoView, S: IntoView + 'static, C: IntoView + 'static, F: IntoView + 'static>(
    header: impl FnOnce() -> H1 + Send + 'static,
    header_left: Option<impl FnOnce() -> AnyView + Send + 'static>,
    header_right: Option<impl FnOnce() -> AnyView + Send + 'static>,
    separator: Option<impl FnOnce() -> S + Send + 'static>,
    children: impl FnOnce() -> C + Send + 'static,
    footer: Option<impl FnOnce() -> F + Send + 'static>,
) -> impl IntoView {
    use thaw::{
        Card, CardFooter, CardHeader, CardHeaderAction, CardHeaderDescription, CardHeaderProps,
        CardPreview, Divider,
    };
    inject_css("ftml-block", include_str!("block.css"));
    let header = CardHeader(CardHeaderProps {
        class: Option::<String>::None.into(),
        card_header_action: header_right.map(|c| CardHeaderAction {
            children: Box::new(c),
        }),
        card_header_description: header_left.map(|c| CardHeaderDescription {
            children: Box::new(c),
        }),
        children: Box::new(|| leptos::html::div().child(header()).into_any()),
    });
    let separator = separator.map_or_else(
        || view!(<div style="margin:5px;"><Divider/></div>),
        |c| view!(<div style="margin:5px;"><Divider>{c()}</Divider></div>),
    );

    view! {
        <Card class="ftml-block-card">
            {header}
            {separator}
        </Card>
        <CardPreview class="ftml-block-card-inner">
          {children()}
        </CardPreview>
        {footer.map(|f| view!{
            <CardFooter>{f()}</CardFooter>
        })}
    }
}
 */
