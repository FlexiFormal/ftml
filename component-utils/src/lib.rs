#![recursion_limit = "256"]
#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

pub mod icons;
pub mod text;
pub mod theming;
use leptos::html::{div, span};
pub use text::*;
pub mod popover;
pub use popover::*;
pub mod toasts;
pub use toasts::*;

#[cfg(any(feature = "csr", feature = "hydrate"))]
pub mod events;

pub use thaw::{Avatar, Scrollbar, Tooltip};
pub use thaw::{Badge, BadgeAppearance, BadgeColor};
pub use thaw::{Button, ButtonAppearance, ButtonShape, ButtonSize};
pub use thaw::{
    Card, CardFooter, CardHeader, CardHeaderAction, CardHeaderDescription, CardHeaderProps,
    CardPreview,
};
pub use thaw::{Checkbox, Input, InputPrefix, InputType, Radio, RadioGroup};
pub use thaw::{Combobox, ComboboxOption, ComboboxOptionGroup};
pub use thaw::{Dialog, DialogBody, DialogContent, DialogSurface, ProgressBar};
pub use thaw::{
    DrawerBody, DrawerHeader, DrawerHeaderTitle, DrawerHeaderTitleAction, DrawerPosition,
    OverlayDrawer,
};
pub use thaw::{Flex, FlexAlign};
pub use thaw::{Grid, GridItem};
pub use thaw::{Layout, LayoutHeader, LayoutPosition, LayoutSider};
pub use thaw::{Menu, MenuItem, MenuPosition, MenuTrigger, MenuTriggerType, NavDrawer, NavItem};
pub use thaw::{Select, SelectSize};
pub use thaw::{Tab, TabList};
pub use thaw::{
    Table, TableBody, TableCell, TableCellLayout, TableHeader, TableHeaderCell, TableRow,
};
pub use thaw::{Tag, TagPicker, TagPickerControl, TagPickerGroup, TagPickerInput, TagPickerOption};

use leptos::prelude::*;
use std::borrow::Cow;

#[component]
pub fn Spinner(#[prop(default = false)] small: bool) -> impl IntoView {
    inject_css("ftml-viewer-spinner", include_str!("spinner.css"));
    let cls = if small {
        "ftml-viewer-spinner ftml-viewer-spinner--small"
    } else {
        "ftml-viewer-spinner"
    };
    div().class(cls).role("progressbar").child(
        span()
            .class("ftml-viewer-spinner__spinner")
            .child(span().class("ftml-viewer-spinner__spinner-tail")),
    )
}

#[component]
pub fn Divider() -> impl IntoView {
    inject_css("ftml-viewer-divider", include_str!("divider.css"));
    div()
        .class("ftml-viewer-divider")
        .aria_orientation("horizontal")
        .role("separator")
}

#[cfg(feature = "ssr")]
pub fn ssr_wrap<V: IntoView + 'static>(f: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
    use leptos::context::Provider;
    use thaw::ssr::SSRMountStyleProvider;
    view!(<SSRMountStyleProvider><Provider value=ssr::CssIds::default()>{
        f()
    }</Provider></SSRMountStyleProvider>)
}

#[inline]
pub fn inject_css(id: impl Into<Cow<'static, str>>, css: impl Into<Cow<'static, str>>) {
    do_inject_css(id.into(), css.into(), false);
}

#[inline]
pub fn inject_css_after(id: impl Into<Cow<'static, str>>, css: impl Into<Cow<'static, str>>) {
    do_inject_css(id.into(), css.into(), true);
}

#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::needless_pass_by_value)]
fn do_inject_css(id: Cow<'static, str>, content: Cow<'static, str>, after: bool) {
    #[cfg(feature = "ssr")]
    {
        ssr::CssIds::add(id, content);
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
        _ = if after {
            head.append_with_node_1(&style)
        } else {
            head.prepend_with_node_1(&style)
        };
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::borrow::Cow;

    #[cfg(feature = "ssr")]
    #[derive(Default, Clone)]
    pub struct CssIds(
        pub std::sync::Arc<std::sync::Mutex<rustc_hash::FxHashSet<Cow<'static, str>>>>,
    );
    #[cfg(feature = "ssr")]
    impl CssIds {
        pub fn add(id: Cow<'static, str>, content: Cow<'static, str>) {
            use leptos::prelude::*;
            if let Some(slf) = use_context::<Self>() {
                Self::add_i(&slf.0, id, content)
            } else {
                let owner = Self::top_owner();
                owner.with(move || {
                    let slf = Self::default();
                    Self::add_i(&slf.0, id, content);
                    provide_context(slf);
                });
            }
        }
        fn top_owner() -> leptos::reactive::owner::Owner {
            let mut o =
                leptos::reactive::owner::Owner::current().expect("Not in a reactive context");
            loop {
                if let Some(p) = o.parent() {
                    o = p;
                } else {
                    return o;
                }
            }
        }
        fn add_i(
            map: &std::sync::Mutex<rustc_hash::FxHashSet<Cow<'static, str>>>,
            id: Cow<'static, str>,
            content: Cow<'static, str>,
        ) {
            use leptos_meta::Style;
            if let Ok(mut ids) = map.lock() {
                if !ids.contains(&id) {
                    ids.insert(id.clone());
                    let _ = leptos::view!(<Style id=id>{content}</Style>);
                }
            }
        }
    }
}
