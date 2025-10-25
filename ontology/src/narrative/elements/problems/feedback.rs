use crate::utils::SVec;
use ftml_uris::DocumentElementUri;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
pub struct ProblemFeedback {
    pub correct: bool,
    #[cfg_attr(feature = "typescript", wasm_bindgen(skip))]
    pub solutions: SVec<Box<str>, 1>,
    #[cfg_attr(feature = "typescript", wasm_bindgen(skip))]
    pub data: SVec<CheckedResult, 4>,
    pub score_fraction: f32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ProblemFeedbackJson {
    pub correct: bool,
    #[cfg_attr(feature = "typescript", tsify(type = "string[]"))]
    pub solutions: SVec<Box<str>, 1>,
    #[cfg_attr(feature = "typescript", tsify(type = "CheckedResult[]"))]
    pub data: SVec<CheckedResult, 4>,
    pub score_fraction: f32,
}

#[cfg(any(feature = "serde", feature = "serde-lite"))]
#[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
impl ProblemFeedback {
    #[must_use]
    pub fn from_json(
        ProblemFeedbackJson {
            correct,
            solutions,
            data,
            score_fraction,
        }: ProblemFeedbackJson,
    ) -> Self {
        Self {
            correct,
            solutions,
            data,
            score_fraction,
        }
    }

    #[must_use]
    pub fn to_json(&self) -> ProblemFeedbackJson {
        let Self {
            correct,
            solutions,
            data,
            score_fraction,
        } = self.clone();
        ProblemFeedbackJson {
            correct,
            solutions,
            data,
            score_fraction,
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
impl ProblemFeedback {
    #[must_use]
    pub fn from_jstring(s: &str) -> Option<Self> {
        use crate::utils::Hexable;
        Self::from_hex(s).ok()
    }

    #[must_use]
    pub fn to_jstring(&self) -> Option<String> {
        use crate::utils::Hexable;
        self.as_hex_string().ok()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct BlockFeedback {
    pub is_correct: bool,
    pub verdict_str: Box<str>,
    pub feedback: Box<str>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FillinFeedback {
    pub is_correct: bool,
    pub feedback: Box<str>,
    pub kind: FillinFeedbackKind,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum FillinFeedbackKind {
    Exact(Box<str>),
    NumRange { from: Option<f32>, to: Option<f32> },
    Regex(Box<str>),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(tag = "type"))]
pub enum CheckedResult {
    SingleChoice {
        selected: Option<u16>,
        #[cfg_attr(feature = "typescript", tsify(type = "BlockFeedback[]"))]
        choices: SVec<BlockFeedback, 4>,
    },
    MultipleChoice {
        #[cfg_attr(feature = "typescript", tsify(type = "boolean[]"))]
        selected: SVec<bool, 8>,
        #[cfg_attr(feature = "typescript", tsify(type = "BlockFeedback[]"))]
        choices: SVec<BlockFeedback, 4>,
    },
    FillinSol {
        matching: Option<usize>,
        text: String,
        #[cfg_attr(feature = "typescript", tsify(type = "FillinFeedback[]"))]
        options: SVec<FillinFeedback, 4>,
    },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ProblemResponse {
    pub uri: DocumentElementUri,
    #[cfg_attr(feature = "typescript", tsify(type = "ProblemResponseType[]"))]
    pub responses: SVec<ProblemResponseType, 4>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(tag = "type"))]
/// Either a list of booleans (multiple choice), a single integer (single choice),
/// or a string (fill-in-the-gaps)
pub enum ProblemResponseType {
    MultipleChoice {
        #[cfg_attr(feature = "typescript", tsify(type = "boolean[]"))]
        value: SVec<bool, 8>,
    },
    SingleChoice {
        value: Option<u16>,
    },
    Fillinsol {
        value: String,
    },
}
