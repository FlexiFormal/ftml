use crate::{
    narrative::elements::problems::{AnswerClass, CognitiveDimension},
    utils::Css,
};
use ftml_uris::{DocumentElementUri, SymbolUri};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Quiz {
    pub css: Box<[Css]>,
    pub title: Option<String>,
    pub elements: Box<[QuizElement]>,
    pub solutions: rustc_hash::FxHashMap<DocumentElementUri, Box<str>>,
    pub answer_classes: rustc_hash::FxHashMap<DocumentElementUri, Box<[AnswerClass]>>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum QuizElement {
    Section {
        title: Box<str>,
        elements: Box<[QuizElement]>,
    },
    Problem(QuizProblem),
    Paragraph {
        html: Box<str>,
    },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct QuizProblem {
    pub html: Box<str>,
    pub title_html: Option<Box<str>>,
    pub uri: DocumentElementUri,
    //pub solution:String,//Solutions,
    pub total_points: Option<f32>,
    //pub is_sub_problem:bool,
    pub preconditions: Box<[(CognitiveDimension, SymbolUri)]>,
    pub objectives: Box<[(CognitiveDimension, SymbolUri)]>,
}
