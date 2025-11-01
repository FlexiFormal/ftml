#![allow(clippy::must_use_candidate)]

use crate::{
    SendBackend,
    components::content::FtmlViewable,
    config::{FtmlConfig, HighlightStyle},
    utils::{LocalCacheExt, ReactiveStore, collapsible::lazy_collapsible},
};
use ftml_backend::{BackendError, GlobalBackend};
use ftml_dom::{
    ClonableView, DocumentState, FtmlViews, TermTrackedViews,
    notations::TermExt,
    terms::ReactiveApplication,
    utils::{
        ContextChain,
        css::{CssExt, inject_css},
        local_cache::{GlobalLocal, LocalCache},
    },
};
use ftml_ontology::{
    narrative::elements::ParagraphOrProblemKind,
    terms::{ArgumentMode, Term, VarOrSym, Variable},
};
use ftml_uris::{DocumentElementUri, Id, IsNarrativeUri, LeafUri, SymbolUri, Uri, UriWithArchive};
use leptos::prelude::*;
use std::hint::unreachable_unchecked;

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
    if FtmlConfig::allow_notation_changes() {
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

pub fn omv<B: SendBackend>(var: Variable, _in_term: bool, children: ClonableView) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
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

pub fn oma<B: SendBackend>(
    is_binder: bool,
    head: ReadSignal<ReactiveApplication>,
    children: ClonableView,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
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
        return Right(children.into_view::<crate::Views<B>>());
    }
    if !children.is_math() {
        tracing::trace!("Not in math");
        return Right(children.into_view::<crate::Views<B>>());
    }

    let uri: Option<LeafUri> = head.with_untracked(|h| match h.head() {
        VarOrSym::Sym(s) => Some(s.clone().into()),
        VarOrSym::Var(Variable::Ref { declaration, .. }) => Some(declaration.clone().into()),
        VarOrSym::Var(_) => None,
    });
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
    Left(ret(children))
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

pub fn defcomp<Be: SendBackend>(uri: Option<SymbolUri>, children: ClonableView) -> impl IntoView {
    use HighlightStyle as HL;
    use leptos::either::EitherOf3::{A, B, C};
    tracing::trace!("doing defcomp");
    if !FtmlConfig::allow_hovers() {
        tracing::trace!("hovers disabled");
        return A(children.into_view::<crate::Views<Be>>());
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
        B(children
            .into_view::<crate::Views<Be>>()
            .attr("class", move || class)
            .add_any_attr(leptos::ev::on(leptos::ev::click, Box::new(on_click))))
    } else {
        C(children
            .into_view::<crate::Views<Be>>()
            .attr("class", move || class))
    }
}

pub fn comp<B: SendBackend>(children: ClonableView) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    tracing::trace!("doing comp");
    if !FtmlConfig::allow_hovers() {
        tracing::trace!("hovers disabled");
        return Left(children.into_view::<crate::Views<B>>());
    }
    let Some(head) = DocumentState::current_term_head() else {
        tracing::warn!("no current head");
        return Left(children.into_view::<crate::Views<B>>());
    };

    let is_var = matches!(&head, VarOrSym::Var(_));
    let Some(is_hovered) = use_context::<InTerm>().map(|h| h.hovered) else {
        tracing::warn!("InTerm is missing!");
        return Left(children.into_view::<crate::Views<B>>());
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

    Right(comp_like::<B, _>(head, Some(is_hovered), true, move || {
        children.into_view::<crate::Views<B>>()
    }))
}

pub fn comp_like<B: SendBackend, V: IntoView + 'static>(
    head: VarOrSym,
    is_hovered: Option<RwSignal<bool>>,
    top_term: bool,
    children: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
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
}

//#[component]
pub fn term_popover<Be: SendBackend>(head: VarOrSym) -> impl IntoView {
    use leptos::either::EitherOf3::{A, B, C};
    match head {
        VarOrSym::Var(Variable::Name { name, notated }) => A(unresolved_var_popover(name, notated)),
        VarOrSym::Var(Variable::Ref {
            declaration,
            is_sequence,
        }) => B(resolved_var_popover::<Be>(
            declaration,
            is_sequence.unwrap_or_default(),
        )),
        VarOrSym::Sym(uri) => C(symbol_popover::<Be>(uri)),
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn unresolved_var_popover(name: Id, notated: Option<Id>) -> impl IntoView {
    view! {
        <div>
            "Variable: " {notated.map_or_else(|| name.to_string(),|n| n.to_string())}
        </div>
    }
}

pub fn resolved_var_popover<B: SendBackend>(
    uri: DocumentElementUri,
    is_sequence: bool,
) -> impl IntoView {
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
        {LocalCache::with::<B,_,_,_>(|b| b.get_variable(uri),|v| {
            let v = match &v {
                either::Either::Left(v) => v,
                either::Either::Right(v) => &**v
            };
            let tp = v.data.tp.as_ref();
            let df = v.data.df.as_ref();
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
            }
        })}
    </div>}
}

pub fn symbol_popover<B: SendBackend>(uri: SymbolUri) -> impl IntoView {
    inject_css("ftml-symbol-popup", include_str!("popup.css"));
    let context = DocumentState::context_uri();
    LocalCache::with::<B, _, _, _>(
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
) -> impl IntoView + use<Be> {
    use leptos::either::{
        Either::{Left, Right},
        EitherOf3::{A, B, C},
    };
    use leptos::prelude::*;
    use thaw::Divider;
    let s = match vos {
        VarOrSym::Var(Variable::Name {
            notated: Some(n), ..
        }) => {
            return Left(A(view! {<span>"Variable "{n.to_string()}</span>}));
        }
        VarOrSym::Var(Variable::Name { name, notated }) => {
            return Left(B(
                view! {<span>"Variable "{notated.as_ref().map_or_else(|| name.to_string(),Id::to_string)}</span>},
            ));
        }
        VarOrSym::Var(Variable::Ref { declaration, .. }) => {
            let uri = declaration.clone();
            return Left(C(LocalCache::with_or_toast::<Be, _, _, _, _>(
                move |c| c.get_variable(uri),
                |v| match v {
                    either::Either::Left(v) => v.as_view::<Be>(),
                    either::Either::Right(v) => v.as_view::<Be>(),
                },
                || "Error",
            )));
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
    Right(view! {
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
    })
}

fn formals<Be: SendBackend>(
    symbol: SymbolUri,
    uri: ReadSignal<Option<DocumentElementUri>>,
) -> impl IntoView + use<Be> {
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
                LocalCache::with_or_toast::<Be,_,_,_,_>(
                    move |r| r.get_symbol(sym),
                    |s| match s {
                        ::either::Left(s) => super::content::symbols::symbol_view::<Be>(&s,false),
                        ::either::Right(s) => super::content::symbols::symbol_view::<Be>(&s,false)
                    },
                    move || view!({format!("error getting uri {bol}")}<br/>)
            )}
            {move || { uri.with(|u| u.clone().map(|u|  {
               let uri = u.clone();
               view!("In term: "{LocalCache::with_or_toast::<Be,_,_,_,_>(
                   |r| r.get_document_term(u),
                   |t|  {
                       tracing::warn!("Rendering term {t:?}");
                       ftml_dom::utils::math(|| ReactiveStore::render_term::<Be>(
                           match t {
                               ::either::Left(t) => t.term,
                               ::either::Right(t) => t.term.clone(),
                           }

                       ))
                   },
                   move || format!("error: {uri}")
               )})
            }
            ))}}
        })}
    }
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
                    Left(LocalCache::with_or_toast::<Be, _, _, _, _>(
                        |c| c.get_fragment(Uri::DocumentElement(u), None),
                        |(html, css, stripped)| {
                            for c in css {
                                c.inject();
                            }
                            crate::Views::<Be>::render_fragment::<Be, _>(
                                Some(uri.into()),
                                crate::SidebarPosition::None,
                                stripped,
                                || crate::Views::<Be>::render_ftml(html.into_string(), None),
                            )
                        },
                        || view!(<span style="color:red;">"error"</span>),
                    ))
                },
            )
    }
}
