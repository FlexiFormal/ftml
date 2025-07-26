#![allow(clippy::must_use_candidate)]

use ftml_backend::FtmlBackend;
use ftml_dom::{
    DocumentState, FtmlViews, VarOrSym,
    utils::{
        css::{CssExt, inject_css},
        local_cache::LocalCache,
    },
};
use ftml_ontology::terms::Variable;
use ftml_uris::{DocumentElementUri, LeafUri, SymbolUri, UriName};
use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::{
    SendBackend,
    config::{FtmlConfigState, HighlightStyle},
    utils::LocalCacheExt,
};

#[derive(Clone)]
pub(crate) struct ReactiveTerm {
    head: LeafUri,
    arguments: Vec<ReadSignal<Option<TermFn>>>,
    done: ReadSignal<bool>,
}
impl ReactiveTerm {
    pub fn with_new<V: IntoView>(
        head: LeafUri,
        f: impl FnOnce() -> V + Clone + 'static,
    ) -> impl IntoView {
        let done = RwSignal::new(false);
        let tm = Self {
            head,
            arguments: Vec::new(),
            done: done.read_only(),
        };
        provide_context(tm);

        let arg = with_context::<Argument, _>(|arg| arg.0);
        if let Some(arg) = arg {
            if arg.update_untracked(|a| a.is_none()) {
                let f = f.clone();
                arg.set(Some(TermFn(SendWrapper::new(Box::new(move || {
                    f().into_any()
                })))));
            }
        }
        let r = f();
        done.set(true);
        r
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Argument(WriteSignal<Option<TermFn>>);

pub(crate) struct TermFn(send_wrapper::SendWrapper<Box<dyn FnOnce() -> AnyView>>);
impl std::fmt::Debug for TermFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[TermFn]")
    }
}
pub(crate) trait TermFnTrait {
    fn call(&self) -> AnyView;
}
impl<F, V: IntoView> TermFnTrait for F
where
    F: FnOnce() -> V + Clone + 'static,
{
    fn call(&self) -> AnyView {
        let s = self.clone();
        s().into_any()
    }
}

#[derive(Copy, Clone)]
struct InTerm {
    hovered: RwSignal<bool>,
}

pub fn symbol_reference<B: SendBackend, V: IntoView>(
    _uri: SymbolUri,
    children: impl FnOnce() -> V,
) -> impl IntoView {
    provide_context(InTerm {
        hovered: RwSignal::new(false),
    });
    children()
}

pub fn oms<B: SendBackend, V: IntoView + 'static>(
    uri: SymbolUri,
    in_term: bool,
    children: impl FnOnce() -> V + Clone + Send + 'static,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    provide_context(InTerm {
        hovered: RwSignal::new(false),
    });
    if FtmlConfigState::allow_notation_changes() {
        let head: LeafUri = uri.into();
        Left(ReactiveTerm::with_new(head.clone(), move || {
            super::notations::has_notation::<B, _, _>(head, children)
        }))
    } else {
        Right(children())
    }
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

pub fn comp<B: SendBackend, V: IntoView + 'static>(children: impl FnOnce() -> V) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    use thaw::{Popover, PopoverSize, PopoverTrigger};
    if !FtmlConfigState::allow_hovers() {
        return Left(children());
    }
    let Some(head) = DocumentState::current_term_head() else {
        return Left(children());
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
    let children = children();
    Right(view! {
        <Popover
            class=top_class
            size=PopoverSize::Small
            on_open=move || is_hovered.set(true)
            on_close=move || is_hovered.set(false)
            //on_click_signal=ocp
        >
            <PopoverTrigger slot>{
            children.add_any_attr(leptos::tachys::html::class::class(move || class))
            }</PopoverTrigger>
            {symbol_popover::<B>(head)}
        </Popover>
    })
}

//#[component]
pub fn symbol_popover<BE: SendBackend>(head: VarOrSym) -> impl IntoView {
    use leptos::either::EitherOf3::{A, B, C};
    match head {
        VarOrSym::V(Variable::Name(n)) => A(unresolved_var(n)),
        VarOrSym::V(Variable::Ref {
            declaration,
            is_sequence,
        }) => B(resolved_var(declaration, is_sequence.unwrap_or_default())),
        VarOrSym::S(uri) => C(symbol::<BE>(uri)),
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn unresolved_var(name: UriName) -> impl IntoView {
    view! {
        <div>
            "Variable: " {name.to_string()}
        </div>
    }
}

pub fn resolved_var(uri: DocumentElementUri, is_sequence: bool) -> impl IntoView {
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

pub fn symbol<B: SendBackend>(uri: SymbolUri) -> impl IntoView {
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
                {crate::Views::<B>::render_ftml(html)}
              </div>
            }
        },
    )
}
