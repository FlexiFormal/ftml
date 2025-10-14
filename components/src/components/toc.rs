use crate::{
    config::FtmlConfig,
    utils::{
        LocalCacheExt,
        collapsible::{collapse_marker, fancy_collapsible},
    },
};
use ftml_dom::{
    DocumentState, FtmlViews,
    toc::{CurrentTOC, NavElems, TocSource},
    utils::{
        css::{CssExt, inject_css},
        local_cache::{LocalCache, SendBackend},
    },
};
use ftml_ontology::{narrative::documents::TocElem, utils::time::Timestamp};
use ftml_uris::{DocumentElementUri, DocumentUri};
use leptos::prelude::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
/// A section that has been "covered" at the specified timestamp; will be marked accordingly
/// in the TOC.
pub struct TocProgress {
    pub uri: DocumentElementUri,
    #[serde(default)]
    pub timestamp: Option<Timestamp>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TocProgresses(pub Box<[TocProgress]>);
impl wasm_bindgen::convert::TryFromJsValue for TocProgresses {
    type Error = serde_wasm_bindgen::Error;
    fn try_from_js_value(value: leptos::wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        Ok(Self(serde_wasm_bindgen::from_value(value)?))
    }
}
impl ftml_js_utils::conversion::FromWasmBindgen for TocProgresses {}

#[derive(Default)]
struct Gottos {
    current: Option<TocProgress>,
    iter: std::vec::IntoIter<TocProgress>,
}
impl Gottos {
    fn next(&mut self, uri: &DocumentElementUri) {
        if let Some(c) = self.current.as_ref()
            && c.uri == *uri
        {
            loop {
                self.current = self.iter.next();
                if let Some(c) = &self.current {
                    if c.uri != *uri {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }

    fn new(v: TocProgresses, toc: &[TocElem]) -> Self {
        let mut v = v.0.into_vec();
        v.retain(|e| {
            toc.iter().any(|s| {
                if let TocElem::Section { uri, .. } = s {
                    *uri == e.uri
                } else {
                    false
                }
            })
        });
        let mut gotto_iter = v.into_iter();
        Self {
            current: gotto_iter.next(),
            iter: gotto_iter,
        }
    }
}

pub fn toc<Be: SendBackend>(uri: DocumentUri) -> impl IntoView {
    use leptos::either::EitherOf4::{A, B, C, D};
    let Some(toc) = FtmlConfig::with_toc_source(Clone::clone) else {
        return A(());
    };
    match toc {
        TocSource::None => A(()),
        TocSource::Get => B({
            let csr = RwSignal::new(false);
            let _ = Effect::new(move || {
                csr.set(true);
            });
            move || {
                let uri = uri.clone();
                if csr.get() {
                    Some(LocalCache::with_or_toast::<Be, _, _, _, _>(
                        move |c| c.get_toc(uri),
                        move |(css, toc)| {
                            for c in css {
                                c.inject();
                            }
                            let gottos: TocProgresses = use_context().unwrap_or_default();
                            let mut gottos = Gottos::new(gottos, &toc);
                            wrap_toc(move |data| {
                                CurrentTOC::set(toc.clone().into_vec());
                                let r = do_toc::<Be>(&toc, &mut gottos, data);
                                NavElems::retry();
                                r
                            })
                        },
                        || "error",
                    ))
                } else {
                    None
                }
            }
        }), // TODO
        TocSource::Extract => {
            let toc = DocumentState::get_toc();
            C(wrap_toc(move |data| {
                move || {
                    let gottos: TocProgresses = use_context().unwrap_or_default();
                    toc.with(|toc| {
                        toc.toc.as_ref().map(|v| {
                            let mut gottos = Gottos::new(gottos, v);
                            do_toc::<Be>(v, &mut gottos, data)
                        })
                    })
                }
            }))
        }
        TocSource::Ready(toc) => D(wrap_toc(move |data| {
            let gottos: TocProgresses = use_context().unwrap_or_default();
            let mut gottos = Gottos::new(gottos, &toc);
            do_toc::<Be>(&toc, &mut gottos, data)
        })),
    }
}

fn wrap_toc<V: IntoView + 'static>(body: impl FnOnce(AnchorData) -> V) -> impl IntoView {
    use thaw::Scrollbar;
    inject_css("ftml-toc", include_str!("toc.css"));
    //owned(move || {
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
    //})
}

fn do_toc<Be: SendBackend>(
    toc: &[TocElem],
    gottos: &mut Gottos,
    data: AnchorData,
) -> impl IntoView + use<Be> {
    use leptos::either::{
        Either::{Left, Right},
        EitherOf3::{A, B, C},
    };
    use thaw::Caption1Strong;

    toc.iter()
        .map(|toc_elem| match toc_elem {
            TocElem::Section {
                title,
                uri,
                id,
                children,
            } => {
                let style = if gottos.current.is_some() {
                    "background-color:var(--colorPaletteYellowBorder1);"
                } else {
                    ""
                };
                let after = gottos.current.as_ref().and_then(|e| e.timestamp).map(|ts| {
                    view! {
                        <sup><i>" Covered: "{ts.into_date().to_string()}</i></sup>
                    }
                });
                gottos.next(uri);
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
                /*let on_click = move |_| {
                    href.with_value(move |href_id| {
                        scroll_into_view(href_id, nav_elems);
                    });
                };*/
                let title = title.as_ref().map_or_else(
                    || Right(uri.name().last().to_string()),
                    |t| Left(crate::Views::<Be>::render_ftml(t.to_string(), None)),
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
                    let i = do_toc::<Be>(children, gottos, data).into_any();
                    let ch = fancy_collapsible(move || i, visible, "", "");
                    (Some(visible), Some(ch))
                } else {
                    (None, None)
                };

                A(view! {
                    <div class=class>
                        <Caption1Strong>
                            {visible.map(|visible|
                                view!{
                                    <a href on:click=move |_| visible.set(!visible.get_untracked())>
                                        {collapse_marker(visible,true)}
                                    </a>
                                    " "
                                }
                            )}
                            <a
                                href=href.get_value()
                                class="thaw-anchor-link__title"
                                //on:click=on_click
                                node_ref=title_ref
                                style=style
                            >
                                {title}{after}
                            </a>
                        </Caption1Strong>
                        {children}
                    </div>
                })
            }
            TocElem::Inputref { children, .. } | TocElem::SkippedSection { children } => {
                B(do_toc::<Be>(children, gottos, data).into_any())
            }
            _ => C(()),
        })
        .collect_view()
}

fn has_section(elems: &[TocElem]) -> bool {
    for e in elems {
        match e {
            TocElem::Section { .. } => return true,
            TocElem::Inputref { children, .. } | TocElem::SkippedSection { children }
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
                } else if temp_link.is_some() {
                    break;
                }
                /*else {
                id.with_value(|id| tracing::warn!("Element with id {id} disappeared!"));
                }*/
            }
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
