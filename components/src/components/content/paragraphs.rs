use crate::{
    components::content::{CommaSep, FtmlViewable},
    utils::{
        Header,
        block::{Block, HeaderLeft, HeaderRight},
    },
};
use ftml_dom::{FtmlViews, notations::TermExt, utils::css::inject_css};
use ftml_ontology::narrative::elements::{
    DocumentElement, FlatIterable, LogicalParagraph, Section,
};
use ftml_uris::DocumentElementUri;
use leptos::prelude::*;
use thaw::Caption1Strong;

pub(super) fn slide<Be: ftml_dom::utils::local_cache::SendBackend>(
    uri: &DocumentElementUri,
    title: Option<&str>,
    children: &[DocumentElement],
) -> impl IntoView + use<Be> + 'static {
    use leptos::either::Either::{Left, Right};
    let title = super::hover_paragraph::<Be>(
        uri.clone(),
        view!(<span style="font-style:italic;">{
            title.as_ref().map_or_else(
                || Right(uri.name().last().to_string()),
                |t| Left(crate::Views::<Be>::render_ftml((*t).to_string(), None)),
            )
        }</span>),
    );
    let uses = children.iter().flat().filter_map(|e| {
        if let DocumentElement::UseModule(u) = e {
            Some(u)
        } else {
            None
        }
    });
    let uses = super::uses::<Be, _>("Uses", uses);
    let children = children
        .iter()
        .map(FtmlViewable::as_view::<Be>)
        .collect_view();

    view! {
      <Block>
        <Header slot><Caption1Strong>
            "Slide "{title}
        </Caption1Strong></Header>
        <HeaderLeft slot>{uses}</HeaderLeft>
        {children}
      </Block>
    }
}

impl FtmlViewable for Section {
    fn as_view<Be: ftml_dom::utils::local_cache::SendBackend>(
        &self,
    ) -> impl IntoView + use<Be> + 'static {
        use leptos::either::Either::{Left, Right};

        let Self {
            uri,
            title,
            children,
            ..
        } = self;
        let title = title.as_ref().map_or_else(
            || Right(uri.name().last().to_string()),
            |t| Left(crate::Views::<Be>::render_ftml(t.to_string(), None)),
        );
        let uses = children.iter().flat().filter_map(|e| {
            if let DocumentElement::UseModule(u) = e {
                Some(u)
            } else {
                None
            }
        });
        let uses = super::uses::<Be, _>("Uses", uses);
        let children = children
            .iter()
            .map(FtmlViewable::as_view::<Be>)
            .collect_view();

        view! {
          <Block>
            <Header slot><Caption1Strong>
                "Section "{title}
            </Caption1Strong></Header>
            <HeaderLeft slot>{uses}</HeaderLeft>
            {children}
          </Block>
        }
    }
}

impl FtmlViewable for LogicalParagraph {
    fn as_view<Be: ftml_dom::utils::local_cache::SendBackend>(
        &self,
    ) -> impl IntoView + use<Be> + 'static {
        use leptos::either::Either::{Left, Right};
        let Self {
            kind,
            uri,
            title,
            styles,
            children,
            fors,
            ..
        } = self;
        let title = title.as_ref().map_or_else(
            || Right(uri.as_view::<Be>()),
            |t| {
                Left(super::hover_paragraph::<Be>(
                    uri.clone(),
                    view!(<span style="font-style:italic;">{
                        crate::Views::<Be>::render_ftml(t.to_string(), None)}
                    </span>),
                ))
            },
        );
        let uses = children.iter().flat().filter_map(|e| {
            if let DocumentElement::UseModule(u) = e {
                Some(u)
            } else {
                None
            }
        });
        let uses = super::uses::<Be, _>("Uses", uses);
        let definition_like = kind.is_definition_like(styles);
        let kind = kind.as_display_str();
        let children = children
            .iter()
            .map(FtmlViewable::as_view::<Be>)
            .collect_view();
        let fors = CommaSep(
            if definition_like {
                "Defines"
            } else {
                "Concerns"
            },
            fors.iter().map(|(k, t)| {
                let name = k.as_view::<Be>();
                let t = t.clone().map(|t| {
                    let t = t.into_view_safe::<crate::Views<Be>, Be>();
                    view!(<span>" as "{ftml_dom::utils::math(|| t)}</span>)
                });
                view!({name}{t})
            }),
        )
        .into_view();
        view! {
          <Block>
            <Header slot><Caption1Strong>
                {kind}" "{title}
            </Caption1Strong></Header>
            <HeaderLeft slot>{uses}</HeaderLeft>
            <HeaderRight slot>{fors}</HeaderRight>
            {children}
          </Block>
        }
    }
}
