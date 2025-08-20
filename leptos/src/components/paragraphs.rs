use ftml_dom::utils::css::inject_css;
use ftml_uris::DocumentElementUri;
use leptos::prelude::*;

use crate::config::FtmlConfig;

pub fn paragraph<V: IntoView>(
    info: ftml_dom::markers::ParagraphInfo,
    then: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    inject_css("ftml-sections", include_str!("sections.css"));
    view! {
        <div class=info.class style=info.style>{
            FtmlConfig::wrap_paragraph(&info.uri,info.kind,then)
        }</div>
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn slide<V: IntoView>(
    uri: DocumentElementUri,
    then: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    use thaw::{Button, ButtonAppearance, ButtonShape, ButtonSize, Icon};
    inject_css("ftml-slide", include_str!("slides.css"));
    if FtmlConfig::allow_fullscreen() {
        let div = NodeRef::new();
        let inner = NodeRef::new();
        let slides: crate::Slides = expect_context();
        let index = slides.all.update_untracked(|v| {
            v.push(Slide {
                div,
                inner,
                #[cfg(target_family = "wasm")]
                closure: RwSignal::new(None),
            });
            v.len() - 1
        });
        Left(view! {
            <div style="display:flex;flex-direction:row;">
                <div node_ref=div class="ftml-slide"><div node_ref=inner>{
                    FtmlConfig::wrap_slide(&uri, then)
                }</div></div>
                <div><Button
                    size=ButtonSize::Small
                    appearance=ButtonAppearance::Subtle
                    shape=ButtonShape::Circular
                    on:click=move |_| slides.go(index)
                >
                    <Icon icon=icondata_ai::AiFullscreenOutlined/>
                </Button></div>
            </div>
        })
    } else {
        Right(FtmlConfig::wrap_slide(&uri, then).attr("class", "ftml-slide"))
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::type_complexity)]
pub(crate) struct Slides {
    all: RwSignal<Vec<Slide>>,
    current: RwSignal<Option<usize>>,
    #[cfg(target_family = "wasm")]
    #[allow(dead_code)]
    closure: StoredValue<
        send_wrapper::SendWrapper<
            leptos::wasm_bindgen::closure::Closure<dyn Fn(leptos::web_sys::Event)>,
        >,
    >,
    #[allow(dead_code)]
    top: NodeRef<leptos::html::Div>,
}
impl Slides {
    pub fn new() -> (impl IntoView, Self) {
        let current = RwSignal::new(None);
        let all = RwSignal::new(Vec::new());
        let top = NodeRef::new();
        Self::update_slide(all, current, top);
        #[cfg(target_family = "wasm")]
        let closure = Self::arrow_keys(all, current, top);
        (
            view!(<div node_ref=top class="ftml-slide" style="display:none;"/>),
            Self {
                all,
                current,
                top,
                #[cfg(target_family = "wasm")]
                closure: StoredValue::new(send_wrapper::SendWrapper::new(closure)),
            },
        )
    }

    fn update_slide(
        all: RwSignal<Vec<Slide>>,
        current: RwSignal<Option<usize>>,
        top: NodeRef<leptos::html::Div>,
    ) {
        #[cfg(target_family = "wasm")]
        use leptos::wasm_bindgen::JsCast;
        let is_fullscreen = RwSignal::new(None::<bool>);
        let _ = Effect::new(move || {
            let Some(index) = current.get() else { return };
            let Some(fullscreen) = is_fullscreen.get() else {
                return;
            };
            if let Some(top) = top.get() {
                if fullscreen {
                    all.with_untracked(|all| {
                        tracing::trace!("fullscreen: Move target node");
                        let current = all.get(index).expect("this is a bug");
                        current.mount(&top);
                    });
                } else {
                    tracing::trace!("fullscreen: remove target node");
                    current.update_untracked(|o| *o = None);
                    is_fullscreen.update_untracked(|o| *o = None);
                    all.with_untracked(|all| {
                        let current = all.get(index).expect("this is a bug");
                        current.unmount();
                    });
                }
            }
        });
        #[cfg(target_family = "wasm")]
        let top_one = move |_: leptos::web_sys::Event| {
            let Some(top) = top.get_untracked() else {
                tracing::trace!("fullscreen listener: to top");
                return;
            };
            if document().fullscreen_element().is_some_and(|e| e == **top) {
                tracing::trace!("fullscreen listener: set is_fullscreen=true");
                let _ = top.set_attribute("style", "");
                is_fullscreen.set(Some(true));
            } else {
                tracing::trace!("fullscreen listener: set is_fullscreen=false");
                let _ = top.set_attribute("style", "display:none;");
                is_fullscreen.set(Some(false));
            }
        };
        #[cfg(target_family = "wasm")]
        let f = StoredValue::new(send_wrapper::SendWrapper::new(
            leptos::wasm_bindgen::closure::Closure::wrap(Box::new(top_one) as Box<dyn Fn(_)>),
        ));
        #[cfg(target_family = "wasm")]
        top.on_load(move |top| {
            f.with_value(|f| {
                let _ = top.add_event_listener_with_callback(
                    "fullscreenchange",
                    f.as_ref().unchecked_ref(),
                );
            });
        });
    }

    #[allow(dead_code)]
    fn arrow_keys(
        all: RwSignal<Vec<Slide>>,
        current: RwSignal<Option<usize>>,
        top: NodeRef<leptos::html::Div>,
    ) -> leptos::wasm_bindgen::closure::Closure<dyn Fn(leptos::web_sys::Event)> {
        use leptos::wasm_bindgen::JsCast;
        let cl = leptos::wasm_bindgen::closure::Closure::<dyn Fn(_)>::new(
            move |e: leptos::web_sys::Event| {
                const ARROW_LEFT: u32 = 37;
                const ARROW_RIGHT: u32 = 39;
                let keyboard_event = e.dyn_into::<leptos::web_sys::KeyboardEvent>().expect("wut");
                if let Some(i) = current.get_untracked() {
                    match keyboard_event.key_code() {
                        ARROW_LEFT if i > 0 => {
                            let has_next = all.with_untracked(|all| {
                                all.get(i).is_some_and(|d: &Slide| {
                                    d.unmount();
                                    true
                                })
                            });
                            if has_next {
                                Self::go_i(i - 1, all, current, top);
                            }
                        }
                        ARROW_RIGHT => {
                            let has_next = all.with_untracked(|all| {
                                if all.len() > i + 1 {
                                    all.get(i).is_some_and(|d: &Slide| {
                                        d.unmount();
                                        true
                                    })
                                } else {
                                    false
                                }
                            });
                            if has_next {
                                Self::go_i(i + 1, all, current, top);
                            }
                        }
                        _ => (),
                    }
                }
            },
        );
        let _ = document().add_event_listener_with_callback("keydown", cl.as_ref().unchecked_ref());
        cl
    }
}

#[allow(clippy::type_complexity)]
#[allow(dead_code)]
pub(crate) struct Slide {
    div: NodeRef<leptos::html::Div>,
    inner: NodeRef<leptos::html::Div>,
    #[cfg(target_family = "wasm")]
    closure: RwSignal<
        Option<
            send_wrapper::SendWrapper<
                leptos::wasm_bindgen::closure::Closure<dyn Fn(leptos::web_sys::Event)>,
            >,
        >,
    >,
}
impl Slide {
    // may only be run *after* fullscreen is in effect; otherwise the scaling
    // will be screwed up
    pub fn mount(&self, top: &leptos::web_sys::HtmlDivElement) {
        let inner_e = self.inner.get_untracked().expect("this is a bug");
        let original_width = inner_e.client_width();
        let new_width = top.client_width() - 15; // padding
        let scale = new_width / original_width;
        /*
        let div_e = self.div.get_untracked().expect("this is a bug");
        (**top).style().set_css_text(
            &document()
                .default_view()
                .expect("this is a bug")
                .get_computed_style_with_pseudo_elt(&div_e, "")
                .expect("this is a bug")
                .expect("this is a bug")
                .css_text(),
        ); */
        let _ = inner_e.set_attribute(
            "style",
            &format!("transform-origin:top left;scale:{scale};width:{original_width}px;"),
        );
        tracing::trace!("fullscreen listener: appending to top");
        let _ = top.append_child(&inner_e);
    }
    pub fn unmount(&self) {
        let inner_e = self.inner.get_untracked().expect("this is a bug");
        let div_e = self.div.get_untracked().expect("this is a bug");
        let _ = div_e.append_child(&inner_e);
        let _ = inner_e.set_attribute("style", "");
    }
}

impl Slides {
    #[inline]
    pub fn go(self, index: usize) {
        Self::go_i(index, self.all, self.current, self.top);
    }

    #[inline(never)]
    #[allow(unused_variables)]
    #[allow(clippy::missing_const_for_fn)]
    fn go_i(
        index: usize,
        all: RwSignal<Vec<Slide>>,
        current: RwSignal<Option<usize>>,
        top: NodeRef<leptos::html::Div>,
    ) {
        #[cfg(target_family = "wasm")]
        if document().fullscreen_element().is_none() {
            current.set(Some(index));
            let Some(top) = top.get_untracked() else {
                return;
            };
            tracing::trace!("Requesting fullscreen");
            if top.request_fullscreen().is_err() {
                tracing::error!("Error setting fullscreen!");
            }
        } else if let Some(old) = current.get_untracked() {
            all.with_untracked(|all| {
                let current = all.get(index).expect("this is a bug");
                current.unmount();
            });
            current.set(Some(index));
        }
    }
}
