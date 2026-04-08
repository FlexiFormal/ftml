use ftml_backend::BackendError;
use ftml_dom::{
    FtmlViews,
    toc::TocSource,
    utils::{
        css::CssExt,
        local_cache::{GlobalLocal, LocalCache},
    },
};
use ftml_ontology::narrative::elements::ParagraphOrProblemKind;
use ftml_uris::{DocumentElementUri, IsNarrativeUri, SymbolUri, Uri, UriWithArchive};
use leptos::prelude::*;

use crate::{
    components::content::FtmlViewable,
    utils::{LocalCacheExt, ReactiveStore, collapsible::lazy_collapsible},
};

pub(super) fn formals(symbol: SymbolUri, uri: ReadSignal<Option<DocumentElementUri>>) -> AnyView {
    use thaw::Divider;
    view! {
        <div style="margin:5px;"><Divider/></div>
        {lazy_collapsible(Some(|| "Formal Details"), move ||view!{
            <div style="width:100%"><div style="width:fit-content;margin-left:auto;">
                "(in module "{symbol.module.as_view()}")"
            </div></div>
            {
                let sym = symbol.clone();
                let bol = symbol.clone();
                LocalCache::with_or_toast(
                    move |r| r.get_symbol(crate::backend(),sym),
                    |s| match s {
                        ::either::Left(s) => super::super::content::symbols::symbol_view(&s,false),
                        ::either::Right(s) => super::super::content::symbols::symbol_view(&s,false)
                    },
                    move || view!({format!("error getting uri {bol}")}<br/>).into_any()
            )}
            {move || { uri.with(|u| u.clone().map(|u|  {
               let uri = u.clone();
               view!("In term: "{LocalCache::with_or_toast(
                   |r| r.get_document_term(crate::backend(),u),
                   |t|  {
                       tracing::warn!("Rendering term {t:?}");
                       ftml_dom::utils::math(|| ReactiveStore::render_term(
                           match t {
                               ::either::Left(t) => t.presentation(),
                               ::either::Right(t) => t.presentation(),
                           }
                       )).into_any()
                   },
                   move || format!("error: {uri}").into_any()
               )})
            }
            ))}}
        })}
    }
    .into_any()
}

type Paras = Result<
    GlobalLocal<Vec<(DocumentElementUri, ParagraphOrProblemKind)>, BackendError<String>>,
    BackendError<String>,
>;

pub(super) fn paras_selector(
    paras: ReadSignal<Option<Paras>>,
    selected: RwSignal<Option<String>>,
) -> impl IntoView {
    use leptos::either::EitherOf3::{A, B, C};
    use thaw::{Combobox, ComboboxOption, ComboboxOptionGroup, Spinner};
    move || {
        paras.with(|p| match p {
            Some(Ok(v)) => {
                let mut definitions = Vec::new();
                let mut examples = Vec::new();
                for (uri, knd) in v.iter() {
                    match knd {
                        ParagraphOrProblemKind::Definition => definitions.push(uri.clone()),
                        ParagraphOrProblemKind::Example => examples.push(uri.clone()),
                        _ => (),
                    }
                }
                if let Some(d) = definitions.first() {
                    selected.set(Some(d.to_string()));
                }
                A(view! {
                    <Combobox selected_options=selected placeholder="Select Definition or Example">
                      <ComboboxOptionGroup label="Definitions">{
                          definitions.iter().map(|d| {
                            let line = para_line(d);
                            let value = d.to_string();
                            view!{
                              <ComboboxOption text="" value>{line}</ComboboxOption>
                            }
                        }).collect_view()
                      }</ComboboxOptionGroup>
                      <ComboboxOptionGroup label="Examples">{
                        examples.iter().map(|d| {
                          let line = para_line(d);
                          let value = d.to_string();
                          view!{
                            <ComboboxOption text="" value>{line}</ComboboxOption>
                          }
                        }).collect_view()
                      }</ComboboxOptionGroup>
                    </Combobox>
                })
            }
            Some(Err(e)) => B(view!(<span style="color:red;">{format!("error: {e}")}</span>)),
            None => C(view!(<Spinner/>)),
        })
    }
}

fn para_line(uri: &DocumentElementUri) -> impl IntoView + 'static {
    let archive = uri.archive_id().to_string();
    let name = uri.name().to_string();
    let lang = uri.language().flag_svg();
    view!(<div>
        <span>"["{archive}"] "{name}" "</span>
        <div style="display:contents;" inner_html=lang/>
    </div>)
}

pub(super) fn para_window(selected: RwSignal<Option<String>>) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    use thaw::Spinner;
    move || {
        selected
            .with(|u| {
                u.as_ref()
                    .and_then(|u| u.parse::<DocumentElementUri>().ok())
            })
            .map_or_else(
                || Right(view!(<Spinner/>)),
                |u| {
                    let uri = u.clone();
                    Left(LocalCache::with_or_toast(
                        |c| c.get_fragment(crate::backend(), Uri::DocumentElement(u), None),
                        |(html, css, stripped)| {
                            for c in css {
                                c.inject();
                            }
                            crate::Views::render_fragment(
                                Some(uri.into()),
                                crate::SidebarPosition::None,
                                stripped,
                                TocSource::None,
                                || crate::Views::render_ftml(html.into_string(), None).into_any(),
                            )
                        },
                        || view!(<span style="color:red;">"error"</span>).into_any(),
                    ))
                },
            )
    }
}
