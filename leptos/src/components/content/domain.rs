use crate::{
    components::content::FtmlViewable,
    utils::{
        Header,
        block::{Block, HeaderLeft, HeaderRight},
    },
};
use ftml_dom::{notations::TermExt, utils::local_cache::SendBackend};
use ftml_ontology::domain::declarations::{
    morphisms::{Assignment, Morphism},
    symbols::ArgumentSpec,
};
use leptos::prelude::*;
use thaw::{Caption1, Caption1Strong, Divider};

impl FtmlViewable for Morphism {
    #[inline]
    fn as_view<Be: SendBackend>(&self) -> impl leptos::IntoView + use<Be> + 'static {
        morphism::<Be, ()>(self, None)
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
