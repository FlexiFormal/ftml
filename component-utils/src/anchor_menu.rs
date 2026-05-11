use leptos::{context::Provider, prelude::*};

#[component]
pub fn AnchorMenu(children: Children) -> impl IntoView {
    use crate::Scrollbar;
    crate::inject_css("ftml-viewer-anchor-menu", include_str!("anchor_menu.css"));

    let anchor_ref = NodeRef::new();
    let bar_ref = NodeRef::new();
    let element_ids = RwSignal::new(Vec::new());
    let active_id = RwSignal::new(None);

    #[cfg(any(feature = "csr", feature = "hydrate"))]
    {
        scroll_listener(element_ids, active_id);
    }

    let class = Memo::new(move |_| {
        if active_id.with(Option::is_some) {
            "ftml-viewer-anchor-rail__bar ftml-viewer-anchor-rail__bar--active"
        } else {
            "ftml-viewer-anchor-rail__bar"
        }
    });

    view! {
        <Scrollbar style="width:fit-content;max-height:500px;">
            <div class="ftml-viewer-anchor" node_ref=anchor_ref>
                <div class="ftml-viewer-anchor-rail">
                    <div
                        class=class
                        node_ref=bar_ref
                    ></div>
                </div>
                <Provider value=AnchorData {
                    anchor_ref,
                    bar_ref,
                    element_ids,
                    active_id,
                }>
                {children()}
                </Provider>
            </div>
        </Scrollbar>
    }
}

#[slot]
pub struct AnchorSubMenu {
    children: Children,
    #[prop(default = false)]
    collapsible: bool,
}

#[component]
pub fn AnchorMenuEntry(
    #[prop(into)] mut href: String,
    #[prop(optional)] anchor_sub_menu: Option<AnchorSubMenu>,
    children: Children,
) -> impl IntoView {
    let data: AnchorData = expect_context();
    let title_ref = NodeRef::<leptos::html::A>::new();
    if !href.starts_with('#') {
        href.insert(0, '#');
    }
    let active_id = data.active_id;
    data.append_id(href.clone());
    let memo_href = href.clone();
    let is_active = Memo::new(move |_| {
        active_id.with(|active_id| active_id.as_ref().is_some_and(|s| *s == memo_href))
    });
    let cleanup_href = href.clone();
    on_cleanup(move || data.remove_id(&cleanup_href));
    Effect::new(move |_| {
        let Some(title_el) = title_ref.get() else {
            return;
        };

        if is_active.get() {
            let title_rect = crate::js::get_true_rect(&title_el);
            data.update_background_position(&title_rect);
        }
    });
    let class = Memo::new(move |_| {
        if is_active.get() {
            "ftml-viewer-anchor-link ftml-viewer-anchor-link--active"
        } else {
            "ftml-viewer-anchor-link"
        }
    });

    let visible = if anchor_sub_menu.as_ref().is_some_and(|sm| sm.collapsible) {
        Some(RwSignal::new(true))
    } else {
        None
    };
    let sub_menu = move || {
        anchor_sub_menu.map(|sm| {
            if let Some(vis) = visible {
                leptos::either::Either::Left(crate::fancy_collapsible(
                    move || (sm.children)(),
                    vis,
                    "",
                    "",
                ))
            } else {
                leptos::either::Either::Right((sm.children)())
            }
        })
    };
    let collapse_anchor = visible.map(|visible| {
        view! {
            <span on:click=move |_| visible.set(!visible.get_untracked())>
                {crate::collapse_marker(visible,true)}
            </span>
            " "
        }
    });
    view! {
        <div class=class>
            <span>
                {collapse_anchor}
                <a
                    href=href
                    class="ftml-viewer-anchor-link__title"
                    node_ref=title_ref
                >
                    {children()}
                </a>
            </span>
            {sub_menu()}
        </div>
    }
}

#[derive(Clone, Copy)]
struct AnchorData {
    anchor_ref: NodeRef<leptos::html::Div>,
    bar_ref: NodeRef<leptos::html::Div>,
    element_ids: RwSignal<Vec<String>>,
    active_id: RwSignal<Option<String>>,
}
impl AnchorData {
    pub fn append_id(&self, id: String) {
        self.element_ids.update(|ids| {
            ids.push(id);
        });
    }

    pub fn remove_id(&self, id: &str) {
        self.element_ids.update(|ids| {
            if let Some(index) = ids.iter().position(|item_id| item_id == id) {
                ids.remove(index);
            }
        });
    }

    pub fn update_background_position(&self, title_rect: &leptos::web_sys::DomRect) {
        if let Some(anchor_el) = self.anchor_ref.get_untracked() {
            let bar_el = self
                .bar_ref
                .get_untracked()
                .expect("This should not happen");
            let anchor_rect = crate::js::get_true_rect(&anchor_el);

            let offset_top = title_rect.top() - anchor_rect.top();

            bar_el.style(("top", format!("{offset_top}px")));
            bar_el.style(("height", format!("{}px", title_rect.height())));
        }
    }
}

#[cfg(any(feature = "csr", feature = "hydrate"))]
fn scroll_listener(element_ids: RwSignal<Vec<String>>, active_id: RwSignal<Option<String>>) {
    use leptos::ev;

    let on_scroll = move || {
        element_ids.with(|ids| {
            let mut temp_link = None;
            for id in ids {
                if let Some(link_el) = document().get_element_by_id(&id[1..]) {
                    let top = link_el.get_bounding_client_rect().top();
                    if top >= 0.0 {
                        if top <= 50.0 {
                            temp_link = Some(id);
                        }
                        break;
                    }
                    temp_link = Some(id);
                } else if temp_link.is_some() {
                    break;
                }
            }
            active_id.set(temp_link.cloned());
        });
    };
    let cb = crate::js::throttle(
        move || {
            on_scroll();
        },
        std::time::Duration::from_millis(200),
    );
    let scroll_handle = crate::js::add_event_listener_with_bool(
        document(),
        ev::scroll,
        move |_| {
            cb();
        },
        true,
    );
    on_cleanup(move || {
        scroll_handle.remove();
    });
}
