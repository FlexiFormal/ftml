#![allow(clippy::must_use_candidate)]

use crate::{
    SendBackend,
    components::content::FtmlViewable,
    config::{FtmlConfig, HighlightStyle},
    utils::{LocalCacheExt, ReactiveStore, collapsible::lazy_collapsible, wait_and_then},
};
use ftml_backend::{BackendCheckResult, BackendError, FtmlBackend, GlobalBackend};
use ftml_dom::{
    ClonableView, DocumentState, FtmlViews, TermTrackedViews,
    notations::TermExt,
    terms::ReactiveApplication,
    utils::{
        ContextChain, ModuleContext,
        css::{CssExt, inject_css},
        local_cache::{GlobalLocal, LocalCache},
    },
};
use ftml_ontology::{
    narrative::elements::ParagraphOrProblemKind,
    terms::{ArgumentMode, ComponentVar, Term, VarOrSym, Variable},
};
use ftml_uris::{
    DocumentElementUri, Id, IsNarrativeUri, LeafUri, NarrativeUri, SymbolUri, Uri, UriWithArchive,
};
use leptos::{
    prelude::*,
    tachys::{reactive_graph::OwnedView, renderer::dom::Element},
};
use std::hint::unreachable_unchecked;
use wasm_bindgen::JsCast;

#[derive(Copy, Clone)]
struct InTerm {
    hovered: RwSignal<bool>,
}

#[allow(clippy::needless_pass_by_value)]
pub fn symbol_reference<B: SendBackend>(uri: SymbolUri, children: ClonableView) -> AnyView {
    tracing::trace!("symbol reference {uri}");
    provide_context(InTerm {
        hovered: RwSignal::new(false),
    });
    children.into_view::<crate::Views<B>>()
}

pub fn oms<B: SendBackend>(uri: SymbolUri, _in_term: bool, children: ClonableView) -> AnyView {
    tracing::trace!("OMS({uri})");
    provide_context(InTerm {
        hovered: RwSignal::new(false),
    });
    if FtmlConfig::allow_notation_changes() {
        let head: LeafUri = uri.into(); //.clone().into();
        super::notations::has_notation::<B>(
            /*Term::Symbol {
                uri,
                presentation: None,
            },*/
            head, children, None,
        )
    } else {
        children.into_view::<crate::Views<B>>()
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn variable_reference<B: SendBackend>(var: Variable, children: ClonableView) -> AnyView {
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
    children.into_view::<crate::Views<B>>()
}

pub fn omv<B: SendBackend>(var: Variable, _in_term: bool, children: ClonableView) -> AnyView {
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
            Variable::Name { .. } => children.into_view::<crate::Views<B>>(),
            Variable::Ref {
                declaration,
                is_sequence,
            } => super::notations::has_notation::<B>(
                /*Term::Var {
                    variable: Variable::Ref {
                        declaration: declaration.clone(),
                        is_sequence,
                    },
                    presentation: None,
                },*/
                declaration.into(),
                children,
                None,
            ),
        }
    } else {
        children.into_view::<crate::Views<B>>()
    }
}

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

pub fn oma<V: FtmlViews, B: SendBackend>(
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
        return children.into_view::<crate::Views<B>>();
    }
    if !children.is_math() {
        tracing::trace!("Not in math");
        return children.into_view::<crate::Views<B>>();
    }

    let uri: Option<LeafUri> = head.with_untracked(|h| match h.head() {
        VarOrSym::Sym(s) => Some(s.clone().into()),
        VarOrSym::Var(Variable::Ref { declaration, .. }) => Some(declaration.clone().into()),
        VarOrSym::Var(_) => None,
    });

    #[allow(clippy::option_if_let_else)]
    if let Some(uri) = uri {
        let r = super::notations::has_notation::<B>(
            //t.clone(),
            uri,
            children,
            Some(head),
        );
        with_subterm::<V, B>(head.with_untracked(ReactiveApplication::term), r)
    } else {
        children.into_view::<crate::Views<B>>()
    }
}

pub const fn comp_class(is_hovered: bool, is_var: bool, style: HighlightStyle) -> &'static str {
    use HighlightStyle as HL;
    match (is_hovered, is_var, style) {
        (false, true, _) => "ftml-var-comp",
        (true, true, _) => "ftml-var-comp ftml-comp-hover",
        (false, false, HL::Colored | HL::None) => "ftml-comp",
        (false, false, HL::Subtle) => "ftml-comp-subtle",
        (true, false, HL::Subtle) => "ftml-comp-subtle ftml-comp-hover",
        (true, false, HL::Colored | HL::None) => "ftml-comp ftml-comp-hover",
        (false, _, HL::Off) => "",
        (true, _, HL::Off) => "ftml-comp-hover",
    }
}

pub fn defcomp<Be: SendBackend>(uri: Option<SymbolUri>, children: ClonableView) -> AnyView {
    use HighlightStyle as HL;
    tracing::trace!("doing defcomp");
    if !FtmlConfig::allow_hovers() {
        tracing::trace!("hovers disabled");
        return children.into_view::<crate::Views<Be>>();
    }
    inject_css("ftml-comp", include_str!("comp.css"));
    let is_hovered = use_context::<InTerm>().map(|h| h.hovered);
    let style = FtmlConfig::highlight_style();
    let class = Memo::new(
        move |_| match (is_hovered.is_some_and(|h| h.get()), style.get()) {
            (false, HL::Colored | HL::None) => "ftml-def-comp",
            (false, HL::Subtle) => "ftml-def-comp-subtle",
            (true, HL::Colored | HL::None) => "ftml-def-comp ftml-comp-hover",
            (true, HL::Subtle) => "ftml-def-comp-subtle ftml-comp-hover",
            (false, HL::Off) => "",
            (true, HL::Off) => "ftml-comp-hover",
        },
    );
    if let Some(uri) = uri {
        let on_click = ReactiveStore::on_click::<Be>(&uri.into());
        let allow_formals = FtmlConfig::allow_formal_info();
        let on_click = move |_| {
            on_click.click(allow_formals, None);
        };
        children
            .into_view::<crate::Views<Be>>()
            .attr("class", move || class)
            .add_any_attr(leptos::ev::on(leptos::ev::click, Box::new(on_click)))
            .into_any()
    } else {
        children
            .into_view::<crate::Views<Be>>()
            .attr("class", move || class)
            .into_any()
    }
}

pub fn comp<B: SendBackend>(children: ClonableView) -> AnyView {
    tracing::trace!("doing comp");
    if !FtmlConfig::allow_hovers() {
        tracing::trace!("hovers disabled");
        return children.into_view::<crate::Views<B>>();
    }
    let Some(head) = DocumentState::current_term_head() else {
        tracing::warn!("no current head");
        return children.into_view::<crate::Views<B>>();
    };

    let is_var = matches!(&head, VarOrSym::Var(_));
    let Some(is_hovered) = use_context::<InTerm>().map(|h| h.hovered) else {
        tracing::warn!("InTerm is missing!");
        return children.into_view::<crate::Views<B>>();
    };
    if is_var {
        let arg = DocumentState::arguments().next();
        tracing::trace!("variable comp: {head}@{arg:?}");
        if arg.is_some_and(|a| [ArgumentMode::Sequence, ArgumentMode::Simple].contains(&a.mode())) {
            let VarOrSym::Var(var) = head.clone() else {
                // SAFETY: is_var==true
                unsafe { unreachable_unchecked() }
            };
            let actual_signal = RwSignal::new(None);
            Effect::new(move || {
                if let Some(new_hovered) = ContextChain::<InBinder>::iter().find_map(|ctx| {
                    if ctx.vars.with(|v| v.contains(&var)) {
                        Some(ctx.hovered)
                    } else {
                        None
                    }
                }) {
                    tracing::trace!("{var}@{arg:?} inherits signal");
                    actual_signal.set(Some(new_hovered));
                }
            });
            let has_changed = RwSignal::new(false);
            Effect::new(move || {
                is_hovered.track();
                if let Some(sig) = actual_signal.get() {
                    if has_changed.get_untracked() {
                        has_changed.update_untracked(|b| *b = false);
                    } else {
                        has_changed.update_untracked(|b| *b = true);
                        is_hovered.set(sig.get());
                    }
                }
            });
            Effect::new(move || {
                is_hovered.track();
                if let Some(sig) = actual_signal.get() {
                    if has_changed.get_untracked() {
                        has_changed.update_untracked(|b| *b = false);
                    } else {
                        has_changed.update_untracked(|b| *b = true);
                        sig.set(is_hovered.get());
                    }
                }
            });
        }
    }

    comp_like::<B, _>(head, Some(is_hovered), true, move || {
        children.into_view::<crate::Views<B>>()
    })
}

pub fn comp_like<B: SendBackend, V: IntoView + 'static>(
    head: VarOrSym,
    is_hovered: Option<RwSignal<bool>>,
    top_term: bool,
    children: impl FnOnce() -> V + Send + 'static,
) -> AnyView {
    use thaw::{Popover, PopoverSize, PopoverTrigger};

    inject_css("ftml-comp", include_str!("comp.css"));
    let is_hovered = is_hovered.unwrap_or_else(|| RwSignal::new(false));
    let on_click = ReactiveStore::on_click::<B>(&head);
    let allow_formals = FtmlConfig::allow_formal_info();
    let top_class = Memo::new(move |_| {
        if is_hovered.get() {
            tracing::trace!("Hovering");
            "ftml-symbol-hover ftml-symbol-hover-hovered".to_string()
        } else {
            "ftml-symbol-hover ftml-symbol-hover-hidden".to_string()
        }
    });
    let style = FtmlConfig::highlight_style();
    let is_var = matches!(head, VarOrSym::Var(_));
    let class = Memo::new(move |_| comp_class(is_hovered.get(), is_var, style.get()).to_string());
    let top_term = if top_term && allow_formals {
        crate::Views::<B>::current_top_term()
    } else {
        None
    };
    let on_click = move |_| {
        on_click.click(allow_formals, top_term.clone());
    };
    view! {
        <Popover
            class=top_class
            size=PopoverSize::Small
            on_open=move || is_hovered.set(true)
            on_close=move || is_hovered.set(false)
            //on_click_signal=ocp
        >
            <PopoverTrigger slot>{
                children().attr("class",move || class)
                .add_any_attr(leptos::ev::on(
                    leptos::ev::click,
                    Box::new(on_click)
                ))
            }</PopoverTrigger>
            {term_popover::<B>(head)}
        </Popover>
    }
    .into_any()
}

//#[component]
pub fn term_popover<Be: SendBackend>(head: VarOrSym) -> AnyView {
    match head {
        VarOrSym::Var(Variable::Name { name, notated }) => unresolved_var_popover(name, notated),
        VarOrSym::Var(Variable::Ref {
            declaration,
            is_sequence,
        }) => resolved_var_popover::<Be>(declaration, is_sequence.unwrap_or_default()),
        VarOrSym::Sym(uri) => symbol_popover::<Be>(uri),
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

pub fn resolved_var_popover<B: SendBackend>(uri: DocumentElementUri, is_sequence: bool) -> AnyView {
    use thaw::Text;
    let title = if is_sequence {
        "Variable Sequence "
    } else {
        "Variable "
    };
    let declaration = uri.clone();
    let tm = ftml_dom::utils::math(move || {
        ReactiveStore::render_term::<B>(Term::Var {
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
        {LocalCache::with::<B,_,_>(|b| b.get_variable(uri),|v| {
            let v = match &v {
                either::Either::Left(v) => v,
                either::Either::Right(v) => &**v
            };
            let tp = v.data.tp.parsed();
            let df = v.data.df.parsed();
            view! {
                {df.map(|df| {
                    let v = view!{"defined as "
                        {
                            let t = df.clone().into_view::<crate::Views<B>,B>(false);
                            ftml_dom::utils::math(move || t)
                        }};
                    view!{<div><Text>{v}</Text></div>}
                })}
                {tp.map(|tp| {
                    let v = view!{"of type "
                        {
                            let t = tp.clone().into_view::<crate::Views<B>,B>(false);
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

pub fn symbol_popover<B: SendBackend>(uri: SymbolUri) -> AnyView {
    inject_css("ftml-symbol-popup", include_str!("popup.css"));
    let context = DocumentState::context_uri();
    LocalCache::with::<B, _, _>(
        |b| b.get_definition(uri, Some(context)),
        |(html, css, _)| {
            for c in css {
                c.inject();
            }
            view! {
              <div class="ftml-symbol-popup">
                {
                    DocumentState::no_document(
                        || crate::Views::<B>::render_ftml(html.into_string(),None)
                    )
                }
              </div>
            }
            .into_any()
        },
    )
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

pub(crate) fn do_onclick<Be: SendBackend>(
    vos: &VarOrSym,
    top_term: ReadSignal<Option<DocumentElementUri>>,
    allow_formals: ReadSignal<bool>,
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
            let uri = declaration.clone();
            return LocalCache::with_or_toast::<Be, _, _>(
                move |c| c.get_variable(uri),
                |v| match v {
                    either::Either::Left(v) => v.as_view::<Be>(),
                    either::Either::Right(v) => v.as_view::<Be>(),
                },
                || "Error".into_any(),
            );
        }
        VarOrSym::Sym(s) => s.clone(),
    };
    let name = s.name().last().to_string();
    let uri_string = s.to_string();
    let uri = s.clone();
    let paras =
        LocalCache::resource::<Be, _, _>(
            move |b| async move { Ok(b.get_paragraphs(s, false).await) },
        );
    let selected = RwSignal::new(None);
    let selector = paras_selector::<Be>(paras.read_only(), selected);
    view! {
        // paras
        <div style="display:flex;flex-direction:row;">
            <div style="font-weight:bold;" title=uri_string>{name}</div>
            <div style="margin-left:auto;">{selector}</div>
        </div>
        <div style="margin:5px;"><Divider/></div>

        // defi
        {para_window::<Be>(selected)}
        <div style="margin:5px;"><Divider/></div>

        // notations
        {super::notations::notation_selector::<Be>(uri.clone().into())}

        // formals
        {move || if allow_formals.get() {
            Some(formals::<Be>(uri.clone(),top_term))
        } else {None} }
    }
    .into_any()
}

fn formals<Be: SendBackend>(
    symbol: SymbolUri,
    uri: ReadSignal<Option<DocumentElementUri>>,
) -> AnyView {
    use thaw::Divider;
    view! {
        <div style="margin:5px;"><Divider/></div>
        {lazy_collapsible(Some(|| "Formal Details"), move ||view!{
            <div style="width:100%"><div style="width:fit-content;margin-left:auto;">
                "(in module "{symbol.module.as_view::<Be>()}")"
            </div></div>
            {
                let sym = symbol.clone();
                let bol = symbol.clone();
                LocalCache::with_or_toast::<Be,_,_>(
                    move |r| r.get_symbol(sym),
                    |s| match s {
                        ::either::Left(s) => super::content::symbols::symbol_view::<Be>(&s,false),
                        ::either::Right(s) => super::content::symbols::symbol_view::<Be>(&s,false)
                    },
                    move || view!({format!("error getting uri {bol}")}<br/>).into_any()
            )}
            {move || { uri.with(|u| u.clone().map(|u|  {
               let uri = u.clone();
               view!("In term: "{LocalCache::with_or_toast::<Be,_,_>(
                   |r| r.get_document_term(u),
                   |t|  {
                       tracing::warn!("Rendering term {t:?}");
                       ftml_dom::utils::math(|| ReactiveStore::render_term::<Be>(
                           match t {
                               ::either::Left(t) => t.parsed().clone(),
                               ::either::Right(t) => t.parsed().clone(),
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

type Paras<Be> = Result<
    GlobalLocal<
        Vec<(DocumentElementUri, ParagraphOrProblemKind)>,
        BackendError<<Be as GlobalBackend>::Error>,
    >,
    BackendError<<Be as GlobalBackend>::Error>,
>;

fn paras_selector<Be: SendBackend>(
    paras: ReadSignal<Option<Paras<Be>>>,
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

fn para_window<Be: SendBackend>(selected: RwSignal<Option<String>>) -> impl IntoView {
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
                    Left(LocalCache::with_or_toast::<Be, _, _>(
                        |c| c.get_fragment(Uri::DocumentElement(u), None),
                        |(html, css, stripped)| {
                            for c in css {
                                c.inject();
                            }
                            crate::Views::<Be>::render_fragment::<Be>(
                                Some(uri.into()),
                                crate::SidebarPosition::None,
                                stripped,
                                || {
                                    crate::Views::<Be>::render_ftml(html.into_string(), None)
                                        .into_any()
                                },
                            )
                        },
                        || view!(<span style="color:red;">"error"</span>).into_any(),
                    ))
                },
            )
    }
}

fn with_subterm<V: FtmlViews, Be: SendBackend>(t: ReadSignal<Option<Term>>, v: AnyView) -> AnyView {
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
                    let t = move || t.get().map(|t| t.into_view::<V, Be>(false));
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
    subterm_dialog::<V, Be>(v, t, owner, selected, dialog_open)
}

#[allow(clippy::too_many_lines)]
fn subterm_dialog<V: FtmlViews, Be: SendBackend>(
    v: AnyView,
    t: ReadSignal<Option<Term>>,
    owner: Owner,
    selected: RwSignal<bool>,
    dialog_open: RwSignal<bool>,
) -> AnyView {
    use thaw::{Dialog, DialogBody, DialogSurface, DialogTitle, Tooltip};

    let nv = v.directive(move |e| selection_listener(e, &owner, selected), ());

    let nt = move || t.get().map(|t| t.into_view::<V, Be>(false));
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
                LocalCache::resource::<Be, _, _>(|r| r.get_document_term(uri)),
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
                            ::either::Left(t) => t.parsed().clone(),
                            ::either::Right(t) => t.parsed().clone(),
                        };
                        view!{
                            <div>"In full term: "{ftml_dom::utils::math(move || ReactiveStore::render_term::<Be>(t))}</div>
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
                                            ::either::Left(t) => t.parsed(),
                                            ::either::Right(t) => t.parsed(),
                                        };
                                        if let Some(path) = t.path_of_subterm(sub) {
                                            //leptos::logging::log!("Path: {path:?}");
                                            let fut = Be::get().check_term(&context, t, &path);
                                            leptos::task::spawn_local(async move {
                                                let r = fut.await;
                                                //leptos::logging::log!("Setting signal");
                                                sig.set(Some(r.map_err(|e| e.to_string())));
                                            });
                                        } else {
                                            /*leptos::logging::log!(
                                                "{:?} is not a subterm of {:?}",
                                                sub.debug_short(),
                                                t.debug_short()
                                            );*/
                                        }
                                    }
                                    Err(e) => sig.set(Some(Err(e.to_string()))),
                                }
                            } else {
                                //leptos::logging::log!("full_term is None");
                            }
                        }
                    });
                });
            });
            move || {
                sig.get().map_or_else(
                    || leptos::either::Either::Left(view! {<Spinner/>}),
                    |r| {
                        leptos::either::Either::Right(match r {
                            Ok(r) => leptos::either::Either::Left(check_result::<Be>(r)),
                            Err(e) => leptos::either::Either::Right(
                                view! {<div style="color:red">"Error: "{e}</div>},
                            ),
                        })
                    },
                )
            }
        });
        view! {
            <div style="display:flex;flex-direction:column;">
                <div><b>"Selected term: "</b>{term}</div>
                {rendered_term}
                {inferred}
            </div>
        }
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

fn check_result<Be: SendBackend>(
    BackendCheckResult {
        context,
        inferred_type,
        simplified,
    }: BackendCheckResult,
) -> impl IntoView {
    #[allow(clippy::option_if_let_else)]
    let tp = if let Some(tp) = inferred_type {
        let tp = ftml_dom::utils::math(move || ReactiveStore::render_term::<Be>(tp));
        leptos::either::Either::Left(view! {
            <div>"Inferred type: "{tp}</div>
        })
    } else {
        leptos::either::Either::Right(view! {<div>"(Type inferrence failed)"</div>})
    };
    let simplified = view! {
        <div>"Simplified: "
            {ftml_dom::utils::math(move || ReactiveStore::render_term::<Be>(simplified))}
        </div>
    };
    let ctx = if context.is_empty() {
        None
    } else {
        let mut iter = context.into_iter();
        // SAFETY: !context.is_empty()
        let first = cv::<Be>(unsafe { iter.next().unwrap_unchecked() });
        let rest = iter
            .map(|v| view! {<mo>", "</mo>{cv::<Be>(v)}})
            .collect_view();
        let inner = ftml_dom::utils::math(move || view! {<mrow>{first}{rest}</mrow>});
        Some(view! {<div>"...where " {inner}</div>})
    };
    view! {{simplified}{tp}{ctx}}
}

fn cv<Be: SendBackend>(v: ComponentVar) -> impl IntoView {
    let tp = v.tp.map(|t| {
        let t = ReactiveStore::render_term::<Be>(t);
        view! {<mo>":"</mo>{t}}
    });
    let df = v.df.map(|t| {
        let t = ReactiveStore::render_term::<Be>(t);
        view! {<mo>":="</mo>{t}}
    });
    if tp.is_none() && df.is_none() {
        None
    } else {
        let var = ReactiveStore::render_term::<Be>(Term::Var {
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
