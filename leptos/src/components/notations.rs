use crate::{config::FtmlConfig, utils::LocalCacheExt};
use ftml_dom::{
    ClonableView,
    notations::NotationExt,
    terms::ReactiveApplication,
    utils::local_cache::{GlobalLocal, LocalCache, SendBackend},
};
use ftml_ontology::narrative::elements::Notation;
use ftml_uris::{DocumentElementUri, LeafUri};
use leptos::prelude::*;

#[must_use]
pub fn has_notation<B: SendBackend>(
    uri: LeafUri,
    children: ClonableView,
    arguments: Option<ReadSignal<ReactiveApplication>>,
) -> impl IntoView + use<B> + 'static {
    use leptos::either::Either::{Left, Right};
    let notation = FtmlConfig::notation_preference(&uri);
    let finished = RwSignal::new(false);
    let _ = Effect::new(move || finished.set(true));

    move || {
        notation.get().map_or_else(
            || Left(children.clone().into_view::<crate::Views<B>>()),
            |notation| {
                if finished.try_get().is_some_and(|b| b) {
                    tracing::trace!(
                        "Replacing notation for {uri} with {:?} arguments",
                        arguments
                            .as_ref()
                            .map(|v| v.with_untracked(ReactiveApplication::len))
                    );
                    Right(with_notation::<B>(
                        uri.clone(),
                        notation,
                        arguments,
                        children.clone(),
                    ))
                } else {
                    Left(children.clone().into_view::<crate::Views<B>>())
                }
            },
        )
    }
}

#[must_use]
pub fn with_notation<B: SendBackend>(
    head: LeafUri,
    notation: DocumentElementUri,
    arguments: Option<ReadSignal<ReactiveApplication>>,
    children: ClonableView,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    let h = head.clone();
    LocalCache::with_or_toast::<B, _, _, _, _>(
        |c| c.get_notation(Some(h), notation),
        move |n| {
            match arguments {
                None => Left(n.as_op::<crate::Views<B>>(&head.into(), None)),
                Some(s) => {
                    let args = s.with(|s| {
                        if let ReactiveApplication::Closed(c) = s {
                            c.arguments.clone()
                        } else {
                            Vec::new()
                        }
                    });
                    Right(n.with_arguments::<crate::Views<B>, _>(&head.into(), None, &args))
                }
            }
            .attr("style", "border: 1px dotted red;")
        },
        move || children.into_view::<crate::Views<B>>(),
    )
}

pub fn notation_selector<Be: SendBackend>(uri: LeafUri) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    use thaw::Spinner;
    if !FtmlConfig::allow_notation_changes() {
        tracing::trace!("No notation changes");
        return Left(());
    }
    let leaf = uri.clone();
    let notations =
        LocalCache::resource::<Be, _, _>(move |b| async move { Ok(b.get_notations(leaf).await) });
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
            Some(Ok(v)) => B(do_notation_selector::<Be,_>(&uri, v)),
            Some(Err(e)) => C(format!("error: {e}")),
            None => D(view!(<Spinner/>)),
        }
    }}</Suspense>})
}

fn do_notation_selector<Be: SendBackend, E: std::fmt::Display>(
    uri: &LeafUri,
    notations: GlobalLocal<Vec<(DocumentElementUri, Notation)>, E>,
) -> impl IntoView + use<E, Be> {
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
                        not.as_view_safe::<crate::Views<Be>>(&head.into(),None)
                    );
                    view!(<ComboboxOption text="" value=not_uri.to_string()>
                        <math>{ notation}</math>
                    </ComboboxOption>)
                }).collect_view()}
            </Combobox>
        </div></div>
    }
}
