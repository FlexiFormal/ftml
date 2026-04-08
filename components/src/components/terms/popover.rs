use ftml_dom::{
    DocumentState, FtmlViews,
    notations::TermExt,
    utils::{
        css::{CssExt, inject_css},
        local_cache::LocalCache,
    },
};
use ftml_ontology::terms::{Term, VarOrSym, Variable};
use ftml_uris::{DocumentElementUri, Id, SymbolUri};
use leptos::prelude::*;

use crate::{
    components::content::FtmlViewable,
    utils::{LocalCacheExt, ReactiveStore},
};

//#[component]
pub fn term_popover(head: VarOrSym) -> AnyView {
    match head {
        VarOrSym::Var(Variable::Name { name, notated }) => unresolved_var_popover(name, notated),
        VarOrSym::Var(Variable::Ref {
            declaration,
            is_sequence,
        }) => resolved_var_popover(declaration, is_sequence.unwrap_or_default()),
        VarOrSym::Sym(uri) => symbol_popover(uri),
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn unresolved_var_popover(name: Id, notated: Option<Id>) -> AnyView {
    view! {
        <div>
            "Variable: " {notated.map_or_else(|| name.to_string(),|n| n.to_string())}
        </div>
    }
    .into_any()
}

pub fn resolved_var_popover(uri: DocumentElementUri, is_sequence: bool) -> AnyView {
    use thaw::Text;
    let title = if is_sequence {
        "Variable Sequence "
    } else {
        "Variable "
    };
    let declaration = uri.clone();
    let tm = ftml_dom::utils::math(move || {
        ReactiveStore::render_term(Term::Var {
            presentation: None,
            variable: Variable::Ref {
                declaration,
                is_sequence: Some(is_sequence),
            },
        })
    });
    let header = view!({title}{tm}" ("{uri.name().to_string()}")");
    view! {<div class="ftml-symbol-popup">
        <Text>{header}</Text>
        {LocalCache::with(|b| b.get_variable(crate::backend(),uri),|v| {
            let v = match &v {
                either::Either::Left(v) => v,
                either::Either::Right(v) => &**v
            };
            let tp = v.data.tp.presentation();
            let df = v.data.df.presentation();
            view! {
                {df.map(|df| {
                    let v = view!{"defined as "
                        {
                            let t = df.into_view::<crate::Views>(crate::backend(),false);
                            ftml_dom::utils::math(move || t)
                        }};
                    view!{<div><Text>{v}</Text></div>}
                })}
                {tp.map(|tp| {
                    let v = view!{"of type "
                        {
                            let t = tp.into_view::<crate::Views>(crate::backend(),false);
                            ftml_dom::utils::math(move || t)
                        }
                    };
                    view!{<div><Text>{v}</Text></div>}
                })}
            }.into_any()
        })}
    </div>}
    .into_any()
}

pub fn symbol_popover(uri: SymbolUri) -> AnyView {
    inject_css("ftml-symbol-popup", include_str!("popup.css"));
    let context = DocumentState::context_uri();
    LocalCache::with(
        |b| b.get_definition(crate::backend(), uri, Some(context)),
        |(html, css, _)| {
            for c in css {
                c.inject();
            }
            view! {
              <div class="ftml-symbol-popup">
                {
                    DocumentState::no_document(
                        || crate::Views::render_ftml(html.into_string(),None)
                    )
                }
              </div>
            }
            .into_any()
        },
    )
}

ftml_js_utils::split! {
    pub(crate) fn do_onclick(
        vos: VarOrSym,
        top_term: ReadSignal<Option<DocumentElementUri>>,
        allow_formals: ReadSignal<bool>
    ) -> AnyView {
        use leptos::prelude::*;
        use thaw::Divider;
        let s = match vos {
            VarOrSym::Var(Variable::Name {
                notated: Some(n), ..
            }) => {
                return view! {<span>"Variable "{n.to_string()}</span>}.into_any();
            }
            VarOrSym::Var(Variable::Name { name, notated }) => {
                return
                    view! {<span>"Variable "{notated.as_ref().map_or_else(|| name.to_string(),Id::to_string)}</span>}.into_any()
                ;
            }
            VarOrSym::Var(Variable::Ref { declaration, .. }) => {
                let uri = declaration;//.clone();
                return LocalCache::with_or_toast(
                    move |c| c.get_variable(crate::backend(), uri),
                    |v| match v {
                        either::Either::Left(v) => v.as_view(),
                        either::Either::Right(v) => v.as_view(),
                    },
                    || "Error".into_any(),
                );
            }
            VarOrSym::Sym(s) => s//.clone(),
        };
        let name = s.name().last().to_string();
        let uri_string = s.to_string();
        let uri = s.clone();
        let paras = LocalCache::resource(move |b| async move {
            Ok(b.get_paragraphs(crate::backend(), s, false).await)
        });
        let selected = RwSignal::new(None);
        let selector = super::formals::paras_selector(paras.read_only(), selected);
        view! {
            // paras
            <div style="display:flex;flex-direction:row;">
                <div style="font-weight:bold;" title=uri_string>{name}</div>
                <div style="margin-left:auto;">{selector}</div>
            </div>
            <div style="margin:5px;"><Divider/></div>

            // defi
            {super::formals::para_window(selected)}
            <div style="margin:5px;"><Divider/></div>

            // notations
            {super::super::notations::notation_selector(uri.clone().into())}

            // formals
            {move || if allow_formals.get() {
                Some(super::formals::formals(uri.clone(),top_term))
            } else {None} }
        }
        .into_any()
    }
}
