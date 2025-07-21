use ftml_uris::Id;
use smallvec::SmallVec;

use crate::narrative::{
    DocumentRange,
    elements::problems::{
        BlockFeedback, CheckedResult, FillinFeedback, FillinFeedbackKind, ProblemFeedback,
        ProblemResponse, ProblemResponseType,
    },
};

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Solutions(Box<[SolutionData]>);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum SolutionData {
    Solution {
        html: Box<str>,
        answer_class: Option<Id>,
    },
    ChoiceBlock(ChoiceBlock),
    FillInSol(FillInSol),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ChoiceBlock {
    pub multiple: bool,
    pub inline: bool,
    pub range: DocumentRange,
    pub styles: Box<[Id]>,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Choice {
    pub correct: bool,
    pub verdict: Box<str>,
    pub feedback: Box<str>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FillInSol {
    pub width: Option<f32>,
    pub opts: Vec<FillInSolOption>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum FillInSolOption {
    Exact {
        value: Box<str>,
        verdict: bool,
        feedback: Box<str>,
    },
    NumericalRange {
        from: Option<f32>,
        to: Option<f32>,
        verdict: bool,
        feedback: Box<str>,
    },
    Regex {
        regex: crate::utils::regex::Regex,
        verdict: bool,
        feedback: Box<str>,
    },
}

impl FillInSolOption {
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn from_values(kind: &str, value: &str, verdict: bool) -> Option<Self> {
        use std::str::FromStr;
        match kind {
            "exact" => Some(Self::Exact {
                value: value.to_string().into(),
                verdict,
                feedback: String::new().into(),
            }),
            "numrange" => {
                let (s, neg) = value
                    .strip_prefix('-')
                    .map_or((value, false), |s| (s, true));
                let (from, to) = if let Some((from, to)) = s.split_once('-') {
                    (from, to)
                } else {
                    ("", s)
                };
                let from = if from.contains('.') {
                    Some(f32::from_str(from).ok()?)
                } else if from.is_empty() {
                    None
                } else {
                    Some(i128::from_str(from).ok()? as _)
                };
                let from = if neg { from.map(|f| -f) } else { from };
                let to = if to.contains('.') {
                    Some(f32::from_str(to).ok()?)
                } else if to.is_empty() {
                    None
                } else {
                    Some(i128::from_str(to).ok()? as _)
                };
                Some(Self::NumericalRange {
                    from,
                    to,
                    verdict,
                    feedback: String::new().into(),
                })
            }
            "regex" => Some(Self::Regex {
                regex: crate::utils::regex::Regex::new(
                    value, //&format!("^{value}?")
                )
                .ok()?,
                verdict,
                feedback: String::new().into(),
            }),
            _ => None,
        }
    }
}

#[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
impl Solutions {
    #[cfg(feature = "serde")]
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
    #[allow(clippy::missing_const_for_fn)]
    pub fn from_solutions(solutions: Box<[SolutionData]>) -> Self {
        Self(solutions)
    }

    #[inline]
    #[must_use]
    pub fn to_solutions(&self) -> Box<[SolutionData]> {
        self.0.clone()
    }

    #[must_use]
    #[inline]
    pub fn check_response(&self, response: &ProblemResponse) -> Option<ProblemFeedback> {
        self.check(response)
    }

    #[must_use]
    #[inline]
    pub fn default_feedback(&self) -> ProblemFeedback {
        self.default()
    }
}

impl Solutions {
    #[must_use]
    pub fn default(&self) -> ProblemFeedback {
        let mut solutions = SmallVec::new();
        let mut data = SmallVec::new();
        for sol in &self.0 {
            match sol {
                SolutionData::Solution { html, .. } => solutions.push(html.clone()),
                SolutionData::ChoiceBlock(ChoiceBlock {
                    multiple: false,
                    choices,
                    ..
                }) => data.push(CheckedResult::SingleChoice {
                    selected: None,
                    choices: choices
                        .iter()
                        .map(|c| BlockFeedback {
                            is_correct: c.correct,
                            verdict_str: c.verdict.to_string(),
                            feedback: c.feedback.to_string(),
                        })
                        .collect(),
                }),
                SolutionData::ChoiceBlock(ChoiceBlock { choices, .. }) => {
                    data.push(CheckedResult::MultipleChoice {
                        selected: choices.iter().map(|_| false).collect(),
                        choices: choices
                            .iter()
                            .map(|c| BlockFeedback {
                                is_correct: c.correct,
                                verdict_str: c.verdict.to_string(),
                                feedback: c.feedback.to_string(),
                            })
                            .collect(),
                    });
                }
                SolutionData::FillInSol(f) => {
                    let mut options = SmallVec::new();
                    for o in &f.opts {
                        match o {
                            FillInSolOption::Exact {
                                value,
                                verdict,
                                feedback,
                            } => options.push(FillinFeedback {
                                is_correct: *verdict,
                                feedback: feedback.to_string(),
                                kind: FillinFeedbackKind::Exact(value.to_string()),
                            }),
                            FillInSolOption::NumericalRange {
                                from,
                                to,
                                verdict,
                                feedback,
                            } => options.push(FillinFeedback {
                                is_correct: *verdict,
                                feedback: feedback.to_string(),
                                kind: FillinFeedbackKind::NumRange {
                                    from: *from,
                                    to: *to,
                                },
                            }),
                            FillInSolOption::Regex {
                                regex,
                                verdict,
                                feedback,
                            } => options.push(FillinFeedback {
                                is_correct: *verdict,
                                feedback: feedback.to_string(),
                                kind: FillinFeedbackKind::Regex(regex.as_str().to_string()),
                            }),
                        }
                    }
                    data.push(CheckedResult::FillinSol {
                        matching: None,
                        options,
                        text: String::new(),
                    });
                }
            }
        }

        ProblemFeedback {
            correct: false,
            solutions,
            data,
            score_fraction: 0.0,
        }
    }

    #[must_use]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cast_precision_loss)]
    pub fn check(&self, response: &ProblemResponse) -> Option<ProblemFeedback> {
        //println!("Here: {self:?}\n{response:?}");
        fn next_sol<'a>(
            solutions: &mut SmallVec<Box<str>, 1>,
            datas: &mut impl Iterator<Item = &'a SolutionData>,
        ) -> Option<&'a SolutionData> {
            loop {
                match datas.next() {
                    None => return None,
                    Some(SolutionData::Solution { html, .. }) => solutions.push(html.clone()),
                    Some(c) => return Some(c),
                }
            }
        }
        let mut correct = true;
        let mut pts: f32 = 0.0;
        let mut total: f32 = 0.0;
        let mut solutions = SmallVec::new();
        let mut data = SmallVec::new();
        let mut datas = self.0.iter();

        for response in &response.responses {
            total += 1.0;
            let sol = next_sol(&mut solutions, &mut datas)?;
            match (response, sol) {
                (
                    ProblemResponseType::SingleChoice { value: selected },
                    SolutionData::ChoiceBlock(ChoiceBlock {
                        multiple: false,
                        choices,
                        ..
                    }),
                ) => data.push(CheckedResult::SingleChoice {
                    selected: *selected,
                    choices: choices
                        .iter()
                        .enumerate()
                        .map(
                            |(
                                i,
                                Choice {
                                    correct: cr,
                                    verdict,
                                    feedback,
                                },
                            )| {
                                if selected.is_some_and(|j| j as usize == i) {
                                    correct = correct && *cr;
                                    if *cr {
                                        pts += 1.0;
                                    }
                                }
                                BlockFeedback {
                                    is_correct: *cr,
                                    verdict_str: verdict.to_string(),
                                    feedback: feedback.to_string(),
                                }
                            },
                        )
                        .collect(),
                }),
                (
                    ProblemResponseType::MultipleChoice { value: selected },
                    SolutionData::ChoiceBlock(ChoiceBlock {
                        multiple: true,
                        choices,
                        ..
                    }),
                ) => {
                    if selected.len() != choices.len() {
                        return None;
                    }
                    let mut corrects = 0;
                    let mut falses = 0;
                    data.push(CheckedResult::MultipleChoice {
                        selected: selected.clone(),
                        choices: choices
                            .iter()
                            .enumerate()
                            .map(
                                |(
                                    i,
                                    Choice {
                                        correct: cr,
                                        verdict,
                                        feedback,
                                    },
                                )| {
                                    if *cr == selected[i] {
                                        corrects += 1;
                                    } else {
                                        falses += 1;
                                    }
                                    correct = correct && (selected[i] == *cr);
                                    BlockFeedback {
                                        is_correct: *cr,
                                        verdict_str: verdict.to_string(),
                                        feedback: feedback.to_string(),
                                    }
                                },
                            )
                            .collect(),
                    });
                    if selected.iter().any(|b| *b) {
                        pts += ((corrects as f32 - falses as f32) / choices.len() as f32).max(0.0);
                    }
                }
                (ProblemResponseType::Fillinsol { value: s }, SolutionData::FillInSol(f)) => {
                    let mut fill_correct = None;
                    let mut matching = None;
                    let mut options = SmallVec::new();
                    for (i, o) in f.opts.iter().enumerate() {
                        match o {
                            FillInSolOption::Exact {
                                value: string,
                                verdict,
                                feedback,
                            } => {
                                if fill_correct.is_none() && &**string == s.as_str() {
                                    if *verdict {
                                        pts += 1.0;
                                    }
                                    fill_correct = Some(*verdict);
                                    matching = Some(i);
                                }
                                options.push(FillinFeedback {
                                    is_correct: *verdict,
                                    feedback: feedback.to_string(),
                                    kind: FillinFeedbackKind::Exact(string.to_string()),
                                });
                            }
                            FillInSolOption::NumericalRange {
                                from,
                                to,
                                verdict,
                                feedback,
                            } => {
                                if fill_correct.is_none() {
                                    let num = if s.contains('.') {
                                        s.parse::<f32>().ok()
                                    } else {
                                        s.parse::<i32>().ok().map(|i| i as f32)
                                    };
                                    if let Some(f) = num {
                                        if !from.is_some_and(|v| f < v)
                                            && !to.is_some_and(|v| f > v)
                                        {
                                            if *verdict {
                                                pts += 1.0;
                                            }
                                            fill_correct = Some(*verdict);
                                            matching = Some(i);
                                        }
                                    }
                                }
                                options.push(FillinFeedback {
                                    is_correct: *verdict,
                                    feedback: feedback.to_string(),
                                    kind: FillinFeedbackKind::NumRange {
                                        from: *from,
                                        to: *to,
                                    },
                                });
                            }
                            FillInSolOption::Regex {
                                regex,
                                verdict,
                                feedback,
                            } => {
                                if fill_correct.is_none() && regex.is_match(s) {
                                    if *verdict {
                                        pts += 1.0;
                                    }
                                    fill_correct = Some(*verdict);
                                    matching = Some(i);
                                }
                                options.push(FillinFeedback {
                                    is_correct: *verdict,
                                    feedback: feedback.to_string(),
                                    kind: FillinFeedbackKind::Regex(regex.as_str().to_string()),
                                });
                            }
                        }
                    }
                    correct = correct && fill_correct.unwrap_or_default();
                    data.push(CheckedResult::FillinSol {
                        matching,
                        options,
                        text: s.to_string(),
                    });
                }
                _ => return None,
            }
        }

        if next_sol(&mut solutions, &mut datas).is_some() {
            return None;
        }

        Some(ProblemFeedback {
            correct,
            solutions,
            data,
            score_fraction: pts / total,
        })
    }
}
