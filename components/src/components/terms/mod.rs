#![allow(clippy::must_use_candidate)]

pub mod comp;
pub mod formals;
pub mod popover;
pub mod subterms;
pub mod symvars;

use crate::config::FtmlConfig;
use ftml_dom::{
    ClonableView, DocumentState, FtmlViews, terms::ReactiveApplication, utils::ContextChain,
};
use ftml_ontology::terms::{ArgumentMode, VarOrSym, Variable};
use ftml_uris::{DocumentElementUri, LeafUri, SymbolUri};
use leptos::prelude::*;
use leptos_posthoc::OriginalNode;
use send_wrapper::SendWrapper;

#[derive(Copy, Clone)]
struct InTerm {
    hovered: RwSignal<bool>,
}

#[derive(Default, Clone)]
struct FoldExpr(Option<SendWrapper<OriginalNode>>);

#[derive(Copy, Clone)]
struct InBinder {
    hovered: RwSignal<bool>,
    vars: RwSignal<Vec<Variable>>,
}
impl std::fmt::Debug for InBinder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InBinder")
            .field("hovered", &self.hovered.get_untracked())
            .field("vars", &self.vars.get_untracked())
            .finish()
    }
}

#[derive(Copy, Clone)]
pub struct OnClickData {
    is_clicked: WriteSignal<bool>,
    top_term: WriteSignal<Option<DocumentElementUri>>,
    allow_formals: WriteSignal<bool>,
}
impl OnClickData {
    pub fn click(&self, allow_formals: bool, top_term: Option<DocumentElementUri>) {
        self.allow_formals.set(allow_formals);
        self.top_term.set(top_term);
        self.is_clicked.set(true);
    }
    pub(crate) fn new() -> (
        Self,
        RwSignal<bool>,
        ReadSignal<Option<DocumentElementUri>>,
        ReadSignal<bool>,
    ) {
        let allow_formals = RwSignal::new(false);
        let top_term = RwSignal::new(None);
        let is_clicked = RwSignal::new(false);
        (
            Self {
                is_clicked: is_clicked.write_only(),
                top_term: top_term.write_only(),
                allow_formals: allow_formals.write_only(),
            },
            is_clicked,
            top_term.read_only(),
            allow_formals.read_only(),
        )
    }
}

pub fn fold_expr<V: IntoView>(
    show: bool,
    then: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    let show = RwSignal::new(show);
    let short = RwSignal::new(FoldExpr(None));
    provide_context(short);
    let fullstyle = Signal::derive(move || {
        if show.get() {
            None
        } else {
            Some("display:none;")
        }
    });
    let otherstyle = Signal::derive(move || {
        if show.get() {
            Some("display:none;")
        } else {
            None
        }
    });
    view! {
        <munder displaystyle="true">
            <munder displaystyle="true">
                <msup>
                    <mrow>
                        {then().attr("style", fullstyle)}
                        {move || if show.get() {None} else {Some(view!{
                            <mi>...</mi>
                        })}}
                    </mrow>
                    <mi style="color:blue;cursor:pointer;" on:click=move|_| {show.update(|b| *b = !*b)}>"🛈"</mi>
                </msup>
                <mo stretchy="true" style=otherstyle>"⏟"</mo>
            </munder>
            {move || short.get().0.map(|n| n.take().attr("style",otherstyle))}
        </munder>
    }
}

pub fn fold_expr_short(then: OriginalNode) -> impl IntoView {
    let _ = crate::Views::cont(then.clone(), true);
    if let Some(sig) = use_context::<RwSignal<FoldExpr>>() {
        sig.set(FoldExpr(Some(SendWrapper::new(then))));
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn symbol_reference(uri: SymbolUri, children: ClonableView) -> AnyView {
    tracing::trace!("symbol reference {uri}");
    provide_context(InTerm {
        hovered: RwSignal::new(false),
    });
    children.into_view::<crate::Views>()
}

pub fn oms(uri: SymbolUri, _in_term: bool, children: ClonableView) -> AnyView {
    tracing::trace!("OMS({uri})");
    provide_context(InTerm {
        hovered: RwSignal::new(false),
    });
    if FtmlConfig::allow_notation_changes() {
        let head: LeafUri = uri.into(); //.clone().into();
        super::notations::has_notation(
            /*Term::Symbol {
                uri,
                presentation: None,
            },*/
            head, children, None,
        )
    } else {
        children.into_view::<crate::Views>()
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn variable_reference(var: Variable, children: ClonableView) -> AnyView {
    tracing::trace!("variable reference {var}");
    provide_context(InTerm {
        hovered: RwSignal::new(false),
    });
    if let Some(pos) = DocumentState::arguments().next()
        && [
            ArgumentMode::BoundVariable,
            ArgumentMode::BoundVariableSequence,
        ]
        .contains(&pos.mode())
    {
        if let Some(binder) = ContextChain::<InBinder>::get() {
            tracing::trace!("Updating context {binder:?}");
            binder.vars.update(|v| v.push(var.clone()));
        } else {
            tracing::trace!("No binder found");
        }
    }
    children.into_view::<crate::Views>()
}

pub fn omv(var: Variable, _in_term: bool, children: ClonableView) -> AnyView {
    tracing::trace!("OMV({var})");
    if let Some(pos) = DocumentState::arguments().next()
        && [
            ArgumentMode::BoundVariable,
            ArgumentMode::BoundVariableSequence,
        ]
        .contains(&pos.mode())
    {
        if let Some(binder) = ContextChain::<InBinder>::get() {
            tracing::trace!("Updating context {binder:?}");
            binder.vars.update(|v| v.push(var.clone()));
        } else {
            tracing::trace!("No binder found");
        }
    }
    provide_context(InTerm {
        hovered: RwSignal::new(false),
    });
    if FtmlConfig::allow_notation_changes() {
        match var {
            Variable::Name { .. } => children.into_view::<crate::Views>(),
            Variable::Ref {
                declaration,
                is_sequence,
            } => super::notations::has_notation(declaration.into(), children, None),
        }
    } else {
        children.into_view::<crate::Views>()
    }
}

pub fn oma(
    is_binder: bool,
    head: ReadSignal<ReactiveApplication>,
    children: ClonableView,
) -> AnyView {
    tracing::trace!("OMA|OMBIND({head:?},...)");
    let hovered = RwSignal::new(false);
    provide_context(InTerm { hovered });
    if is_binder {
        ContextChain::provide(InBinder {
            hovered,
            vars: RwSignal::new(Vec::new()),
        });
    }
    if !FtmlConfig::allow_notation_changes() {
        tracing::trace!("No notation changes");
        return children.into_view::<crate::Views>();
    }
    if !children.is_math() {
        tracing::trace!("Not in math");
        return children.into_view::<crate::Views>();
    }

    let uri: Option<LeafUri> = head.with_untracked(|h| match h.head() {
        VarOrSym::Sym(s) => Some(s.clone().into()),
        VarOrSym::Var(Variable::Ref { declaration, .. }) => Some(declaration.clone().into()),
        VarOrSym::Var(_) => None,
    });

    let allow_subterms = FtmlConfig::allow_subterms();

    #[allow(clippy::option_if_let_else)]
    if let Some(uri) = uri {
        let r = super::notations::has_notation(
            //t.clone(),
            uri,
            children,
            Some(head),
        );
        if allow_subterms {
            subterms::with_subterm(head.with_untracked(ReactiveApplication::term), r)
        } else {
            r
        }
    } else {
        children.into_view::<crate::Views>()
    }
}
