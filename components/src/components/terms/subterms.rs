use ftml_backend::BackendCheckResult;
use ftml_dom::{
    DocumentState,
    notations::TermExt,
    utils::{ModuleContext, local_cache::LocalCache},
};
use ftml_ontology::terms::{ComponentVar, Term};
use ftml_uris::NarrativeUri;
use leptos::prelude::*;
use leptos::web_sys::Element;

use crate::utils::ReactiveStore;

ftml_js_utils::split! {
    pub(super) fn with_subterm(t: ReadSignal<Option<Term>>, v: AnyView) -> AnyView {
        let Some(ti) = use_context::<thaw::ToasterInjection>() else {
            return v;
        };
        let selected = RwSignal::new(false);
        let current = std::cell::Cell::new(None);
        let owner = Owner::current().expect("not in a reactive context");
        let dialog_open = RwSignal::new(false);

        let ownercl = owner.child();
        let ownerclcl = ownercl.clone();
        Owner::on_cleanup(move || drop(ownerclcl));
        let _ = Effect::new(move || {
            use thaw::{Button, ButtonShape, ButtonSize, Toast, ToastBody, ToastTitle};
            ownercl.with(|| {
                if selected.get() && current.get().is_none() {
                    let id = uuid::Uuid::new_v4();
                    current.set(Some(id));
                    let body = {
                        let t = move || {
                            t.get()
                                .map(|t| t.into_view::<crate::Views>(crate::backend(), false))
                        };
                        view! {
                            <div>{ftml_dom::utils::math(move || t)}</div>
                            <div style="width:100%"><div style="margin-left:auto;"/>
                                <Button
                                    shape=ButtonShape::Rounded
                                    size=ButtonSize::Small
                                    on_click=move |_| dialog_open.set(true)
                                >"Details"</Button>
                            </div>
                        }
                    };
                    ti.dispatch_toast(
                        move || {
                            view! {
                                <Toast>
                                    <ToastTitle>"Selected subterm"</ToastTitle>
                                    <ToastBody>{body}</ToastBody>
                                </Toast>
                            }
                        },
                        thaw::ToastOptions::default()
                            .with_id(id)
                            .with_timeout(std::time::Duration::from_secs(0))
                            .with_intent(thaw::ToastIntent::Info)
                            .with_position(thaw::ToastPosition::BottomEnd),
                    );
                } else if !selected.get()
                    && let Some(id) = current.get()
                {
                    //leptos::logging::log!("dismissing subterm toast");
                    ti.dismiss_toast(id);
                    current.set(None);
                }
            });
        });
        subterm_dialog(v, t, owner, selected, dialog_open)
    }
}

#[allow(clippy::too_many_lines)]
fn subterm_dialog(
    v: AnyView,
    t: ReadSignal<Option<Term>>,
    owner: Owner,
    selected: RwSignal<bool>,
    dialog_open: RwSignal<bool>,
) -> AnyView {
    use thaw::{
        Dialog, DialogBody, DialogSurface, Table, TableBody, TableCell, TableCellLayout, TableRow,
        Tooltip,
    };

    let nv = v.directive(move |e| selection_listener(e, &owner, selected), ());

    let nt = move || {
        t.get()
            .map(|t| t.into_view::<crate::Views>(crate::backend(), false))
    };
    let ts = move || t.get().map(|t| format!("{:?}", t.debug_short()));
    let dialog = move || {
        let term = ftml_dom::utils::math(move || {
            move || {
                #[allow(clippy::option_if_let_else)]
                if let Some(ts) = ts() {
                    leptos::either::Either::Left(view! {<msup>{nt()}<Tooltip content = ts>
                        <mo>"🛈"</mo>
                    </Tooltip></msup>})
                } else {
                    leptos::either::Either::Right(nt())
                }
            }
        });
        let full_term = if let NarrativeUri::Element(uri) = DocumentState::current_uri() {
            //leptos::logging::log!("getting term at {uri}");
            Some((
                uri.clone(),
                LocalCache::resource(|r| r.get_document_term(crate::backend(), uri)),
            ))
        } else {
            None
        };
        let rendered_term = full_term.as_ref().map(|(_, res)| {
            let res = *res;
            move || {
                res.get().and_then(|r| {
                    r.ok().map(|t| {
                        let t = match t {
                            ::either::Left(t) => t.presentation(),
                            ::either::Right(t) => t.presentation(),
                        };
                        /*
                        let ts = move || t.get().map(|t| format!("{:?}", t.debug_short())); */
                        view!{
                            <TableRow>
                                <TableCell><TableCellLayout>"In full term: "</TableCellLayout></TableCell>
                                <TableCell><TableCellLayout>
                                    {ftml_dom::utils::math(move || {
                                        let ts = format!("{:?}",t.debug_short());
                                        view! {<msup>{
                                            ReactiveStore::render_term(t)
                                        }<Tooltip content = ts>
                                            <mo>"🛈"</mo>
                                        </Tooltip></msup>}
                                    })}
                                </TableCellLayout></TableCell>
                            </TableRow>
                        }
                    })
                })
            }
        });
        let inferred = full_term.as_ref().map(|(_, full_term)| {
            use thaw::Spinner;
            let full_term = *full_term;
            let sig = RwSignal::new(None);
            let context = ModuleContext::get_context().into_iter().collect::<Vec<_>>();
            Effect::new(move || {
                t.with(|t| {
                    /*leptos::logging::log!(
                        "Sending check for {:?}?",
                        t.as_ref().map(Term::debug_short)
                    );*/
                    full_term.with(|full_term| {
                        if let Some(sub) = t.as_ref() {
                            if let Some(full_term) = full_term {
                                match full_term {
                                    Ok(full_term) => {
                                        //leptos::logging::log!("Term is here!");
                                        let t = match full_term {
                                            ::either::Left(t) => t.get_parsed(),
                                            ::either::Right(t) => t.get_parsed(),
                                        };
                                        let fut = crate::backend().check_term(
                                            &context,
                                            either::Left(t),
                                            either::Left(sub),
                                        );
                                        leptos::task::spawn_local(async move {
                                            let r = fut.await;
                                            //leptos::logging::log!("Setting signal");
                                            sig.set(Some(r.map_err(|e| e.to_string())));
                                        });
                                        /*
                                        if let Some(path) = t.path_of_subterm(sub) {
                                            //leptos::logging::log!("Path: {path:?}");
                                            let fut =
                                                crate::backend().check_term(&context, t, &path);
                                            leptos::task::spawn_local(async move {
                                                let r = fut.await;
                                                //leptos::logging::log!("Setting signal");
                                                sig.set(Some(r.map_err(|e| e.to_string())));
                                            });
                                        } else {
                                            sig.set(Some(Err(
                                                "Failed to compute subterm path".to_string()
                                            )));
                                        }
                                         */
                                    }
                                    Err(e) => sig.set(Some(Err(e.to_string()))),
                                }
                            } else {
                                sig.set(Some(Err("no full term found".to_string())));
                                //leptos::logging::log!("full_term is None");
                            }
                        }
                    });
                });
            });
            move || {
                sig.get().map_or_else(
                    || {
                        leptos::either::Either::Left(view! {
                            <TableRow>
                                <TableCell><TableCellLayout><Spinner/></TableCellLayout></TableCell>
                            </TableRow>
                        })
                    },
                    |r| {
                        leptos::either::Either::Right(match r {
                            Ok(r) => leptos::either::Either::Left(check_result(r)),
                            Err(e) => leptos::either::Either::Right(view! {
                                <TableRow>
                                    <TableCell><TableCellLayout>
                                        <span style="color:red;">"Error"</span>
                                    </TableCellLayout></TableCell>
                                    <TableCell><TableCellLayout>
                                        {e}
                                    </TableCellLayout></TableCell>
                                </TableRow>
                            }),
                        })
                    },
                )
            }
        });
        view! {
            <Table>
                <TableBody>
                    <TableRow>
                        <TableCell><TableCellLayout>"Selected term: "</TableCellLayout></TableCell>
                        <TableCell><TableCellLayout>{term}</TableCellLayout></TableCell>
                    </TableRow>
                    {rendered_term}
                    {inferred}
                </TableBody>
            </Table>
        }
        .attr("style", "width:max-content;")
    };
    view! {
        {nv}
        <Dialog open=dialog_open><DialogSurface><DialogBody>
            //<DialogTitle>"Subterm"</DialogTitle>
            {dialog()}
        </DialogBody></DialogSurface></Dialog>
    }
    .into_any()
}

fn check_result(
    BackendCheckResult {
        context,
        inferred_type,
        simplified,
    }: BackendCheckResult,
) -> impl IntoView {
    use thaw::{TableCell, TableCellLayout, TableRow};
    #[allow(clippy::option_if_let_else)]
    let tp = if let Some(tp) = inferred_type {
        let tp = ftml_dom::utils::math(move || ReactiveStore::render_term(tp));
        leptos::either::Either::Left(view! {
            <TableRow>
                <TableCell><TableCellLayout>"Inferred type:"</TableCellLayout></TableCell>
                <TableCell><TableCellLayout>{tp}</TableCellLayout></TableCell>
            </TableRow>
        })
    } else {
        leptos::either::Either::Right(view! {
            <TableRow>
                <TableCell>""</TableCell>
                <TableCell><TableCellLayout>"(Type inferrence failed)"</TableCellLayout></TableCell>
            </TableRow>
        })
    };
    let simplified = view! {
        <TableRow>
            <TableCell><TableCellLayout>"Simplified:"</TableCellLayout></TableCell>
            <TableCell><TableCellLayout>
                {ftml_dom::utils::math(move || ReactiveStore::render_term(simplified))}
            </TableCellLayout></TableCell>
        </TableRow>
    };
    let ctx = if context.is_empty() {
        None
    } else {
        let mut iter = context.into_iter();
        // SAFETY: !context.is_empty()
        let first = cv(unsafe { iter.next().unwrap_unchecked() });
        let rest = iter.map(|v| view! {<mo>", "</mo>{cv(v)}}).collect_view();
        let inner = ftml_dom::utils::math(move || view! {<mrow>{first}{rest}</mrow>});
        Some(view! {
            <TableRow>
                <TableCell>""</TableCell>
                <TableCell><TableCellLayout>"...where " {inner}</TableCellLayout></TableCell>
            </TableRow>
        })
    };
    view! {{simplified}{tp}{ctx}}
}

fn cv(v: ComponentVar) -> impl IntoView {
    let tp = v.tp.map(|t| {
        let t = ReactiveStore::render_term(t);
        view! {<mo>":"</mo>{t}}
    });
    let df = v.df.map(|t| {
        let t = ReactiveStore::render_term(t);
        view! {<mo>":="</mo>{t}}
    });
    if tp.is_none() && df.is_none() {
        None
    } else {
        let var = ReactiveStore::render_term(Term::Var {
            variable: v.var,
            presentation: None,
        });
        Some(view! {{var}{tp}{df}})
    }
}

fn selection_listener(e: Element, owner: &Owner, is_selected: RwSignal<bool>) {
    #[cfg(any(feature = "csr", feature = "hydrate"))]
    {
        struct DocWrap {
            doc: send_wrapper::SendWrapper<leptos::web_sys::Document>,
            closure: send_wrapper::SendWrapper<
                leptos::wasm_bindgen::closure::Closure<dyn Fn(leptos::web_sys::Event)>,
            >,
        }
        impl DocWrap {
            fn new(doc: leptos::web_sys::Document) -> Self {
                use wasm_bindgen::JsCast;

                let closure = leptos::wasm_bindgen::closure::Closure::<dyn Fn(_)>::new(|_| {
                    let Some(e) = get_selection() else {
                        STORE.with_borrow(|o| {
                            if let Some((_, sigs)) = o {
                                for s in sigs {
                                    if s.1.get_untracked() {
                                        s.1.set(false);
                                    }
                                }
                            }
                        });
                        return;
                    };
                    STORE.with_borrow(|o| {
                        let Some((_, sigs)) = o else { return };
                        if let Some((_, sig)) = sigs.iter().find(|(a, _)| *a == e) {
                            sig.set(true);
                            for (n, s) in sigs {
                                if *n != e && s.get_untracked() {
                                    s.set(false);
                                }
                            }
                        } else {
                            for (_, s) in sigs {
                                s.set(false);
                            }
                        }
                    });
                });
                let _ = doc.add_event_listener_with_callback(
                    "selectionchange",
                    closure.as_ref().unchecked_ref(),
                );
                Self {
                    doc: send_wrapper::SendWrapper::new(doc),
                    closure: send_wrapper::SendWrapper::new(closure),
                }
            }
        }
        impl Drop for DocWrap {
            fn drop(&mut self) {
                use wasm_bindgen::JsCast;

                let _ = self.doc.remove_event_listener_with_callback(
                    "selectionchange",
                    self.closure.as_ref().unchecked_ref(),
                );
            }
        }
        thread_local! {
            static STORE:
                std::cell::RefCell<Option<(DocWrap, Vec<(Element,RwSignal<bool>)>)>>
            = const{ std::cell::RefCell::new(None) };
        }
        let Some(doc) = leptos::web_sys::window().and_then(|w| w.document()) else {
            return;
        };
        let e2 = e.clone();
        STORE.with_borrow_mut(move |o| {
            let vs = match o {
                None => &mut o.get_or_insert((DocWrap::new(doc), Vec::new())).1,
                Some((d, _)) if *d.doc != doc => {
                    let _ = o.take();
                    &mut o.get_or_insert((DocWrap::new(doc), Vec::new())).1
                }
                Some((_, v)) => v,
            };
            vs.push((e2, is_selected));
        });
        let e = send_wrapper::SendWrapper::new(e);
        owner.with(move || {
            Owner::on_cleanup(move || {
                STORE.with_borrow_mut(move |o| {
                    if let Some((_, vs)) = o {
                        vs.retain(|(i, _)| *i != *e);
                    }
                });
            });
        });
    }
}

#[cfg(any(feature = "csr", feature = "hydrate"))]
fn get_selection() -> Option<Element> {
    let window = leptos::web_sys::window()?;
    let Ok(Some(selection)) = window.get_selection() else {
        return None;
    };
    //leptos::web_sys::console::log_3(&"Range: ".into(), &anchor, &focus);
    selection.get_range_at(0).ok().and_then(|r| {
        r.common_ancestor_container()
            .ok()
            .and_then(|node| leptos::wasm_bindgen::JsCast::dyn_into(node).ok())
    })
}
