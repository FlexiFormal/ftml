use crate::{
    components::content::FtmlViewable,
    config::FtmlConfig,
    utils::{
        Header, LocalCacheExt, ReactiveStore,
        block::{Block, HeaderLeft, HeaderRight},
    },
};
use ftml_dom::{FtmlViews, notations::NotationExt, utils::local_cache::LocalCache};
use ftml_ontology::{
    narrative::{
        documents::Document,
        elements::{DocumentElement, DocumentTerm, FlatIterable},
    },
    terms::{Term, VarOrSym, Variable},
};
use ftml_uris::{DocumentElementUri, DocumentUri, ModuleUri, SymbolUri};
use leptos::prelude::*;
use thaw::{Caption1Strong, Flex, Text};

impl super::FtmlViewable for Document {
    fn as_view<Be: ftml_dom::utils::local_cache::SendBackend>(
        &self,
    ) -> impl leptos::IntoView + use<Be> {
        let uses = self.elements.iter().flat().filter_map(|e| {
            if let DocumentElement::UseModule(u) = e {
                Some(u)
            } else {
                None
            }
        });
        let uses = super::uses::<Be, _>("Uses", uses);
        let children = self
            .elements
            .iter()
            .map(FtmlViewable::as_view::<Be>)
            .collect_view();
        view! {<Block show_separator=false>
          <HeaderLeft slot>{uses}</HeaderLeft>
          {children}
        </Block>}
    }
}

impl super::FtmlViewable for DocumentElement {
    fn as_view<Be: ftml_dom::utils::local_cache::SendBackend>(
        &self,
    ) -> impl IntoView + use<Be> + 'static {
        //use leptos::either::EitherOf10::{A, B, C, D, E, F, G, H, I, J};
        match self {
            Self::UseModule(_)
            | Self::ImportModule(_)
            | Self::SymbolReference { .. }
            | Self::VariableReference { .. }
            | Self::Definiendum { .. } => ().into_any(),
            Self::SkipSection(s) => s
                .iter()
                .map(FtmlViewable::as_view::<Be>)
                .collect_view()
                .into_any(),
            Self::SymbolDeclaration(s) => {
                let s = s.clone();
                LocalCache::with_or_toast::<Be, _, _, _, _>(
                    |e| e.get_symbol(s),
                    move |s| match s {
                        either::Either::Left(s) => s.as_view::<Be>(),
                        either::Either::Right(s) => s.as_view::<Be>(),
                    },
                    || "error",
                )
                .into_any()
            }
            Self::DocumentReference { target, .. } => view_inputref::<Be>(target).into_any(),
            Self::Module {
                module, children, ..
            } => view_module::<Be>(module, children).into_any(),
            Self::MathStructure {
                structure,
                children,
                ..
            } => view_structure::<Be>(structure, children).into_any(),
            Self::Extension {
                extension,
                target,
                children,
                ..
            } => view_extension::<Be>(extension, target, children).into_any(),
            Self::VariableDeclaration(v) => v.as_view::<Be>().into_any(),
            Self::Notation { symbol, uri, .. } => {
                view_notation::<Be>(uri.clone(), VarOrSym::Sym(symbol.clone())).into_any()
            }
            Self::VariableNotation { variable, uri, .. } => view_notation::<Be>(
                uri.clone(),
                VarOrSym::Var(Variable::Ref {
                    declaration: variable.clone(),
                    is_sequence: None,
                }),
            )
            .into_any(),
            Self::Paragraph(p) => p.as_view::<Be>().into_any(),
            Self::Section(s) => s.as_view::<Be>().into_any(),
            Self::Term(DocumentTerm { uri, term }) => view_term::<Be>(uri, term.clone()).into_any(),
            o => {
                let txt = format!("{o:?}");
                view!(<div><Text tag=thaw::TextTag::Code>"TODO: "{txt}</Text></div>).into_any()
            }
        }
    }
}

fn view_term<Be: ftml_dom::utils::local_cache::SendBackend>(
    uri: &DocumentElementUri,
    term: Term,
) -> impl IntoView + 'static {
    let name = view!(<span title=uri.to_string()>{uri.name().last().to_string()}</span>);
    let tm = ReactiveStore::render_term::<Be>(term);

    view! {//<Block>
        <Flex>
            <div style="min-width:150px;">
                <Caption1Strong>"Term "{name}</Caption1Strong>
            </div>
            <span><math>{tm}</math></span>
        </Flex>
        //</Block>
    }
}

fn view_notation<Be: ftml_dom::utils::local_cache::SendBackend>(
    uri: DocumentElementUri,
    head: VarOrSym,
) -> impl IntoView + 'static {
    use leptos::either::EitherOf3::{A, B, C};
    let name = view!(<span title=uri.to_string()>{uri.name().last().to_string()}</span>);
    let (target, leaf) = match &head {
        VarOrSym::Sym(s) => (A(s.as_view::<Be>()), Some(s.clone().into())),
        VarOrSym::Var(Variable::Ref { declaration, .. }) => {
            let name = declaration.name().last().to_string();
            (
                B(view!(<Text class="ftml-var-comp">{name}</Text>)),
                Some(declaration.clone().into()),
            )
        }
        VarOrSym::Var(Variable::Name { .. }) => (C("TODO"), None),
    };
    let not = FtmlConfig::disable_hovers(move || {
        LocalCache::with_or_toast::<Be, _, _, _, _>(
            |e| e.get_notation(leaf, uri),
            move |n| n.as_view_safe::<crate::Views<Be>>(&head, None),
            || "error",
        )
    });
    view! {//<Block>
        <Flex>
            <div style="min-width:150px;">
                <Caption1Strong>"Notation "{name}</Caption1Strong>
            </div>
            <div style="min-width:100px;">
                <span><math>{not}</math></span>
            </div>
            <Text>" for "{target}</Text>
        </Flex>
        //</Block>
    }
}

fn view_inputref<Be: ftml_dom::utils::local_cache::SendBackend>(
    uri: &DocumentUri,
) -> impl IntoView + 'static {
    use crate::utils::collapsible::LazyCollapsible;
    let name = uri.as_view::<Be>();
    let uri = uri.clone();
    view! {
    <LazyCollapsible>
        <Header slot>
            <Caption1Strong>"Document "{name}</Caption1Strong>
        </Header>
        <div style="padding-left:15px;">{
            let uri = uri.clone();
            LocalCache::with_or_toast::<Be,_,_,_,_>(
                move |b| b.get_document(uri), move |d| {
                    let title = d.title.as_ref().map(ToString::to_string);
                    view!{
                    {title.map(|s|
                        view!(<Caption1Strong>{crate::Views::<Be>::render_ftml(s)}</Caption1Strong>)
                    )}
                   { d.as_view::<Be>() }
                }
                },
                || "error"
            )
        }</div>
    </LazyCollapsible>
    }
}

fn view_module<Be: ftml_dom::utils::local_cache::SendBackend>(
    uri: &ModuleUri,
    children: &[DocumentElement],
) -> impl IntoView + 'static {
    let name = uri.as_view::<Be>();
    let imports = children.iter().flat().filter_map(|e| {
        if let DocumentElement::ImportModule(u) = e {
            Some(u)
        } else {
            None
        }
    });
    let imports = super::uses::<Be, _>("Imports", imports);
    let children = children
        .iter()
        .map(FtmlViewable::as_view::<Be>)
        .collect_view();
    view! {<Block show_separator=true>
        <Header slot>
            <Caption1Strong>"Module "{name}</Caption1Strong>
        </Header>
        <HeaderRight slot>{imports}</HeaderRight>
        {children}
    </Block>}
}

fn view_structure<Be: ftml_dom::utils::local_cache::SendBackend>(
    uri: &SymbolUri,
    children: &[DocumentElement],
) -> impl IntoView + 'static {
    let name = uri.as_view::<Be>();
    let imports = children.iter().flat().filter_map(|e| {
        if let DocumentElement::ImportModule(u) = e {
            Some(u)
        } else {
            None
        }
    });
    let imports = super::uses::<Be, _>("Extends", imports);
    let children = children
        .iter()
        .map(FtmlViewable::as_view::<Be>)
        .collect_view();
    view! {<Block show_separator=false>
        <Header slot>
            <Caption1Strong>"Structure "{name}</Caption1Strong>
        </Header>
        <HeaderRight slot>{imports}</HeaderRight>
        {children}
    </Block>}
}

fn view_extension<Be: ftml_dom::utils::local_cache::SendBackend>(
    uri: &SymbolUri,
    target: &SymbolUri,
    children: &[DocumentElement],
) -> impl IntoView + 'static {
    let name = uri.as_view::<Be>();
    let target = target.as_view::<Be>();
    let imports = children.iter().flat().filter_map(|e| {
        if let DocumentElement::ImportModule(u) = e {
            Some(u)
        } else {
            None
        }
    });
    let imports = super::uses::<Be, _>("Extends", imports);
    let children = children
        .iter()
        .map(FtmlViewable::as_view::<Be>)
        .collect_view();
    view! {<Block show_separator=false>
        <Header slot>
            <Caption1Strong>"Conservative Extension "{name}" for "{target}</Caption1Strong>
        </Header>
        <HeaderRight slot>{imports}</HeaderRight>
        {children}
    </Block>}
}
