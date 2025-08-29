use crate::{
    narrative::{
        DataRef, DocumentRange,
        elements::{
            DocumentElement, DocumentElementRef,
            problems::{AnswerClass, CognitiveDimension, GradingNote, Solutions},
        },
    },
    utils::Css,
};
use ftml_uris::{DocumentElementUri, DocumentUri, SymbolUri};
use std::hint::unreachable_unchecked;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Quiz {
    #[cfg_attr(feature = "serde", serde(default))]
    pub css: Box<[Css]>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub title: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub elements: Box<[QuizElement]>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub solutions: rustc_hash::FxHashMap<DocumentElementUri, Box<str>>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub answer_classes: rustc_hash::FxHashMap<DocumentElementUri, Vec<AnswerClass>>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum QuizElement {
    Section {
        title: Box<str>,
        #[cfg_attr(feature = "serde", serde(default))]
        elements: Box<[QuizElement]>,
    },
    Problem(QuizProblem),
    Paragraph {
        html: Box<str>,
    },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct QuizProblem {
    pub html: Box<str>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub title_html: Option<Box<str>>,
    pub uri: DocumentElementUri,
    //pub solution:String,//Solutions,
    #[cfg_attr(feature = "serde", serde(default))]
    pub total_points: Option<f32>,
    //pub is_sub_problem:bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub preconditions: Box<[(CognitiveDimension, SymbolUri)]>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub objectives: Box<[(CognitiveDimension, SymbolUri)]>,
}

impl crate::narrative::Document {
    #[cfg(feature = "serde")]
    #[allow(clippy::redundant_else)]
    #[allow(clippy::too_many_lines)]
    /// # Errors
    pub fn as_quiz(
        &self,
        get_document: &dyn Fn(&DocumentUri) -> Option<Self>,
        get_fragment: &dyn Fn(&DocumentUri, DocumentRange) -> Option<(Box<[Css]>, Box<str>)>,
        get_solutions: &dyn Fn(&DocumentUri, DataRef<Solutions>) -> Option<Solutions>,
        get_gnote: &dyn Fn(&DocumentUri, DataRef<GradingNote>) -> Option<GradingNote>,
    ) -> Result<Quiz, QuizError> {
        let mut css = Vec::new();
        let mut elements = Vec::new();
        let mut in_problem = false;
        let mut quiz = Quiz::default();

        let mut stack: smallvec::SmallVec<_, 2> = smallvec::SmallVec::new();
        let mut curr = self.elements.iter().map(DocumentElement::as_ref);

        macro_rules! push {
            ($c:expr;$e:expr) => {
                stack.push((
                    std::mem::replace(&mut curr, $c),
                    std::mem::take(&mut elements),
                    $e,
                ))
            };
        }
        macro_rules! pop {
            () => {
                if let Some((c, mut e, s)) = stack.pop() {
                    curr = c;
                    std::mem::swap(&mut elements, &mut e);
                    match s {
                        Some(either::Either::Left(s)) => elements.push(QuizElement::Section {
                            title: s,
                            elements: e.into_boxed_slice(),
                        }),
                        Some(either::Either::Right(b)) => {
                            in_problem = b;
                            elements.extend(e.into_iter());
                        }
                        _ => elements.extend(e.into_iter()),
                    }
                    continue;
                } else {
                    break;
                }
            };
        }

        loop {
            let Some(e) = curr.next() else { pop!() };
            match e {
                DocumentElementRef::DocumentReference { target, .. } => {
                    // safety first
                    let Some(d) = get_document(target) else {
                        return Err(QuizError::MissingDocument(target.clone()));
                    };
                    let ret = d.as_quiz(get_document, get_fragment, get_solutions, get_gnote)?;
                    for c in ret.css {
                        if !css.contains(&c) {
                            css.push(c);
                        }
                    }
                    elements.extend(ret.elements);
                    for (u, s) in ret.solutions {
                        quiz.solutions.insert(u, s);
                    }
                }
                DocumentElementRef::Section(sect) => {
                    if let Some(title) = &sect.title {
                        push!(sect.children.iter().map(DocumentElement::as_ref);Some(either::Either::Left(title.clone())));
                    } else {
                        push!(sect.children.iter().map(DocumentElement::as_ref);None);
                    }
                }
                DocumentElementRef::Paragraph(p) => {
                    let Some((c, html)) = get_fragment(&self.uri, p.range) else {
                        return Err(QuizError::MissingFragment(p.uri.clone()));
                    };
                    for c in c {
                        if !css.contains(&c) {
                            css.push(c);
                        }
                    }
                    elements.push(QuizElement::Paragraph { html });
                }
                DocumentElementRef::Problem(e) if in_problem => {
                    let Some(solution) = get_solutions(&self.uri, e.data.solutions) else {
                        return Err(QuizError::MissingSolutions(e.uri.clone()));
                    };
                    let Some(solution) = solution.to_jstring() else {
                        return Err(QuizError::InvalidSolutions(e.uri.clone()));
                    };
                    quiz.solutions
                        .insert(e.uri.clone(), solution.into_boxed_str());
                }
                DocumentElementRef::Problem(e) => {
                    let Some((c, html)) = get_fragment(&self.uri, e.range) else {
                        return Err(QuizError::MissingFragment(e.uri.clone()));
                    };
                    for c in c {
                        if !css.contains(&c) {
                            css.push(c);
                        }
                    }
                    let Some(solution) = get_solutions(&self.uri, e.data.solutions) else {
                        return Err(QuizError::MissingSolutions(e.uri.clone()));
                    };
                    let title_html = if let Some(ttl) = e.data.title {
                        let Some(t) = get_fragment(&self.uri, ttl) else {
                            return Err(QuizError::MissingFragment(e.uri.clone()));
                        };
                        Some(t.1)
                    } else {
                        None
                    };
                    let Some(solution) = solution.to_jstring() else {
                        return Err(QuizError::InvalidSolutions(e.uri.clone()));
                    };
                    for note in &e.data.gnotes {
                        let Some(gnote) = get_gnote(&self.uri, *note) else {
                            return Err(QuizError::MissingGNote(e.uri.clone()));
                        };
                        quiz.answer_classes
                            .entry(e.uri.clone())
                            .or_default()
                            .extend(gnote.answer_classes.iter().cloned());
                    }
                    quiz.solutions
                        .insert(e.uri.clone(), solution.into_boxed_str());
                    elements.push(QuizElement::Problem(QuizProblem {
                        html, //solution,
                        title_html,
                        uri: e.uri.clone(),
                        preconditions: e.data.preconditions.clone(),
                        objectives: e.data.objectives.clone(),
                        total_points: e.data.points,
                    }));
                    push!(e.children.iter().map(DocumentElement::as_ref);Some(either::Either::Right(in_problem)));
                    in_problem = true;
                }
                DocumentElementRef::Module { children, .. }
                | DocumentElementRef::MathStructure { children, .. }
                | DocumentElementRef::Extension { children, .. }
                | DocumentElementRef::Morphism { children, .. }
                | DocumentElementRef::Slide { children, .. }
                | DocumentElementRef::SkipSection(children) => {
                    if !children.is_empty() {
                        push!(children.iter().map(DocumentElement::as_ref);None);
                    }
                }
                DocumentElementRef::UseModule(_)
                | DocumentElementRef::SymbolDeclaration(_)
                | DocumentElementRef::ImportModule(_)
                | DocumentElementRef::VariableDeclaration(_)
                | DocumentElementRef::Definiendum { .. }
                | DocumentElementRef::SymbolReference { .. }
                | DocumentElementRef::VariableReference { .. }
                | DocumentElementRef::Notation { .. }
                | DocumentElementRef::VariableNotation { .. }
                | DocumentElementRef::Term { .. } => (),
            }
        }
        if elements.len() == 1 && matches!(elements.first(), Some(QuizElement::Section { .. })) {
            let Some(QuizElement::Section { elements: es, .. }) = elements.pop() else {
                // SAFETY: match above
                unsafe { unreachable_unchecked() }
            };
            elements = es.into_vec();
        }
        quiz.elements = elements.into_boxed_slice();
        quiz.css = css.into_boxed_slice();
        quiz.title = self.title.as_ref().map(Box::to_string);
        Ok(quiz)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QuizError {
    #[error("document {0} not found")]
    MissingDocument(DocumentUri),
    #[error("html fragment for {0} not found")]
    MissingFragment(DocumentElementUri),
    #[error("solutions for problem {0} not found")]
    MissingSolutions(DocumentElementUri),
    #[error("grading note for problem {0} not found")]
    MissingGNote(DocumentElementUri),
    #[error("invalid solutions for problem {0}")]
    InvalidSolutions(DocumentElementUri),
}
