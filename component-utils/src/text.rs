use leptos::html::div;
use leptos::prelude::*;

pub use thaw::Body1 as BodyText;
pub use thaw::Caption1 as Caption;
pub use thaw::Caption1Strong as BoldCaption;
pub use thaw::Text;

#[component]
pub fn Code(children: Children) -> impl IntoView {
    inject_css("ftml-viewer-text", include_str!("text.css"));
    view!(<div class="ftml-viewer-code">{children()}</div>)
}

/*
#[component]
pub fn SText<V: IntoView + 'static>(children: TypedChildren<V>) -> impl IntoView {
    let children = children.into_inner();
    code(children())
}
pub fn stext(children: impl IntoView + 'static) -> impl IntoView {
    use thaw::{Text, TextTag};
    view!(<Text>{children}</Text>)
}

#[component]
pub fn Code<V: IntoView + 'static>(children: TypedChildren<V>) -> impl IntoView {
    let children = children.into_inner();
    code(children())
}
pub fn code(children: impl IntoView + 'static) -> impl IntoView {
    use thaw::{Text, TextTag};
    view!(<Text tag=TextTag::Code>{children}</Text>)
}

#[component]
pub fn BoldCaption<V: IntoView + 'static>(children: TypedChildren<V>) -> impl IntoView {
    let children = children.into_inner();
    bold_caption(children())
}
pub fn bold_caption(children: impl IntoView + 'static) -> impl IntoView {
    use thaw::Caption1Strong;
    view!(<Caption1Strong>{children}</Caption1Strong>)
}
 */
