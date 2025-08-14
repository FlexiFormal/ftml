use crate::{
    config::FtmlConfig,
    utils::collapsible::{collapse_marker, fancy_collapsible},
};
use ftml_dom::{
    DocumentState, FtmlViews,
    toc::{TOCElem, TocSource},
    utils::{css::inject_css, local_cache::SendBackend, owned},
};
use leptos::prelude::*;

pub fn toc<Be: SendBackend>() -> impl IntoView {
    use leptos::either::EitherOf4::{A, B, C, D};
    let Some(toc) = FtmlConfig::with_toc_source(Clone::clone) else {
        return A(());
    };
    match toc {
        TocSource::None => A(()),
        TocSource::Get => B(()), // TODO
        TocSource::Extract => {
            let toc = DocumentState::get_toc();
            C(wrap_toc(move |data| {
                move || toc.with(|toc| toc.toc.as_ref().map(|v| do_toc::<Be>(v, data)))
            }))
        } // TODO
        TocSource::Ready(toc) => D(wrap_toc(move |data| do_toc::<Be>(&toc, data))), // TODO
    }
}

fn wrap_toc<V: IntoView + 'static>(body: impl FnOnce(AnchorData) -> V) -> impl IntoView {
    use thaw::Scrollbar;
    inject_css("ftml-toc", include_str!("toc.css"));
    // TODO gottos
    let anchor_ref = NodeRef::new();
    let bar_ref = NodeRef::new();
    let element_ids = RwSignal::new(Vec::new());
    let active_id = RwSignal::new(None);

    #[cfg(any(feature = "csr", feature = "hydrate"))]
    {
        scroll_listener(element_ids, active_id);
    }

    let body = body(AnchorData {
        anchor_ref,
        bar_ref,
        element_ids,
        active_id,
    });
    let class = Memo::new(move |_| {
        if active_id.with(Option::is_some) {
            "thaw-anchor-rail__bar thaw-anchor-rail__bar--active"
        } else {
            "thaw-anchor-rail__bar"
        }
    });
    view! {
        <Scrollbar style="width:fit-content;max-height:500px;">
            <div class="thaw-anchor" node_ref=anchor_ref>
                <div class="thaw-anchor-rail">
                    <div
                        class=class
                        node_ref=bar_ref
                    ></div>
                </div>
                {body}
            </div>
        </Scrollbar>
    }
}

fn do_toc<Be: SendBackend>(toc: &[TOCElem], data: AnchorData) -> impl IntoView + use<Be> {
    use leptos::either::{
        Either::{Left, Right},
        EitherOf3::{A, B, C},
    };
    use thaw::Caption1Strong;
    toc.iter()
        .map(|toc_elem| match toc_elem {
            TOCElem::Section {
                title,
                uri,
                id,
                children,
            } => A(owned(|| {
                let href = StoredValue::new(format!("#{id}"));
                let title_ref = NodeRef::<leptos::html::A>::new();
                let is_active = Memo::new(move |_| {
                    data.active_id.with(|active_id| {
                        active_id
                            .as_ref()
                            .is_some_and(|s| href.with_value(|v| s.with_value(|s| s == v)))
                    })
                });
                data.append_id(href);
                on_cleanup(move || href.with_value(|s| data.remove_id(s)));
                Effect::new(move |_| {
                    let Some(title_el) = title_ref.get() else {
                        return;
                    };

                    if is_active.get() {
                        let title_rect = ftml_dom::utils::get_true_rect(&title_el);
                        data.update_background_position(&title_rect);
                    }
                });
                let on_click = move |_| {
                    href.with_value(move |href_id| {
                        scroll_into_view(href_id);
                    });
                };
                let title = title.as_ref().map_or_else(
                    || Right(uri.name().last().to_string()),
                    |t| Left(crate::Views::<Be>::render_ftml(t.to_string())),
                );
                let class = Memo::new(move |_| {
                    if is_active.get() {
                        "thaw-anchor-link thaw-anchor-link--active"
                    } else {
                        "thaw-anchor-link"
                    }
                });

                let (visible, children) = if has_section(children) {
                    let visible = RwSignal::new(true);
                    let ch =
                        fancy_collapsible(move || do_toc::<Be>(children, data), visible, "", "");
                    (Some(visible), Some(ch))
                } else {
                    (None, None)
                };

                view! {
                    <div class=class>
                        <Caption1Strong>
                            {visible.map(|visible|
                                view!{
                                    <a on:click=move |_| visible.set(!visible.get_untracked())>
                                        {collapse_marker(visible)}
                                    </a>
                                    " "
                                }
                            )}
                            <a
                                href=href.get_value()
                                class="thaw-anchor-link__title"
                                on:click=on_click
                                node_ref=title_ref
                            >
                                {title}
                            </a>
                        </Caption1Strong>
                        {children}
                    </div>
                }
            })
            .into_any()),
            TOCElem::Inputref { children, .. } | TOCElem::SkippedSection { children } => {
                B(do_toc::<Be>(children, data).into_any())
            }
            _ => C(()),
        })
        .collect_view()
}

fn has_section(elems: &[TOCElem]) -> bool {
    for e in elems {
        match e {
            TOCElem::Section { .. } => return true,
            TOCElem::Inputref { children, .. } | TOCElem::SkippedSection { children }
                if has_section(children) =>
            {
                return true;
            }
            _ => (),
        }
    }
    false
}

#[cfg(any(feature = "csr", feature = "hydrate"))]
fn scroll_listener(
    element_ids: RwSignal<Vec<StoredValue<String>>>,
    active_id: RwSignal<Option<StoredValue<String>>>,
) {
    use leptos::ev;
    use thaw_utils::{add_event_listener_with_bool, throttle};
    /*
    struct LinkInfo {
        top: f64,
        id: StoredValue<String>,
    }
    impl std::fmt::Debug for LinkInfo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.id.with_value(|id| write!(f, "{}|{id}", self.top))
        }
    }
     */

    let on_scroll = move || {
        element_ids.with(|ids| {
            let mut temp_link = None;
            for id in ids {
                if let Some(link_el) = id.with_value(|id| document().get_element_by_id(&id[1..])) {
                    let top = link_el.get_bounding_client_rect().top(); //ftml_dom::utils::get_true_rect(&link_el);
                    if top >= 0.0 {
                        if top <= 50.0 {
                            temp_link = Some(*id);
                        }
                        break;
                    }
                    temp_link = Some(*id);
                } else {
                    id.with_value(|id| tracing::warn!("Element with id {id} disappeared!"));
                }
            }
            /* This assumes the elements are not already in order:

            let mut links: Vec<LinkInfo> = vec![];
            for id in ids {
                if let Some(link_el) = id.with_value(|id| document().get_element_by_id(&id[1..])) {
                    let link_rect = link_el.get_bounding_client_rect(); //ftml_dom::utils::get_true_rect(&link_el);
                    links.push(LinkInfo {
                        top: link_rect.top(),
                        id: *id,
                    });
                } else {
                    id.with_value(|id| tracing::warn!("Element with id {id} disappeared!"));
                }
            }
            links.sort_by(|a, b| a.top.total_cmp(&b.top));

            let mut temp_link = None::<LinkInfo>;
            for link in links {
                if link.top >= 0.0 {
                    if link.top <= 12.0 {
                        temp_link = Some(link);
                        break;
                    } else if temp_link.is_some() {
                        break;
                    }
                    temp_link = None;
                } else {
                    temp_link = Some(link);
                }
            }
            active_id.set(temp_link.map(|link| link.id));
             */
            active_id.set(temp_link);
        });
    };
    let cb = throttle(
        move || {
            on_scroll();
        },
        std::time::Duration::from_millis(200),
    );
    let scroll_handle = add_event_listener_with_bool(
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

fn scroll_into_view(id: &str) {
    let Some(link_el) = document().get_element_by_id(id) else {
        return;
    };
    link_el.scroll_into_view();
}

#[derive(Clone, Copy)]
struct AnchorData {
    anchor_ref: NodeRef<leptos::html::Div>,
    bar_ref: NodeRef<leptos::html::Div>,
    element_ids: RwSignal<Vec<StoredValue<String>>>,
    active_id: RwSignal<Option<StoredValue<String>>>,
}
impl AnchorData {
    pub fn append_id(&self, id: StoredValue<String>) {
        self.element_ids.update(|ids| {
            ids.push(id);
        });
    }

    pub fn remove_id(&self, id: &str) {
        self.element_ids.update(|ids| {
            if let Some(index) = ids
                .iter()
                .position(|item_id| item_id.with_value(|v| v == id))
            {
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
            let anchor_rect = ftml_dom::utils::get_true_rect(&anchor_el);

            let offset_top = title_rect.top() - anchor_rect.top();

            bar_el.style(("top", format!("{offset_top}px")));
            bar_el.style(("height", format!("{}px", title_rect.height())));
        }
    }
}

/*
{
    gottos.next(&uri);
    let id = format!("#{id}");
    let ch = children
        .into_iter()
        .map(|e| e.into_view(gottos))
        .collect_view();
    Some(Either::Left(view! {
      <AnchorLink href=id>
        <Header slot>
          <div style=style><DomStringCont html=title cont=crate::iterate/>{after}</div>
        </Header>
        {ch}
      </AnchorLink>
    }))
}
Self::Section {
    title: None,
    children,
    uri,
    ..
} => {
    gottos.next(&uri);
    Some(Either::Right(
        children
            .into_iter()
            .map(|e| e.into_view(gottos))
            .collect_view()
            .into_any(),
    ))
}
 */
