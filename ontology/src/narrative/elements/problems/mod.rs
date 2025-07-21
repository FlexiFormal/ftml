mod feedback;
//mod quizzes;
mod solutions;

use crate::narrative::{
    DataRef, DocumentRange, Narrative,
    elements::{DocumentElement, DocumentElementRef, IsDocumentElement},
};
pub use feedback::*;
use ftml_uris::{DocumentElementUri, Id, SymbolUri};
//pub use quizzes::*;
pub use solutions::*;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Problem {
    pub sub_problem: bool,
    pub uri: DocumentElementUri,
    pub range: DocumentRange,
    pub autogradable: bool,
    pub points: Option<f32>,
    pub solutions: DataRef<Solutions>, //State::Seq<SolutionData>,
    pub gnotes: Box<[DataRef<GradingNote>]>,
    pub hints: Box<[DocumentRange]>,
    pub notes: Box<[DataRef<Box<str>>]>,
    pub title: Option<DocumentRange>,
    pub children: Box<[DocumentElement]>,
    pub styles: Box<[Id]>,
    pub preconditions: Box<[(CognitiveDimension, SymbolUri)]>,
    pub objectives: Box<[(CognitiveDimension, SymbolUri)]>,
}
impl crate::__private::Sealed for Problem {}
impl crate::Ftml for Problem {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;
        let iri = self.uri.to_iri();
        let iri2 = iri.clone();
        let iri3 = iri.clone();

        self.contains_triples()
            .into_iter()
            .chain(self.objectives.iter().flat_map(move |(d, s)| {
                let b = ulo::rdf_types::BlankNode::default();
                [
                    triple!(<(iri2.clone())> ulo:has_objective (b.clone())!),
                    triple!((b.clone())! ulo:has_cognitive_dimension <(d.to_iri().into_owned())>),
                    triple!((b)! ulo:po_has_symbol <(s.to_iri())>),
                ]
            }))
            .chain(self.preconditions.iter().flat_map(move |(d, s)| {
                let b = ulo::rdf_types::BlankNode::default();
                [
                    triple!(<(iri3.clone())> ulo:has_precondition (b.clone())!),
                    triple!((b.clone())! ulo:has_cognitive_dimension <(d.to_iri().into_owned())>),
                    triple!((b)! ulo:po_has_symbol <(s.to_iri())>),
                ]
            }))
            .chain(std::iter::once(if self.sub_problem {
                triple!(<(iri)> : ulo:subproblem)
            } else {
                triple!(<(iri)> : ulo:problem)
            }))
    }
}
impl Narrative for Problem {
    #[inline]
    fn narrative_uri(&self) -> Option<ftml_uris::NarrativeUriRef<'_>> {
        Some(ftml_uris::NarrativeUriRef::Element(&self.uri))
    }
    #[inline]
    fn children(&self) -> &[DocumentElement] {
        &self.children
    }
}
impl IsDocumentElement for Problem {
    #[inline]
    fn element_uri(&self) -> Option<&DocumentElementUri> {
        Some(&self.uri)
    }
    #[inline]
    fn as_ref(&self) -> DocumentElementRef<'_> {
        DocumentElementRef::Problem(self)
    }
    #[inline]
    fn from_element(e: DocumentElementRef<'_>) -> Option<&Self>
    where
        Self: Sized,
    {
        match e {
            DocumentElementRef::Problem(p) => Some(p),
            _ => None,
        }
    }
}

impl Eq for Problem {}
impl PartialEq for Problem {
    fn eq(&self, other: &Self) -> bool {
        self.sub_problem == other.sub_problem && self.uri == other.uri
    }
}
impl std::hash::Hash for Problem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.sub_problem.hash(state);
        self.uri.hash(state);
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct GradingNote {
    pub html: Box<str>,
    pub answer_classes: Vec<AnswerClass>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct AnswerClass {
    pub id: Id,
    pub feedback: Box<str>,
    pub kind: AnswerKind,
}
impl Eq for AnswerClass {}
impl PartialEq for AnswerClass {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}
impl std::hash::Hash for AnswerClass {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum AnswerKind {
    Class(f32),
    Trait(f32),
}

#[derive(Debug, thiserror::Error)]
#[error("invalid value for answerclass kind")]
pub struct InvalidAnswerKind;

impl std::str::FromStr for AnswerKind {
    type Err = InvalidAnswerKind;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[allow(clippy::cast_precision_loss)]
        fn num(s: &str) -> Result<f32, InvalidAnswerKind> {
            if s.contains('.') {
                s.parse().map_err(|_| InvalidAnswerKind)
            } else {
                let i: Result<i32, InvalidAnswerKind> = s.parse().map_err(|_| InvalidAnswerKind);
                i.map(|i| i as _)
            }
        }
        let s = s.trim();
        s.strip_prefix('+').map_or_else(
            || {
                s.strip_prefix('-').map_or_else(
                    || num(s).map(AnswerKind::Class),
                    |s| num(s).map(|f| Self::Trait(-f)),
                )
            },
            |s| num(s).map(AnswerKind::Trait),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum CognitiveDimension {
    Remember,
    Understand,
    Apply,
    Analyze,
    Evaluate,
    Create,
}
impl CognitiveDimension {
    #[cfg(feature = "rdf")]
    #[must_use]
    #[allow(clippy::enum_glob_use)]
    pub const fn to_iri(&self) -> ulo::rdf_types::NamedNodeRef<'static> {
        use CognitiveDimension::*;
        use ulo::ulo;
        match self {
            Remember => ulo::remember,
            Understand => ulo::understand,
            Apply => ulo::apply,
            Analyze => ulo::analyze,
            Evaluate => ulo::evaluate,
            Create => ulo::create,
        }
    }

    #[cfg(feature = "rdf")]
    #[must_use]
    pub fn from_iri(iri: ulo::rdf_types::NamedNodeRef) -> Option<Self> {
        Some(match iri {
            ulo::ulo::remember => Self::Remember,
            ulo::ulo::understand => Self::Understand,
            ulo::ulo::apply => Self::Apply,
            ulo::ulo::analyze => Self::Analyze,
            ulo::ulo::evaluate => Self::Evaluate,
            ulo::ulo::create => Self::Create,
            _ => return None,
        })
    }
}
impl std::fmt::Display for CognitiveDimension {
    #[allow(clippy::enum_glob_use)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CognitiveDimension::*;
        write!(
            f,
            "{}",
            match self {
                Remember => "remember",
                Understand => "understand",
                Apply => "apply",
                Analyze => "analyze",
                Evaluate => "evaluate",
                Create => "create",
            }
        )
    }
}

#[derive(Debug, thiserror::Error)]
#[error("string is not a cognitive dimension")]
pub struct NotACognitiveDimension;

impl std::str::FromStr for CognitiveDimension {
    type Err = NotACognitiveDimension;
    #[allow(clippy::enum_glob_use)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use CognitiveDimension::*;
        if s.eq_ignore_ascii_case("remember") {
            return Ok(Remember);
        }
        if s.eq_ignore_ascii_case("understand") {
            return Ok(Understand);
        }
        if s.eq_ignore_ascii_case("apply") {
            return Ok(Apply);
        }
        if s.eq_ignore_ascii_case("analyze") || s.eq_ignore_ascii_case("analyse") {
            return Ok(Analyze);
        }
        if s.eq_ignore_ascii_case("evaluate") {
            return Ok(Evaluate);
        }
        if s.eq_ignore_ascii_case("create") {
            return Ok(Create);
        }
        Err(NotACognitiveDimension)
    }
}
