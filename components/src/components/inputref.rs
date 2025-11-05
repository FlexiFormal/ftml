use crate::{config::FtmlConfig, utils::LocalCacheExt};
use ftml_dom::{
    DocumentState, FtmlViews,
    counters::LogicalLevel,
    structure::Inputref,
    utils::{
        actions::{OneShot, SetOneShotDone},
        css::{CssExt, inject_css},
        local_cache::{LocalCache, SendBackend},
    },
};
use ftml_ontology::narrative::elements::SectionLevel;
use ftml_uris::{DocumentElementUri, DocumentUri};
use leptos::prelude::*;

#[must_use]
pub fn inputref<B: SendBackend>(info: Inputref) -> AnyView {
    use leptos::either::Either::{Left, Right};
    /*let Inputref {
        uri,
        target,
        replace,
        replacing_done,
        id,
        title,
        ..
    } = info;*/
    let lvl = DocumentState::current_section_level();
    let limit = FtmlConfig::autoexpand_limit();
    let replacing_done = info.done;
    tracing::debug!("inputref {} at level {lvl:?}", info.uri);
    let expand = Memo::new(move |_| {
        lvl <= limit.get().0
            || matches!(
                lvl,
                LogicalLevel::None | LogicalLevel::Section(SectionLevel::Part)
            )
            || replacing_done.was_clicked()
    });
    (move || {
        if expand.get() {
            do_replace::<B>(info.target.clone(), info.uri.clone(), replacing_done)
        } else {
            do_unreplaced::<B>(info.id.to_string(), &info, info.replace)
        }
    })
    .into_any()
}

fn do_unreplaced<B: SendBackend>(id: String, title: &Inputref, load: OneShot) -> AnyView {
    inject_css("ftml-inputref", include_str!("inputref.css"));
    view! {
        <div class="ftml-inputref" id=id on:click=move |_| load.activate()>
        {title.title::<crate::Views<B>>()}
        </div>
    }
    .into_any()
}

fn do_replace<B: SendBackend>(
    uri: DocumentUri,
    inputref: DocumentElementUri,
    on_load: SetOneShotDone,
) -> AnyView {
    let context = DocumentState::context_uri();
    let uri2 = uri.clone();
    tracing::debug!("expanding inputref {inputref}");
    LocalCache::with::<B, _, _>(
        |b| b.get_fragment(uri2.into(), Some(context)),
        move |(html, css, b)| {
            for c in css {
                c.inject();
            }
            DocumentState::inner_document(uri.clone(), &inputref, b, move || {
                crate::Views::<B>::render_ftml_and_then(html.into_string(), move || {
                    tracing::debug!("inputref expansion for {uri} finished!");
                    let _ = on_load.set();
                })
                .into_any()
            })
        },
    )
}
