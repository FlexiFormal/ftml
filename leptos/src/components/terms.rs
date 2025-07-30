#![allow(clippy::must_use_candidate)]

use crate::{
    SendBackend,
    config::{FtmlConfigState, HighlightStyle},
    utils::{LocalCacheExt, ReactiveStore},
};
use ftml_core::extraction::VarOrSym;
use ftml_dom::{
    ClonableView, DocumentState, FtmlViews,
    terms::{ClosedApp, ReactiveApplication},
    utils::{
        css::{CssExt, inject_css},
        local_cache::LocalCache,
    },
};
use ftml_ontology::terms::Variable;
use ftml_uris::{DocumentElementUri, Id, LeafUri, SymbolUri};
use leptos::prelude::*;

#[derive(Copy, Clone)]
struct InTerm {
    hovered: RwSignal<bool>,
}

#[allow(clippy::needless_pass_by_value)]
pub fn symbol_reference<B: SendBackend>(uri: SymbolUri, children: ClonableView) -> impl IntoView {
    tracing::trace!("symbol reference {uri}");
    provide_context(InTerm {
        hovered: RwSignal::new(false),
    });
    children.into_view::<crate::Views<B>>()
}

pub fn oms<B: SendBackend>(
    uri: SymbolUri,
    _in_term: bool,
    children: ClonableView,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    tracing::trace!("OMS({uri})");
    provide_context(InTerm {
        hovered: RwSignal::new(false),
    });
    if FtmlConfigState::allow_notation_changes() {
        let head: LeafUri = uri.into();
        Left(super::notations::has_notation::<B>(head, children, None))
    } else {
        Right(children.into_view::<crate::Views<B>>())
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn variable_reference<B: SendBackend>(var: Variable, children: ClonableView) -> impl IntoView {
    tracing::trace!("variable reference {var}");
    provide_context(InTerm {
        hovered: RwSignal::new(false),
    });
    children.into_view::<crate::Views<B>>()
}

pub fn omv<B: SendBackend>(var: Variable, _in_term: bool, children: ClonableView) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    tracing::trace!("OMV({var})");
    provide_context(InTerm {
        hovered: RwSignal::new(false),
    });
    if FtmlConfigState::allow_notation_changes() {
        match var {
            Variable::Name { .. } => Right(children.into_view::<crate::Views<B>>()),
            Variable::Ref { declaration, .. } => Left(super::notations::has_notation::<B>(
                declaration.into(),
                children,
                None,
            )),
        }
    } else {
        Right(children.into_view::<crate::Views<B>>())
    }
}

pub fn oma<B: SendBackend>(
    head: ReadSignal<ReactiveApplication>,
    children: ClonableView,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    tracing::trace!("OMA|OMBIND({head:?},...)");
    provide_context(InTerm {
        hovered: RwSignal::new(false),
    });
    if !FtmlConfigState::allow_notation_changes() {
        tracing::trace!("No notation changes");
        return Right(children.into_view::<crate::Views<B>>());
    }

    let uri: Option<LeafUri> = head.with_untracked(|h| match h.head() {
        VarOrSym::S(s) => Some(s.clone().into()),
        VarOrSym::V(Variable::Ref { declaration, .. }) => Some(declaration.clone().into()),
        VarOrSym::V(_) => None,
    });
    if !children.is_math() {
        tracing::trace!("Not in math");
        return Right(children.into_view::<crate::Views<B>>());
    }
    let ret = move |children| {
        if let Some(uri) = &uri {
            Left(super::notations::has_notation::<B>(
                uri.clone(),
                children,
                Some(head),
            ))
        } else {
            Right(children.into_view::<crate::Views<B>>())
        }
    };
    let memo = Memo::new(move |_| {
        head.with(|a| match a {
            ReactiveApplication::Open(_) => "No term yet".to_string(),
            ReactiveApplication::Closed(ClosedApp { term, .. }) => {
                tracing::debug!("New term arrived: {:?}", term.debug_short());
                format!("{:?}", term.debug_short())
            }
        })
    });
    Left(ret(children).attr("title", memo))
}

const fn comp_class(
    is_defi: bool,
    is_hovered: bool,
    is_var: bool,
    style: HighlightStyle,
) -> &'static str {
    use HighlightStyle as HL;
    match (is_defi, is_hovered, is_var, style) {
        (_, false, true, _) => "ftml-var-comp",
        (_, true, true, _) => "ftml-var-comp ftml-comp-hover",
        (true, false, _, HL::Colored | HL::None) => "ftml-def-comp",
        (true, false, _, HL::Subtle) => "ftml-def-comp-subtle",
        (true, true, _, HL::Colored | HL::None) => "ftml-def-comp ftml-comp-hover",
        (true, true, _, HL::Subtle) => "ftml-def-comp-subtle ftml-comp-hover",
        (_, false, false, HL::Colored | HL::None) => "ftml-comp",
        (_, false, false, HL::Subtle) => "ftml-comp-subtle",
        (_, true, false, HL::Subtle) => "ftml-comp-subtle ftml-comp-hover",
        (_, true, false, HL::Colored | HL::None) => "ftml-comp ftml-comp-hover",
        (_, false, _, HL::Off) => "",
        (_, true, _, HL::Off) => "ftml-comp-hover",
    }
}

pub fn comp<B: SendBackend>(children: ClonableView) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    use thaw::{Popover, PopoverSize, PopoverTrigger};
    tracing::trace!("doing comp");
    if !FtmlConfigState::allow_hovers() {
        tracing::trace!("hovers disabled");
        return Left(children.into_view::<crate::Views<B>>());
    }
    let Some(head) = DocumentState::current_term_head() else {
        tracing::trace!("no current head");
        return Left(children.into_view::<crate::Views<B>>());
    };

    inject_css("ftml-comp", include_str!("comp.css"));

    let is_var = matches!(&head, VarOrSym::V(_));
    let is_hovered = expect_context::<InTerm>().hovered;
    let style = FtmlConfigState::highlight_style();
    let class = Memo::new(move |_| comp_class(false, is_hovered.get(), is_var, style.get()));
    let top_class = Memo::new(move |_| {
        if is_hovered.get() {
            tracing::trace!("Hovering");
            "ftml-symbol-hover ftml-symbol-hover-hovered".to_string()
        } else {
            "ftml-symbol-hover ftml-symbol-hover-hidden".to_string()
        }
    });
    //let ocp = expect_context::<crate::config::FTMLConfig>().get_on_click(&s);
    //let none: Option<FragmentContinuation> = None;
    let children = children.into_view::<crate::Views<B>>();
    let on_click = ReactiveStore::get().write_value().on_click::<B>(&head);
    Right(view! {
        <Popover
            class=top_class
            size=PopoverSize::Small
            on_open=move || is_hovered.set(true)
            on_close=move || is_hovered.set(false)
            //on_click_signal=ocp
        >
            <PopoverTrigger slot>{
            children.attr("class",move || class)
            .add_any_attr(leptos::ev::on(
                leptos::ev::click,
                Box::new(move |_| on_click.set(true)),
            ))
            }</PopoverTrigger>
            {term_popover::<B>(head)}
        </Popover>
    })
}

//#[component]
pub fn term_popover<BE: SendBackend>(head: VarOrSym) -> impl IntoView {
    use leptos::either::EitherOf3::{A, B, C};
    match head {
        VarOrSym::V(Variable::Name { name, notated }) => A(unresolved_var_popover(name, notated)),
        VarOrSym::V(Variable::Ref {
            declaration,
            is_sequence,
        }) => B(resolved_var_popover(
            declaration,
            is_sequence.unwrap_or_default(),
        )),
        VarOrSym::S(uri) => C(symbol_popover::<BE>(uri)),
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn unresolved_var_popover(name: Id, _notated: Option<Id>) -> impl IntoView {
    view! {
        <div>
            "Variable: " {name.to_string()}
        </div>
    }
}

pub fn resolved_var_popover(uri: DocumentElementUri, is_sequence: bool) -> impl IntoView {
    use thaw::Tooltip;
    let title = if is_sequence {
        "Variable Sequenc: "
    } else {
        "Variable: "
    };
    view! {
        <Tooltip content = uri.to_string()>
            {title}{uri.name().to_string()}
        </Tooltip>
    }
}

pub fn symbol_popover<B: SendBackend>(uri: SymbolUri) -> impl IntoView {
    inject_css("ftml-symbol-popup", include_str!("popup.css"));
    let context = DocumentState::context_uri();
    LocalCache::with::<B, _, _, _>(
        |b| b.get_definition(uri, Some(context)),
        |(html, css)| {
            for c in css {
                c.inject();
            }
            view! {
              <div class="ftml-symbol-popup">
                {
                    DocumentState::no_document(
                        || crate::Views::<B>::render_ftml(html)
                    )
                }
              </div>
            }
        },
    )
}

pub(crate) fn do_onclick<Be: SendBackend>(vos: &VarOrSym) -> impl IntoView + use<Be> {
    use leptos::either::Either::{Left as A, Right as B};
    use leptos::prelude::*;
    use thaw::{Divider, Skeleton, SkeletonItem, Spinner};
    let s = match vos {
        VarOrSym::V(Variable::Name {
            notated: Some(n), ..
        }) => {
            return A(view! {<span>"Variable "{n.to_string()}</span>});
        }
        VarOrSym::V(Variable::Name { name, .. }) => {
            return A(view! {<span>"Variable "{name.to_string()}</span>});
        }
        VarOrSym::V(Variable::Ref { declaration, .. }) => {
            return A(view! {<span>"Variable "{declaration.name.last().to_string()}</span>});
        }
        VarOrSym::S(s) => s.clone(),
    };
    let name = s.name().last().to_string();
    let uri_string = s.to_string();
    let uri = s.clone();
    let paras = LocalCache::resource::<Be, _, _>(move |b| b.get_paragraphs(s, false));
    B(view! {
        // paras
        <div style="display:flex;flex-direction:row;">
            <div style="font-weight:bold;" title=uri_string>{name}</div>
            <div style="margin-left:auto;">{move || {
                match paras.get() {
                    Some(Ok(_)) => A("Here".to_string()),
                    Some(Err(e)) => A(format!("error: {e}")),
                    None => B(view!(<Spinner/>))
                }
            }}</div>
        </div>
        <div style="margin:5px;"><Divider/></div>

        // defi
        <Skeleton><SkeletonItem attr:style="height:150px;"/></Skeleton>
        <div style="margin:5px;"><Divider/></div>

        // notations
        {super::notations::notation_selector::<Be>(uri.into())}
    })
}
