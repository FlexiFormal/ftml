use crate::{
    components::content::FtmlViewable,
    utils::{
        Header,
        block::{Block, HeaderLeft, HeaderRight},
    },
};
use ftml_dom::{notations::TermExt, utils::local_cache::SendBackend};
use ftml_ontology::domain::{
    HasDeclarations,
    declarations::{
        AnyDeclarationRef,
        morphisms::{Assignment, Morphism},
        structures::{MathStructure, StructureExtension},
        symbols::ArgumentSpec,
    },
    modules::{Module, ModuleLike, NestedModule},
};
use leptos::prelude::*;
use thaw::{Caption1, Caption1Strong, Divider};

impl FtmlViewable for ModuleLike {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Be> + 'static {
        use leptos::either::EitherOf5::{A, B, C, D, E};
        match self {
            Self::Module(m) => A(m.as_view::<Be>()),
            Self::Structure(s) => B(s.as_view::<Be>()),
            Self::Extension(s) => C(s.as_view::<Be>()),
            Self::Nested(s) => D(s.as_view::<Be>()),
            Self::Morphism(s) => E(s.as_view::<Be>()),
        }
    }
}

impl FtmlViewable for AnyDeclarationRef<'_> {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Be> + 'static {
        use leptos::either::EitherOf6::{A, B, C, D, E, F};
        match self {
            Self::Import(_) => A(()),
            Self::Morphism(m) => B(m.as_view::<Be>()),
            Self::Symbol(s) => C(super::symbols::symbol_view::<Be>(s, true)),
            Self::MathStructure(m) => D(m.as_view::<Be>()),
            Self::Extension(e) => E(e.as_view::<Be>()),
            Self::NestedModule(m) => F(m.as_view::<Be>()),
        }
    }
}

impl FtmlViewable for MathStructure {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Be> + 'static {
        let name = self.uri.as_view::<Be>();
        let imports = self.declarations().filter_map(|e| {
            if let AnyDeclarationRef::Import(u) = e {
                Some(u)
            } else {
                None
            }
        });
        let imports = super::uses::<Be, _>("Extends", imports);
        let children = self
            .declarations()
            .map(|e| e.as_view::<Be>())
            .collect_view();
        let paragraphs = super::symbols::do_paragraphs::<Be>(self.uri.clone());
        let macroname = self
            .macroname
            .as_ref()
            .map(|n| super::symbols::do_macroname(n, &ArgumentSpec::default()));
        view! {<Block show_separator=true>
            <Header slot>
                <Caption1Strong>"Structure "{name}</Caption1Strong>
                {macroname}
            </Header>
            <HeaderRight slot>{imports}</HeaderRight>
            {paragraphs}
            {children}
        </Block>}
    }
}

impl FtmlViewable for StructureExtension {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Be> + 'static {
        let name = self.uri.as_view::<Be>();
        let target = self.target.as_view::<Be>();
        let imports = self.declarations().filter_map(|e| {
            if let AnyDeclarationRef::Import(u) = e {
                Some(u)
            } else {
                None
            }
        });
        let imports = super::uses::<Be, _>("Extends", imports);
        let children = self
            .declarations()
            .map(|d| d.as_view::<Be>())
            .collect_view();
        view! {<Block show_separator=false>
            <Header slot>
                <Caption1Strong>"Conservative Extension "{name}" for "{target}</Caption1Strong>
            </Header>
            <HeaderRight slot>{imports}</HeaderRight>
            {children}
        </Block>}
    }
}

impl FtmlViewable for Morphism {
    #[inline]
    fn as_view<Be: SendBackend>(&self) -> impl leptos::IntoView + use<Be> + 'static {
        morphism::<Be, ()>(self, None)
    }
}

impl FtmlViewable for NestedModule {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Be> + 'static {
        let name = super::module_with_hover(&self.uri.clone().into_module());
        let imports = self.declarations().filter_map(|e| {
            if let AnyDeclarationRef::Import(u) = e {
                Some(u)
            } else {
                None
            }
        });
        let imports = super::uses::<Be, _>("Imports", imports);
        let children = self
            .declarations()
            .map(|d| d.as_view::<Be>())
            .collect_view();
        view! {<Block show_separator=true>
            <Header slot>
                <Caption1Strong>"Nested Module "{name}</Caption1Strong>
            </Header>
            <HeaderRight slot>{imports}</HeaderRight>
            {children}
        </Block>}
    }
}

impl FtmlViewable for Module {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Be> + 'static {
        let name = super::module_with_hover(&self.uri);
        let imports = self.declarations().filter_map(|e| {
            if let AnyDeclarationRef::Import(u) = e {
                Some(u)
            } else {
                None
            }
        });
        let imports = super::uses::<Be, _>("Imports", imports);
        let children = self
            .declarations()
            .map(|d| d.as_view::<Be>())
            .collect_view();
        view! {<Block show_separator=true>
            <Header slot>
                <Caption1Strong>"Module "{name}</Caption1Strong>
            </Header>
            <HeaderRight slot>{imports}</HeaderRight>
            {children}
        </Block>}
    }
}

pub fn morphism<Be: SendBackend, V: IntoView + 'static>(
    m: &Morphism,
    doc_elems: Option<V>,
) -> impl IntoView + use<Be, V> + 'static {
    let domain = m.domain.as_view::<Be>();
    let name = m.uri.as_view::<Be>();
    let assignments = m.elements.iter().map(do_assignment::<Be>).collect_view();
    let elems = doc_elems.map(move |elems| {
        view! {
            <div style="margin:5px;"><Divider/></div>
            {elems}
        }
    });
    view! {<Block>
            <Header slot>
                <Caption1Strong>"Morphism "{name}</Caption1Strong>
            </Header>
            <HeaderLeft slot><Caption1>"From "{domain}</Caption1></HeaderLeft>
            {assignments}
            {elems}
        </Block>
    }
}

fn do_assignment<Be: SendBackend>(a: &Assignment) -> impl IntoView + use<Be> + 'static {
    let elaborated_uri = a.elaborated_uri();
    let name = super::symbol_uri::<Be>(elaborated_uri.name().to_string(), &elaborated_uri);
    let header = view!(<Caption1Strong>"Symbol "{name}</Caption1Strong>);
    let orig = a.original.as_view::<Be>();
    let paragraphs = super::symbols::do_paragraphs::<Be>(elaborated_uri.clone());
    let notations =
        super::symbols::do_notations::<Be>(elaborated_uri.into(), ArgumentSpec::default());
    let df = a.definiens.as_ref().map(|t| {
        let t = t.clone().into_view::<crate::Views<Be>, Be>(false);
        view! {<Caption1>
            "Assigned to: "{ftml_dom::utils::math(|| t)}
            </Caption1>
        }
        .attr("style", "white-space:nowrap;")
    });
    view! {
        <Block show_separator=true>
            <Header slot>{header}</Header>
            <HeaderLeft slot>
                <Caption1>"Elaborated from "{orig}</Caption1>
            </HeaderLeft>
            <HeaderRight slot>{df}</HeaderRight>
            {notations}
            {paragraphs}
        </Block>
    }
}
