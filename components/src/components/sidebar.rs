use crate::{
    components::content::FtmlViewable,
    config::HighlightStyle,
    utils::{
        LocalCacheExt,
        collapsible::{collapse_marker, fancy_collapsible},
    },
};
use ftml_backend::FtmlBackend;
use ftml_dom::{
    DocumentState,
    utils::{
        css::inject_css,
        get_true_rect,
        local_cache::{LocalCache, SendBackend},
    },
};
use ftml_uris::DocumentUri;
use leptos::{prelude::*, web_sys::HtmlDivElement};

#[allow(clippy::fn_params_excessive_bools)]
pub fn do_sidebar<B: SendBackend, Ch: IntoView + 'static>(
    uri: DocumentUri,
    show_content: bool,
    pdf_link: bool,
    choose_highlight_style: bool,
    floating: bool,
    children: impl FnOnce() -> Ch,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    inject_css("ftml-sidebar", include_str!("./sidebar.css"));

    if floating {
        Left(floating_sidebar::<B, _>(
            uri,
            show_content,
            pdf_link,
            choose_highlight_style,
            children,
        ))
    } else {
        Right(flex_sidebar::<B, _>(
            uri,
            show_content,
            pdf_link,
            choose_highlight_style,
            children,
        ))
    }
}

fn flex_sidebar<B: SendBackend, Ch: IntoView + 'static>(
    uri: DocumentUri,
    show_content: bool,
    pdf_link: bool,
    choose_highlight_style: bool,
    children: impl FnOnce() -> Ch,
) -> impl IntoView {
    use thaw::{Button, ButtonShape, ButtonSize, Caption1, Flex};

    let visible = RwSignal::new(true);
    let body = fancy_collapsible(
        move || {
            view! {
                {if choose_highlight_style {Some(select_highlighting())} else {None}}
                <Flex>
                    {if show_content {Some(content_drawer::<B>())} else {None}}
                    {if pdf_link {Some(pdf::<B>())} else {None}}
                </Flex>
                {super::toc::toc::<B>(uri)}
            }
        },
        visible,
        "ftml-sidebar-inner",
        "width:fit-content;",
    );

    let children = view!(<div>{children()}</div>);

    view! {
        <Flex>
        {children}
        <div class="ftml-sidebar">
            <div style="position:fixed;">
                <Button
                    shape=ButtonShape::Rounded
                    size=ButtonSize::Small
                    on_click=move |_| visible.set(!visible.get_untracked())
                >
                    <Caption1>{collapse_marker(visible,true)}"FTML"</Caption1>
                </Button>
                {body}
            </div>
        </div>
        </Flex>
    }
}

fn floating_sidebar<B: SendBackend, Ch: IntoView + 'static>(
    uri: DocumentUri,
    show_content: bool,
    pdf_link: bool,
    choose_highlight_style: bool,
    children: impl FnOnce() -> Ch,
) -> impl IntoView {
    use thaw::{Button, ButtonShape, ButtonSize, Caption1, Flex};

    let pos_ref = NodeRef::new();
    let sidebar_ref = NodeRef::new();
    let _ = Effect::new(move || {
        if let Some(pos_ref) = pos_ref.get()
            && let Some(sidebar_ref) = sidebar_ref.get()
        {
            position_sidebar(&pos_ref, &sidebar_ref);
        }
    });
    let visible = RwSignal::new(true);
    let body = fancy_collapsible(
        move || {
            view! {
                {if choose_highlight_style {Some(select_highlighting())} else {None}}
                <Flex>
                    {if show_content {Some(content_drawer::<B>())} else {None}}
                    {if pdf_link {Some(pdf::<B>())} else {None}}
                </Flex>
                {super::toc::toc::<B>(uri)}
            }
        },
        visible,
        "ftml-sidebar-inner",
        "",
    );

    view! {
        <div style="display:contents;" node_ref=pos_ref>
            {children()}
        </div>
        <div class="ftml-sidebar" node_ref=sidebar_ref>
            <div>
                <Button
                    shape=ButtonShape::Rounded
                    size=ButtonSize::Small
                    on_click=move |_| visible.set(!visible.get_untracked())
                >
                    <Caption1>{collapse_marker(visible,true)}"FTML"</Caption1>
                </Button>
                {body}
            </div>
        </div>
    }
}

fn position_sidebar(position: &HtmlDivElement, sidebar: &HtmlDivElement) {
    use leptos::wasm_bindgen::JsCast;

    // hacky: insert sidebar next to the first "reasonable" container:
    let mut parent = if let Ok(Some(e)) = position.query_selector("main") {
        e
    } else {
        // SAFETY: we added the node above
        unsafe { position.first_element_child().unwrap_unchecked() }
    };
    while {
        parent.get_bounding_client_rect().width() < 10.0
    } //&& parent.child_element_count() == 1
        && let Some(fc) = max_child(&parent)
    //parent.first_element_child()
    {
        parent = fc;
    }
    // first, add it to the end; since width=100%, this will get us a reasonable actual width of
    // the container, which we use as margin-left
    let _ = parent.append_child(sidebar);
    //let rect = get_true_rect(sidebar);
    let _ = sidebar.set_attribute(
        "style",
        &format!(
            "width:fit-content;margin-left:{}px",
            parent.get_bounding_client_rect().width()/*rect.width()*/ + 50.0
        ),
    );
    // then move to the beginning and make use of display:sticky;
    // SAFETY: HtmlDivElements are Elements
    let _ = unsafe {
        parent.insert_before(
            sidebar.dyn_ref().unwrap_unchecked(),
            parent.first_child().as_ref(),
        )
    };
}

fn max_child(e: &leptos::web_sys::Element) -> Option<leptos::web_sys::Element> {
    use leptos::wasm_bindgen::JsCast;
    let mut curr = None::<leptos::web_sys::Element>;
    let mut i = 0;
    let nodes = e.child_nodes();
    while let Some(c) = nodes.get(i) {
        i += 1;
        if let Ok(e) = c.dyn_into::<leptos::web_sys::Element>() {
            if let Some(c) = &mut curr {
                if c.get_bounding_client_rect().width() < e.get_bounding_client_rect().width() {
                    *c = e;
                }
            } else {
                curr = Some(e);
            }
        }
    }
    curr
}

fn content_drawer<B: SendBackend>() -> impl IntoView {
    use thaw::{
        Button, ButtonAppearance, DrawerBody, DrawerHeader, DrawerHeaderTitle,
        DrawerHeaderTitleAction, DrawerPosition, Icon, OverlayDrawer, Popover, PopoverTrigger,
        Text,
    };

    let uri = DocumentState::document_uri();
    if uri == *DocumentUri::no_doc() {
        return None;
    }
    let uricl = uri.clone();
    let uricl2 = uri.clone();
    inject_css("ftml-content-drawer", include_str!("content.css"));
    let open = RwSignal::new(false);
    let title = RwSignal::new("...".to_string());

    Some(view! {
        <Button
            attr:title="Show Content"
           appearance=ButtonAppearance::Subtle
           on_click=move |_| open.set(true)
        >
           <Icon
                icon=icondata_bi::BiBookContentRegular
                height="1.5em".to_string()
                width="1.5em".to_string()
            />
        </Button>
        <OverlayDrawer class="ftml-drawer-absolute-wide" open position=DrawerPosition::Right>
            <DrawerHeader>
                <DrawerHeaderTitle>
                    <DrawerHeaderTitleAction slot>
                        <Button
                        appearance=ButtonAppearance::Subtle
                        on_click=move |_| open.set(false)>
                        "x"
                        </Button>
                    </DrawerHeaderTitleAction>
                    <Popover>
                        <PopoverTrigger slot>
                            <span inner_html={let uri = uricl2; move || {
                                let ttl = title.get();
                                if ttl.is_empty() { uri.name.to_string()} else {ttl}
                            }}/>
                        </PopoverTrigger>
                        <Text>{uricl.to_string()}</Text>
                    </Popover>
                </DrawerHeaderTitle>
            </DrawerHeader>
            <DrawerBody>
                {move || if open.get() { let uri= uri.clone(); Some(LocalCache::with_or_toast::<B,_,_,_,_>(
                    move |b| b.get_document(uri), move |d| {
                        if let Some(t) = &d.title {
                            title.set(t.to_string());
                        }
                        d.as_view::<B>()
                    },
                    || "error"
                ))} else { None }
            }</DrawerBody>
        </OverlayDrawer>
    })
}

fn pdf<B: SendBackend>() -> impl IntoView {
    use thaw::{Button, ButtonAppearance, Icon};

    let uri = DocumentState::document_uri();
    if uri == *DocumentUri::no_doc() {
        return None;
    }
    B::get().resource_link_url(&uri, "pdf").map(|url| {
        view! {
            <a target="_blank" href=url ><Button
                attr:title="Download PDF"
               appearance=ButtonAppearance::Subtle>
               <Icon
                    icon=icondata_bs::BsFiletypePdf
                    height="1.5em".to_string()
                    width="1.5em".to_string()
                />
            </Button></a>
        }
    })
}

fn select_highlighting() -> impl IntoView {
    use thaw::{Select, SelectSize, Text};
    let highlight = expect_context::<RwSignal<HighlightStyle>>();
    let value = RwSignal::new(highlight.get_untracked().as_str().to_string());
    Effect::new(move || {
        if let Some(v) = HighlightStyle::from_str(&value.get())
            && highlight.get_untracked() != v
        {
            highlight.set(v);
        }
    });
    view! {
        <Text>"Symbol Highlighting: "</Text>
        <Select value default_value = value.get_untracked() size=SelectSize::Small>
            <option class="ftml-comp">{HighlightStyle::Colored.as_str()}</option>
            <option class="ftml-comp-subtle">{HighlightStyle::Subtle.as_str()}</option>
            <option>{HighlightStyle::Off.as_str()}</option>
        </Select>
    }
}
