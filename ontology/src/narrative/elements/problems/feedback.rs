use ftml_uris::DocumentElementUri;
use smallvec::SmallVec;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
pub struct ProblemFeedback {
    pub correct: bool,
    #[cfg_attr(feature = "typescript", wasm_bindgen(skip))]
    pub solutions: SmallVec<Box<str>, 1>,
    #[cfg_attr(feature = "typescript", wasm_bindgen(skip))]
    pub data: SmallVec<CheckedResult, 4>,
    pub score_fraction: f32,
}

#[cfg(feature = "typescript")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(from_wasm_abi, into_wasm_abi)]
pub struct ProblemFeedbackJson {
    pub correct: bool,
    #[tsify(type = "string[]")]
    pub solutions: SmallVec<Box<str>, 1>,
    #[tsify(type = "CheckedResult[]")]
    pub data: SmallVec<CheckedResult, 4>,
    pub score_fraction: f32,
}

#[cfg(feature = "typescript")]
#[wasm_bindgen::prelude::wasm_bindgen]
impl ProblemFeedback {
    #[must_use]
    pub fn from_jstring(s: &str) -> Option<Self> {
        use crate::utils::Hexable;
        Self::from_hex(s).ok()
    }

    #[cfg(feature = "serde")]
    #[must_use]
    pub fn to_jstring(&self) -> Option<String> {
        use crate::utils::Hexable;
        self.as_hex_string().ok()
    }

    #[inline]
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

    #[inline]
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct BlockFeedback {
    pub is_correct: bool,
    pub verdict_str: String,
    pub feedback: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FillinFeedback {
    pub is_correct: bool,
    pub feedback: String,
    pub kind: FillinFeedbackKind,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum FillinFeedbackKind {
    Exact(String),
    NumRange { from: Option<f32>, to: Option<f32> },
    Regex(String),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum CheckedResult {
    SingleChoice {
        selected: Option<u16>,
        #[cfg_attr(feature = "typescript", tsify(type = "BlockFeedback[]"))]
        choices: SmallVec<BlockFeedback, 4>,
    },
    MultipleChoice {
        #[cfg_attr(feature = "typescript", tsify(type = "boolean[]"))]
        selected: SmallVec<bool, 8>,
        #[cfg_attr(feature = "typescript", tsify(type = "BlockFeedback[]"))]
        choices: SmallVec<BlockFeedback, 4>,
    },
    FillinSol {
        matching: Option<usize>,
        text: String,
        #[cfg_attr(feature = "typescript", tsify(type = "FillinFeedback[]"))]
        options: SmallVec<FillinFeedback, 4>,
    },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ProblemResponse {
    pub uri: DocumentElementUri,
    #[cfg_attr(feature = "typescript", tsify(type = "ProblemResponseType[]"))]
    pub responses: SmallVec<ProblemResponseType, 4>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
/// Either a list of booleans (multiple choice), a single integer (single choice),
/// or a string (fill-in-the-gaps)
pub enum ProblemResponseType {
    MultipleChoice {
        #[cfg_attr(feature = "typescript", tsify(type = "boolean[]"))]
        value: SmallVec<bool, 8>,
    },
    SingleChoice {
        value: Option<u16>,
    },
    Fillinsol {
        value: String,
    },
}
