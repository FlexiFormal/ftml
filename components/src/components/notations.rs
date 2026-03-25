use crate::{config::FtmlConfig, utils::LocalCacheExt};
use ftml_dom::{
    ClonableView,
    notations::NotationExt,
    terms::ReactiveApplication,
    utils::local_cache::{GlobalLocal, LocalCache},
};
use ftml_ontology::{narrative::elements::Notation, terms::Term};
use ftml_uris::{DocumentElementUri, LeafUri};
use leptos::prelude::*;

ftml_js_utils::split! {
#[must_use]
pub fn has_notation(
    uri: LeafUri,
    children: ClonableView,
    arguments: Option<ReadSignal<ReactiveApplication>>,
) -> AnyView {
    use leptos::either::Either::{Left, Right};
    let notation = FtmlConfig::notation_preference(&uri);
    let finished = RwSignal::new(false);
    let _ = Effect::new(move || finished.set(true));

    (move || {
        notation.get().map_or_else(
            || Left(children.clone().into_view::<crate::Views>()),
            |notation| {
                if finished.try_get().is_some_and(|b| b) {
                    tracing::trace!(
                        "Replacing notation for {uri} with {:?} arguments",
                        arguments
                            .as_ref()
                            .map(|v| v.with_untracked(ReactiveApplication::len))
                    );
                    let term = arguments
                        .and_then(|s| s.with_untracked(ReactiveApplication::term).get_untracked());
                    Right(with_notation(
                        term,
                        uri.clone(),
                        notation,
                        arguments,
                        children.clone(),
                    ))
                } else {
                    Left(children.clone().into_view::<crate::Views>())
                }
            },
        )
    })
    .into_any()
}
}

#[must_use]
fn with_notation(
    term: Option<Term>,
    head: LeafUri,
    notation: DocumentElementUri,
    arguments: Option<ReadSignal<ReactiveApplication>>,
    children: ClonableView,
) -> AnyView {
    use leptos::either::Either::{Left, Right};
    let h = head.clone();
    LocalCache::with_or_toast(
        |c| c.get_notation(crate::backend(), Some(h), notation),
        move |n| {
            match arguments {
                None => {
                    Left(n.as_op::<crate::Views>(crate::backend(), &head.into(), None, i64::MAX))
                }
                Some(s) => {
                    let args = s.with(|s| {
                        if let ReactiveApplication::Closed(c) = s {
                            c.arguments.clone()
                        } else {
                            Vec::new()
                        }
                    });
                    Right(n.with_arguments::<crate::Views, _>(
                        crate::backend(),
                        term,
                        &head.into(),
                        None,
                        &args,
                        i64::MAX,
                    ))
                }
            }
            .attr("style", "border: 1px dotted red;")
            .into_any()
        },
        move || children.into_view::<crate::Views>(),
    )
}

pub fn notation_selector(uri: LeafUri) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    use thaw::Spinner;
    if !FtmlConfig::allow_notation_changes() {
        tracing::trace!("No notation changes");
        return Left(());
    }
    let leaf = uri.clone();
    let notations = LocalCache::resource(move |b| async move {
        Result::<_, ftml_backend::BackendError<String>>::Ok(
            b.get_notations(crate::backend(), leaf).await,
        )
    });
    Right(view! {<Suspense fallback = || view!(<Spinner/>)>{move || {
        use leptos::either::EitherOf4::{A, B, C, D};
        match notations.get() {
            Some(Ok(GlobalLocal { global, local }))
                if (global.is_none()
                    || global
                        .as_ref()
                        .is_some_and(|r| r.is_err() || r.as_ref().is_ok_and(Vec::is_empty)))
                    && local.is_none() =>
            {
                A(())
            }
            Some(Ok(v)) => B(do_notation_selector(&uri, v)),
            Some(Err(e)) => C(format!("error: {e}")),
            None => D(view!(<Spinner/>)),
        }
    }}</Suspense>})
}

fn do_notation_selector(
    uri: &LeafUri,
    notations: GlobalLocal<Vec<(DocumentElementUri, Notation)>, ftml_backend::BackendError<String>>,
) -> impl IntoView + use<> {
    use ftml_dom::notations::NotationExt;
    use leptos::prelude::*;
    use thaw::{Combobox, ComboboxOption};
    let mut all = notations.local.unwrap_or_default();
    match notations.global {
        None => (),
        Some(Err(e)) => {
            crate::utils::error_toast(e.to_string());
        }
        Some(Ok(v)) => {
            for (u, n) in v {
                if !all.iter().any(|(u2, _)| *u2 == u) {
                    all.push((u, n));
                }
            }
        }
    }
    let current = FtmlConfig::notation_preference_signal(uri);
    let string_signal = RwSignal::new(String::new());
    let mut has_changed = false;
    let _ = Effect::new(move || {
        if has_changed {
            let uri = string_signal.with(|s| {
                if s == "none" {
                    None
                } else {
                    Some(s.parse().expect("notation is not valid uri"))
                }
            });
            tracing::info!("setting preferred notation to {uri:?}");
            current.set(uri);
        } else {
            has_changed = true;
            let s = current.with(|o| {
                o.as_ref()
                    .map_or_else(|| "none".to_string(), DocumentElementUri::to_string)
            });
            string_signal.set(s);
            // make sure dependency is registered
            string_signal.with(|_| ());
        }
    });

    let head = uri.clone();
    view! {
        <div style="width:100%;"><div style="width:min-content;margin-left:auto;">
            <Combobox selected_options=string_signal placeholder="Force Notation">
                <ComboboxOption text="None" value="none">"None"</ComboboxOption>
                {all.into_iter().map(|(not_uri,not)| {
                    let head = head.clone();
                    let notation = FtmlConfig::disable_hovers(move ||
                        not.as_view_safe::<crate::Views>(crate::backend(),&head.into(),None).into_any()
                    );
                    view!(<ComboboxOption text="" value=not_uri.to_string()>
                        {ftml_dom::utils::math(|| notation)}
                    </ComboboxOption>)
                }).collect_view()}
            </Combobox>
        </div></div>
    }
}
