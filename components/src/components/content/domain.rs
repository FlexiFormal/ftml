use crate::{
    components::content::{CommaSep, FtmlViewable},
    utils::{
        Header,
        block::{Block, HeaderLeft, HeaderRight},
    },
};
use ftml_component_utils::{BoldCaption, Caption, Divider, Text};
use ftml_dom::notations::TermExt;
use ftml_ontology::{
    domain::{
        HasDeclarations,
        declarations::{
            AnyDeclarationRef,
            morphisms::{Assignment, Morphism},
            structures::{MathStructure, StructureExtension},
            symbols::ArgumentSpec,
        },
        modules::{Module, ModuleLike, NestedModule},
    },
    terms::Term,
};
use ftml_uris::Id;
use leptos::prelude::*;

impl FtmlViewable for ModuleLike {
    fn as_view(&self) -> AnyView {
        match self {
            Self::Module(m) => m.as_view(),
            Self::Structure(s) => s.as_view(),
            Self::Extension(s) => s.as_view(),
            Self::Nested(s) => s.as_view(),
            Self::Morphism(s) => s.as_view(),
        }
    }
}

impl FtmlViewable for AnyDeclarationRef<'_> {
    fn as_view(&self) -> AnyView {
        match self {
            Self::Import { .. } => ().into_any(),
            Self::Morphism(m) => m.as_view(),
            Self::Symbol(s) => super::symbols::symbol_view(s, true),
            Self::MathStructure(m) => m.as_view(),
            Self::Extension(e) => e.as_view(),
            Self::NestedModule(m) => m.as_view(),
            Self::Rule {
                id,
                parameters: args,
                ..
            } => rule(id, args),
        }
    }
}

fn rule(id: &Id, args: &[Term]) -> AnyView {
    let id = id.to_string();
    let header = view! {
        <BoldCaption>"Inference Rule "{id}</BoldCaption>
    };
    let fors = CommaSep(
        "for",
        args.iter()
            .map(|t| t.clone().into_view::<crate::Views>(crate::backend(), false)),
    )
    .into_view();
    view! {
        <Block show_separator=false>
            <Header slot>{header}</Header>
            <HeaderRight slot><Text>{fors}</Text></HeaderRight>
            ""
        </Block>
    }
    .into_any()
}

impl FtmlViewable for MathStructure {
    fn as_view(&self) -> AnyView {
        let name = self.uri.as_view();
        let imports = self.declarations().filter_map(|e| {
            if let AnyDeclarationRef::Import { uri, .. } = e {
                Some(uri)
            } else {
                None
            }
        });
        let imports = super::uses("Extends", imports);
        let children = self.declarations().map(|e| e.as_view()).collect_view();
        let paragraphs = super::symbols::do_paragraphs(self.uri.clone());
        let macroname = self
            .macroname
            .as_ref()
            .map(|n| super::symbols::do_macroname(n, &ArgumentSpec::default()));
        view! {<Block show_separator=true>
            <Header slot>
                <BoldCaption>"Structure "{name}</BoldCaption>
                {macroname}
            </Header>
            <HeaderRight slot>{imports}</HeaderRight>
            {paragraphs}
            {children}
        </Block>}
        .into_any()
    }
}

impl FtmlViewable for StructureExtension {
    fn as_view(&self) -> AnyView {
        let name = self.uri.as_view();
        let target = self.target.as_view();
        let imports = self.declarations().filter_map(|e| {
            if let AnyDeclarationRef::Import { uri, .. } = e {
                Some(uri)
            } else {
                None
            }
        });
        let imports = super::uses("Extends", imports);
        let children = self.declarations().map(|d| d.as_view()).collect_view();
        view! {<Block show_separator=false>
            <Header slot>
                <BoldCaption>"Conservative Extension "{name}" for "{target}</BoldCaption>
            </Header>
            <HeaderRight slot>{imports}</HeaderRight>
            {children}
        </Block>}
        .into_any()
    }
}

impl FtmlViewable for Morphism {
    #[inline]
    fn as_view(&self) -> AnyView {
        morphism(self, None)
    }
}

impl FtmlViewable for NestedModule {
    fn as_view(&self) -> AnyView {
        let name = super::module_with_hover(&self.uri.clone().into_module());
        let imports = self.declarations().filter_map(|e| {
            if let AnyDeclarationRef::Import { uri, .. } = e {
                Some(uri)
            } else {
                None
            }
        });
        let imports = super::uses("Imports", imports);
        let children = self.declarations().map(|d| d.as_view()).collect_view();
        view! {<Block show_separator=true>
            <Header slot>
                <BoldCaption>"Nested Module "{name}</BoldCaption>
            </Header>
            <HeaderRight slot>{imports}</HeaderRight>
            {children}
        </Block>}
        .into_any()
    }
}

impl FtmlViewable for Module {
    fn as_view(&self) -> AnyView {
        let name = super::module_with_hover(&self.uri);
        let imports = self.declarations().filter_map(|e| {
            if let AnyDeclarationRef::Import { uri, .. } = e {
                Some(uri)
            } else {
                None
            }
        });
        let imports = super::uses("Imports", imports);
        let children = self.declarations().map(|d| d.as_view()).collect_view();
        view! {<Block show_separator=true>
            <Header slot>
                <BoldCaption>"Module "{name}</BoldCaption>
            </Header>
            <HeaderRight slot>{imports}</HeaderRight>
            {children}
        </Block>}
        .into_any()
    }
}

pub fn morphism(m: &Morphism, doc_elems: Option<AnyView>) -> AnyView {
    let domain = m.domain.as_view();
    let name = m.uri.as_view();
    let assignments = m.elements.iter().map(do_assignment).collect_view();
    let elems = doc_elems.map(move |elems| {
        view! {
            <div style="margin:5px;"><Divider/></div>
            {elems}
        }
    });
    view! {<Block>
            <Header slot>
                <BoldCaption>"Morphism "{name}</BoldCaption>
            </Header>
            <HeaderLeft slot><Caption>"From "{domain}</Caption></HeaderLeft>
            {assignments}
            {elems}
        </Block>
    }
    .into_any()
}

fn do_assignment(a: &Assignment) -> AnyView {
    let elaborated_uri = a.elaborated_uri();
    let name = super::symbol_uri(elaborated_uri.name().to_string(), &elaborated_uri);
    let header = view!(<BoldCaption>"Symbol "{name}</BoldCaption>);
    let orig = a.original.as_view();
    let paragraphs = super::symbols::do_paragraphs(elaborated_uri.clone());
    let notations = super::symbols::do_notations(elaborated_uri.into(), ArgumentSpec::default());
    let df = a.definiens.as_ref().map(|t| {
        let t = t.clone().into_view::<crate::Views>(crate::backend(), false);
        view! {<Caption>
            "Assigned to: "{ftml_dom::utils::math(|| t)}
            </Caption>
        }
        .attr("style", "white-space:nowrap;")
    });
    view! {
        <Block show_separator=true>
            <Header slot>{header}</Header>
            <HeaderLeft slot>
                <Caption>"Elaborated from "{orig}</Caption>
            </HeaderLeft>
            <HeaderRight slot>{df}</HeaderRight>
            {notations}
            {paragraphs}
        </Block>
    }
    .into_any()
}
