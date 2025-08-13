use crate::{config::FtmlConfig, utils::LocalCacheExt};
use ftml_dom::{
    DocumentState, FtmlViews,
    markers::InputrefInfo,
    utils::{
        actions::{OneShot, SetOneShotDone},
        css::{CssExt, inject_css},
        local_cache::{LocalCache, SendBackend},
    },
};
use ftml_uris::{DocumentElementUri, DocumentUri};
use leptos::prelude::*;

#[must_use]
pub fn inputref<B: SendBackend>(info: InputrefInfo) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    let InputrefInfo {
        uri,
        target,
        replace,
        replacing_done,
        id,
        title,
    } = info;
    let lvl = DocumentState::current_section_level();
    let limit = FtmlConfig::autoexpand_limit();
    tracing::debug!("inputref {uri} at level {lvl:?}");
    let expand = Memo::new(move |_| lvl <= limit.get().0 || replacing_done.was_clicked());
    move || {
        if expand.get() {
            Left(do_replace::<B>(target.clone(), uri.clone(), replacing_done))
        } else {
            Right(do_unreplaced::<B>(id.clone(), title, replace))
        }
    }
}

fn do_unreplaced<B: SendBackend>(
    id: String,
    title: RwSignal<String>,
    load: OneShot,
) -> impl IntoView {
    inject_css("ftml-inputref", include_str!("inputref.css"));
    view! {
        <div class="ftml-inputref" id=id on:click=move |_| load.activate()>
        {move || crate::Views::<B>::render_ftml(title.get())}
        </div>
    }
}

fn do_replace<B: SendBackend>(
    uri: DocumentUri,
    inputref: DocumentElementUri,
    on_load: SetOneShotDone,
) -> impl IntoView {
    let context = DocumentState::context_uri();
    let uri2 = uri.clone();
    tracing::debug!("expanding inputref {inputref}");
    LocalCache::with::<B, _, _, _>(
        |b| b.get_document_html(uri2, Some(context)),
        move |(html, css)| {
            for c in css {
                c.inject();
            }
            let r = DocumentState::inner_document(uri, &inputref, || {
                crate::Views::<B>::render_ftml(html)
            });
            let _ = on_load.set();
            r
        },
    )
}
