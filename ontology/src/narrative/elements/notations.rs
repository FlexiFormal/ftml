use crate::{
    Ftml,
    narrative::{
        DataRef, Narrative,
        elements::{DocumentElementRef, IsDocumentElement},
    },
    terms::ArgumentMode,
    utils::RefTree,
};
use ftml_uris::{DocumentElementUri, Id, NarrativeUriRef, SymbolUri};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct NotationReference {
    pub symbol: SymbolUri,
    pub uri: DocumentElementUri,
    #[cfg_attr(feature = "typescript", tsify(type = "DataRef"))]
    pub notation: DataRef<Notation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct VariableNotationReference {
    pub variable: DocumentElementUri,
    pub uri: DocumentElementUri,
    #[cfg_attr(feature = "typescript", tsify(type = "DataRef"))]
    pub notation: DataRef<Notation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Notation {
    pub precedence: i64,
    #[cfg_attr(feature = "serde", serde(default))]
    pub id: Option<Id>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub argprecs: Vec<i64>,
    pub component: NotationComponent,
    #[cfg_attr(feature = "serde", serde(default))]
    pub op: Option<NotationNode>,
}
impl Notation {
    #[must_use]
    pub fn is_op(&self) -> bool {
        self.op.is_some()
            || !self.component.dfs().any(|c| {
                matches!(
                    c,
                    NotationComponent::Argument { .. }
                        | NotationComponent::ArgMap { .. }
                        | NotationComponent::ArgSep { .. }
                )
            })
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum NotationComponent {
    Node {
        tag: ftml_uris::Id,
        #[cfg_attr(feature = "serde", serde(default))]
        attributes: Box<[(ftml_uris::Id, Box<str>)]>,
        #[cfg_attr(feature = "serde", serde(default))]
        children: Box<[NotationComponent]>,
    },
    Argument {
        index: u8,
        mode: ArgumentMode,
    },
    ArgSep {
        index: u8,
        mode: ArgumentMode,
        #[cfg_attr(feature = "serde", serde(default))]
        sep: Box<[NotationComponent]>,
    },
    ArgMap {
        index: u8,
        #[cfg_attr(feature = "serde", serde(default))]
        segments: Box<[NotationComponent]>,
    },
    MainComp(NotationNode),
    Comp(NotationNode),
    #[cfg_attr(feature = "serde", serde(untagged))]
    Text(Box<str>),
}
impl crate::__private::Sealed for NotationReference {}
impl Ftml for NotationReference {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;
        let iri = self.uri.to_iri();
        [
            triple!(<(iri.clone())>: ulo:notation),
            triple!(<(iri)> ulo:notation_for <(self.symbol.to_iri())>),
        ]
    }
}
impl Narrative for NotationReference {
    fn narrative_uri(&self) -> Option<NarrativeUriRef<'_>> {
        Some(NarrativeUriRef::Element(&self.uri))
    }
    #[inline]
    fn children(
        &self,
    ) -> impl ExactSizeIterator<Item = DocumentElementRef<'_>> + DoubleEndedIterator {
        std::iter::empty()
    }
}
impl IsDocumentElement for NotationReference {
    fn as_ref(&self) -> DocumentElementRef<'_> {
        DocumentElementRef::Notation(self)
    }
    fn from_element(e: DocumentElementRef<'_>) -> Option<&Self>
    where
        Self: Sized,
    {
        if let DocumentElementRef::Notation(n) = e {
            Some(n)
        } else {
            None
        }
    }
    fn element_uri(&self) -> Option<&DocumentElementUri> {
        Some(&self.uri)
    }
}
impl crate::__private::Sealed for VariableNotationReference {}
impl Ftml for VariableNotationReference {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;
        let iri = self.uri.to_iri();
        [
            triple!(<(iri.clone())>: ulo:notation),
            triple!(<(iri)> ulo:notation_for <(self.variable.to_iri())>),
        ]
    }
}
impl Narrative for VariableNotationReference {
    fn narrative_uri(&self) -> Option<NarrativeUriRef<'_>> {
        Some(NarrativeUriRef::Element(&self.uri))
    }
    #[inline]
    fn children(
        &self,
    ) -> impl ExactSizeIterator<Item = DocumentElementRef<'_>> + DoubleEndedIterator {
        std::iter::empty()
    }
}
impl IsDocumentElement for VariableNotationReference {
    fn as_ref(&self) -> DocumentElementRef<'_> {
        DocumentElementRef::VariableNotation(self)
    }
    fn from_element(e: DocumentElementRef<'_>) -> Option<&Self>
    where
        Self: Sized,
    {
        if let DocumentElementRef::VariableNotation(n) = e {
            Some(n)
        } else {
            None
        }
    }
    fn element_uri(&self) -> Option<&DocumentElementUri> {
        Some(&self.uri)
    }
}

impl crate::utils::RefTree for NotationComponent {
    type Child<'a>
        = &'a Self
    where
        Self: 'a;
    fn tree_children(&self) -> impl Iterator<Item = Self::Child<'_>> {
        match self {
            Self::Comp(_) | Self::Text(_) | Self::MainComp(_) | Self::Argument { .. } => {
                either::Left(std::iter::empty())
            }
            Self::Node { children, .. }
            | Self::ArgSep { sep: children, .. }
            | Self::ArgMap {
                segments: children, ..
            } => either::Right(children.iter()),
        }
    }
}

impl std::fmt::Debug for NotationComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(t) => write!(f, "\"{t}\""),
            Self::Argument { index, mode } => write!(f, "<arg {index}@{mode:?}/>"),
            Self::Node {
                tag,
                attributes,
                children,
            } => {
                write!(f, "<{tag}")?;
                for (k, v) in attributes {
                    write!(f, " {k}=\"{v}\"")?;
                }
                f.write_str(">\n")?;
                for t in children {
                    writeln!(f, "{t:?}")?;
                }
                write!(f, "</{tag}>")
            }
            Self::ArgMap { index, segments } => {
                write!(f, "<argmap {index}>")?;
                for s in segments {
                    s.fmt(f)?;
                }
                write!(f, "<argmap/>")
            }
            Self::ArgSep { index, mode, sep } => {
                write!(f, "<argsep {index}@{mode:?}>")?;
                for s in sep {
                    s.fmt(f)?;
                }
                write!(f, "<argsep/>")
            }
            Self::MainComp(c) => {
                write!(f, "<{} data-ftml-maincomp", c.tag)?;
                for (k, v) in &c.attributes {
                    write!(f, " {k}=\"{v}\"")?;
                }
                f.write_str(">\n")?;
                for t in &c.children {
                    writeln!(f, "{t:?}")?;
                }
                write!(f, "</{}>", c.tag)
            }
            Self::Comp(c) => {
                write!(f, "<{} data-ftml-comp", c.tag)?;
                for (k, v) in &c.attributes {
                    write!(f, " {k}=\"{v}\"")?;
                }
                f.write_str(">\n")?;
                for t in &c.children {
                    writeln!(f, "{t:?}")?;
                }
                write!(f, "</{}>", c.tag)
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct NotationNode {
    pub tag: ftml_uris::Id,
    #[cfg_attr(feature = "serde", serde(default))]
    pub attributes: Box<[(ftml_uris::Id, Box<str>)]>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub children: Box<[NodeOrText]>,
}

impl std::fmt::Debug for NotationNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}", self.tag)?;
        for (k, v) in &self.attributes {
            write!(f, " {k}=\"{v}\"")?;
        }
        f.write_str(">\n")?;
        for t in &self.children {
            writeln!(f, "{t:?}")?;
        }
        write!(f, "</{}>", self.tag)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum NodeOrText {
    Node(NotationNode),
    Text(Box<str>),
}

impl std::fmt::Debug for NodeOrText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(t) => write!(f, "\"{t}\""),
            Self::Node(NotationNode {
                tag,
                attributes,
                children,
            }) => {
                write!(f, "<{tag}")?;
                for (k, v) in attributes {
                    write!(f, " {k}=\"{v}\"")?;
                }
                f.write_str(">\n")?;
                for t in children {
                    writeln!(f, "{t:?}")?;
                }
                write!(f, "</{tag}>")
            }
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for NodeOrText {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        match self {
            Self::Node(n) => n.deep_size_of_children(context),
            Self::Text(t) => t.len(),
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for NotationNode {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.children
            .iter()
            .map(|t| std::mem::size_of_val(t) + t.deep_size_of_children(context))
            .sum::<usize>()
            + self
                .attributes
                .iter()
                .map(|p| std::mem::size_of_val(p) + p.1.len())
                .sum::<usize>()
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for NotationComponent {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        match self {
            Self::Node {
                attributes,
                children,
                ..
            } => {
                attributes
                    .iter()
                    .map(|p| std::mem::size_of_val(p) + p.1.len())
                    .sum::<usize>()
                    + children
                        .iter()
                        .map(|p| std::mem::size_of_val(p) + p.deep_size_of_children(context))
                        .sum::<usize>()
            }
            Self::ArgSep { sep, .. } | Self::ArgMap { segments: sep, .. } => sep
                .iter()
                .map(|p| std::mem::size_of_val(p) + p.deep_size_of_children(context))
                .sum::<usize>(),
            Self::Comp(c) | Self::MainComp(c) => c.deep_size_of_children(context),
            Self::Text(t) => t.len(),
            Self::Argument { .. } => 0,
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for Notation {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.component.deep_size_of_children(context)
            + self
                .op
                .as_ref()
                .map(|s| s.deep_size_of_children(context))
                .unwrap_or_default()
    }
}
