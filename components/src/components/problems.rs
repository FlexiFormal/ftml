use ftml_dom::FtmlViews;
use ftml_dom::utils::css::inject_css;
use ftml_dom::utils::local_cache::{LocalCache, SendBackend};
use ftml_js_utils::JsDisplay;
use ftml_ontology::narrative::elements::problems::{
    BlockFeedback, CheckedResult, ChoiceBlockStyle, FillinFeedback, FillinFeedbackKind,
    ProblemFeedback, ProblemResponse as OrigResponse, ProblemResponseType, Solutions,
};
use ftml_ontology::utils::SVec;
use ftml_uris::{DocumentElementUri, Id};
use leptos::either::Either::{Left, Right};
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use smallvec::SmallVec;

use crate::config::FtmlConfig;
use crate::utils::LocalCacheExt;

#[cfg(feature = "typescript")]
#[leptos::wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const PROBLEM_CONT: &str = r#"
export type ProblemContinuation = (r:ProblemResponse) => void;
"#;

#[derive(Clone)]
pub enum ProblemContinuation {
    Rs(std::sync::Arc<dyn Fn(&OrigResponse) + Send + Sync>),
    Js(SendWrapper<leptos::web_sys::js_sys::Function>),
}

pub struct ProblemOptions {
    pub on_response: Option<ProblemContinuation>,
    pub states: rustc_hash::FxHashMap<DocumentElementUri, ProblemState>, //HMap<DocumentElementUri, ProblemState>,
}

#[derive(
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    serde_lite::Serialize,
    serde_lite::Deserialize,
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "type")]
pub enum ProblemState {
    Interactive {
        current_response: Option<OrigResponse>,
        solution: Option<Solutions>,
    },
    Finished {
        current_response: Option<OrigResponse>,
    },
    Graded {
        feedback: ProblemFeedback,
    },
}

#[derive(
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    serde_lite::Serialize,
    serde_lite::Deserialize,
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ProblemStates(
    #[cfg_attr(
        feature = "typescript",
        tsify(type = "Map<DocumentElementUri,ProblemState>")
    )]
    pub rustc_hash::FxHashMap<DocumentElementUri, ProblemState>,
);

impl ftml_js_utils::conversion::FromJs for ProblemStates {
    type Error = ftml_js_utils::conversion::SerdeWasmError;
    #[inline]
    fn from_js(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        ftml_js_utils::conversion::from_value(&value)
    }
}
impl ftml_js_utils::conversion::FromWasmBindgen for ProblemContinuation {}
impl wasm_bindgen::convert::TryFromJsValue for ProblemContinuation {
    type Error = JsDisplay;
    fn try_from_js_value(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use wasm_bindgen::JsCast;
        let f = match value.dyn_into() {
            Ok(f) => f,
            Err(e) => return Err(JsDisplay(e)),
        };
        Ok(Self::Js(SendWrapper::new(f)))
    }
}

impl ProblemContinuation {
    fn apply(&self, res: &OrigResponse) {
        match self {
            Self::Rs(f) => f(res),
            Self::Js(f) => {
                #[cfg(feature = "typescript")]
                {
                    let Ok(js) = serde_wasm_bindgen::to_value(res) else {
                        return;
                    };
                    let _ = f.call1(&leptos::wasm_bindgen::JsValue::UNDEFINED, &js);
                }
            }
        }
    }
}

impl std::fmt::Debug for ProblemOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProblemOptions")
            .field("on_response", &self.on_response.is_some())
            .field("states", &self.states)
            .finish()
    }
}

impl std::fmt::Debug for ProblemContinuation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rs(_) => f.debug_struct("ProblemContinuation(Rust)").finish(),
            Self::Js(_) => f.debug_struct("ProblemContinuation(Js)").finish(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CurrentProblem {
    uri: DocumentElementUri,
    solutions: RwSignal<u8>,
    initial: Option<OrigResponse>,
    responses: RwSignal<Vec<ProblemResponse>>,
    interactive: bool,
    feedback: RwSignal<Option<ProblemFeedback>>,
}
impl CurrentProblem {
    fn to_response(uri: &DocumentElementUri, responses: &[ProblemResponse]) -> OrigResponse {
        OrigResponse {
            uri: uri.clone(),
            responses: responses
                .iter()
                .map(|r| match r {
                    ProblemResponse::MultipleChoice(_, sigs) => {
                        ProblemResponseType::MultipleChoice {
                            value: SVec(sigs.clone()),
                        }
                    }
                    ProblemResponse::SingleChoice(_, sig, _) => {
                        ProblemResponseType::SingleChoice { value: *sig }
                    }
                    ProblemResponse::Fillinsol(s) => {
                        ProblemResponseType::Fillinsol { value: s.clone() }
                    }
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum ProblemResponse {
    MultipleChoice(bool, SmallVec<bool, 8>),
    SingleChoice(bool, Option<u16>, u16),
    Fillinsol(String),
}

#[allow(clippy::too_many_arguments)]
pub fn problem<Be: SendBackend, V: IntoView>(
    uri: DocumentElementUri,
    _styles: Box<[Id]>,
    style: Memo<String>,
    class: String,
    is_subproblem: bool,
    autogradable: bool,
    points: Option<f32>,
    minutes: Option<f32>,
    children: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    inject_css("ftml-sections", include_str!("sections.css"));

    //let uri = with_context::<ForcedName, _>(|n| n.update(uri)).unwrap_or_else(|| uri.clone());
    let mut ex = CurrentProblem {
        solutions: RwSignal::new(0),
        uri,
        initial: None,
        interactive: true,
        responses: RwSignal::new(Vec::new()),
        feedback: RwSignal::new(None),
    };
    let responses = ex.responses;
    let mut is_done = with_context(|states: &Option<ProblemStates>| {
        if let Some(states) = states.as_ref() {
            match states.0.get(&ex.uri) {
                Some(ProblemState::Graded { feedback }) => {
                    ex.feedback
                        .update_untracked(|v| *v = Some(feedback.clone()));
                    return Left(true);
                }
                Some(ProblemState::Interactive {
                    current_response: Some(resp),
                    ..
                }) => ex.initial = Some(resp.clone()),
                Some(ProblemState::Finished {
                    current_response: Some(resp),
                }) => {
                    ex.initial = Some(resp.clone());
                    ex.interactive = false;
                }
                _ => (),
            }
        }
        Left(false)
    })
    .unwrap_or(Left(false));
    if matches!(is_done, Left(false)) {
        with_context(|cont: &Option<ProblemContinuation>| {
            if let Some(cont) = cont {
                is_done = Right(cont.clone());
            }
        });
    }

    let uri = ex.uri.clone();
    let uri2 = uri.clone();
    provide_context(ex);
    FtmlConfig::wrap_problem(&uri2, is_subproblem, move || {
        view! {
          //<Provider value=ForcedName::default()>
            <div class=class style=style>
              {
                let r = children();
                match is_done {
                  Left(true) => Left(r),
                  Right(f) => {
                    let _ = Effect::new(move |_| {
                      if let Some(resp) = responses.try_with(|resp|
                        CurrentProblem::to_response(&uri, resp)
                      ) {
                        f.apply(&resp);
                      }
                    });
                    Left(r)
                  }
                  Left(false) if responses.get_untracked().is_empty() =>
                    Left(r),
                  Left(false) => Right(view!{
                    {r}
                    {submit_answer::<Be>()}
                  })
                }
              }
          </div>
          //</Provider>
        }
    })
}

fn submit_answer<Be: SendBackend>() -> impl IntoView {
    use thaw::{Button, ButtonSize};
    with_context(|current: &CurrentProblem| {
        let uri = current.uri.clone();
        let responses = current.responses;
        let feedback = current.feedback;
        move || {
            if feedback.with(Option::is_none) {
                let do_solution = move |uri: &_, r: &Solutions| {
                    let resp = responses
                        .with_untracked(|responses| CurrentProblem::to_response(uri, responses));
                    //tracing::warn!("Checking response {resp:#?} against solution {r:#?}");
                    if let Some(r) = r.check(&resp) {
                        feedback.set(Some(r));
                    } else {
                        tracing::error!("Answer to Problem does not match solution");
                    }
                };
                let uri = uri.clone();
                let foract = if let Some(s) = with_context(|opt: &ProblemOptions| {
                    if let Some(ProblemState::Interactive {
                        solution: Some(sol),
                        ..
                    }) = opt.states.get(&uri)
                    {
                        Some(sol.clone())
                    } else {
                        None
                    }
                })
                .flatten()
                {
                    leptos::either::Either::Left(move || do_solution(&uri, &s))
                } else {
                    let uricl = uri.clone();
                    leptos::either::Either::Right(move || {
                        let res = LocalCache::resource::<Be, _, _>(|c| c.get_solutions(uricl));
                        Effect::new(move || {
                            res.with(|r| {
                                if let Some(Ok(r)) = r {
                                    do_solution(&uri, r);
                                }
                            });
                        });
                    })
                };
                let foract = move || match &foract {
                    leptos::either::Either::Right(act) => (act.clone())(),
                    leptos::either::Either::Left(sol) => sol(),
                };
                Some(view! {
                  <div style="margin:5px 0;"><div style="margin-left:auto;width:fit-content;">
                    <Button size=ButtonSize::Small on_click=move |_| {foract()}>"Submit Answer"</Button>
                  </div></div>
                })
            } else {
                None
            }
        }
    })
}

pub fn hint<V: IntoView + 'static>(children: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
    use crate::utils::{Header, collapsible::Collapsible};
    view! {
      <Collapsible>
        <Header slot><span style="font-style:italic;color:gray;cursor:pointer;">"Hint"</span></Header>
        {children().attr("style","border:1px solid black;")}
      </Collapsible>
    }
}

#[allow(clippy::missing_panics_doc)]
pub fn fillinsol(wd: Option<f32>) -> impl IntoView {
    use leptos::either::EitherOf3 as Either;
    use thaw::Icon;
    let Some(ex) = use_context::<CurrentProblem>() else {
        tracing::error!("choice outside of problem!");
        return None;
    };
    let Some(choice) = ex.responses.try_update_untracked(|resp| {
        let i = resp.len();
        resp.push(ProblemResponse::Fillinsol(String::new()));
        i
    }) else {
        tracing::error!("fillinsol outside of an problem!");
        return None;
    };
    let feedback = ex.feedback;
    Some(move || {
        let style = wd.map(|wd| format!("width:{wd}px;"));
        feedback.with(|v|
    if let Some(feedback) = v.as_ref() {
      let err = || {
        tracing::error!("Answer to problem does not match solution!");
        Either::C(view!(<div style="color:red;">"ERROR"</div>))
      };
      let Some(CheckedResult::FillinSol { matching, text, options }) = feedback.data.get(choice) else {return err()};
      let (correct,feedback) = if let Some(m) = matching {
        let Some(FillinFeedback{is_correct,feedback,..}) = options.get(*m) else {return err()};

        (*is_correct,Some(feedback.clone()))
      } else {(false,None)};
      let solution = if correct { None } else {
        options.iter().find_map(|f| match f{
          FillinFeedback{is_correct:true,kind:FillinFeedbackKind::Exact(s),..} => Some(s.clone()),
          _ => None
        })
      };
      let icon = if correct {
        view!(<Icon icon=icondata_ai::AiCheckCircleOutlined style="color:green;"/>)
      } else {
        view!(<Icon icon=icondata_ai::AiCloseCircleOutlined style="color:red;"/>)
      };
      Either::B(view!{
        {icon}" "
        <input type="text" style=style disabled value=text.clone()/>
        {solution.map(|s| view!(" "<pre style="color:green;display:inline;">{s.into_string()}</pre>))}
        {feedback.map(|s| view!(" "<span style="background-color:lightgray;" inner_html=s.into_string()/>))}
      })
    } else {
      let sig = create_write_slice(ex.responses,
        move |resps,val| {
          let resp = resps.get_mut(choice).expect("Signal error in problem");
          let ProblemResponse::Fillinsol(s) = resp else { panic!("Signal error in problem")};
          *s = val;
        }
      );
      let txt = if let Some(ProblemResponseType::Fillinsol{value:s}) = ex.initial.as_ref().and_then(|i| i.responses.get(choice)) {
          sig.set(s.clone());
          s.clone()
      } else {String::new()};
      let disabled = !ex.interactive;
      Either::A(view!{
        <input type="text" style=style value=txt disabled=disabled on:input:target=move |ev| {sig.set(ev.target().value());}/>
      })
    }
  )
    })
}

#[must_use]
pub fn solution<Be: SendBackend>() -> impl IntoView {
    let Some((idx, feedback)) = with_context::<CurrentProblem, _>(|problem| {
        let idx = problem.solutions.get_untracked();
        problem.solutions.update_untracked(|i| *i += 1);
        (idx, problem.feedback)
    }) else {
        tracing::error!("solution outside of problem!");
        return None;
    };
    Some(move || {
        feedback.with(|f| {
            f.as_ref().and_then(|f| {
                let Some(f) = f.solutions.get(idx as usize) else {
                    tracing::error!("No solution!");
                    return None;
                };
                Some(
                    crate::Views::<Be>::render_ftml(f.to_string(), None)
                        .attr("style", "background-color:lawngreen;"),
                )
            })
        })
    })
}

#[must_use]
pub fn gnote() -> impl IntoView {}

#[derive(Clone)]
struct CurrentChoice(usize);

pub(super) fn choice_block<V: IntoView + 'static>(
    multiple: bool,
    style: ChoiceBlockStyle,
    children: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    let inline = matches!(style, ChoiceBlockStyle::Dropdown | ChoiceBlockStyle::Inline);
    let response = if multiple {
        ProblemResponse::MultipleChoice(inline, SmallVec::new())
    } else {
        ProblemResponse::SingleChoice(inline, None, 0)
    };
    let Some(i) = with_context::<CurrentProblem, _>(|ex| {
        ex.responses.try_update_untracked(|ex| {
            let i = ex.len();
            ex.push(response);
            i
        })
    })
    .flatten() else {
        tracing::error!(
            "{} choice block outside of a problem!",
            if multiple { "multiple" } else { "single" }
        );
        return None;
    };
    provide_context(CurrentChoice(i));
    Some(children())
}

pub fn choice<V: IntoView + 'static>(
    children: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    let Some(CurrentChoice(block)) = use_context() else {
        tracing::error!("choice outside of choice block!");
        return None;
    };
    let Some(ex) = use_context::<CurrentProblem>() else {
        tracing::error!("choice outside of problem!");
        return None;
    };
    let Some((multiple, inline)) = ex
        .responses
        .try_update_untracked(|resp| {
            resp.get_mut(block).map(|l| match l {
                ProblemResponse::MultipleChoice(inline, sigs) => {
                    let idx = sigs.len();
                    sigs.push(false);
                    Some((Left(idx), *inline))
                }
                ProblemResponse::SingleChoice(inline, _, total) => {
                    let val = *total;
                    *total += 1;
                    Some((Right(val), *inline))
                }
                ProblemResponse::Fillinsol(_) => None,
            })
        })
        .flatten()
        .flatten()
    else {
        tracing::error!("choice outside of choice block!");
        return None;
    };
    let selected = ex
        .initial
        .as_ref()
        .and_then(|i| i.responses.get(block))
        .is_some_and(|init| match (init, multiple) {
            (ProblemResponseType::MultipleChoice { value }, Left(idx)) => {
                value.get(idx).copied().unwrap_or_default()
            }
            (ProblemResponseType::SingleChoice { value }, Right(val)) => {
                value.is_some_and(|v| v == val)
            }
            _ => false,
        });
    let disabled = !ex.interactive;
    Some(match multiple {
        Left(idx) => Left(multiple_choice(
            idx,
            block,
            inline,
            selected,
            disabled,
            ex.responses,
            ex.feedback,
            children,
        )),
        Right(idx) => Right(single_choice(
            idx,
            block,
            inline,
            selected,
            disabled,
            ex.responses,
            ex.uri,
            ex.feedback,
            children,
        )),
    })
}

#[allow(clippy::too_many_arguments)]
fn multiple_choice<V: IntoView + 'static>(
    idx: usize,
    block: usize,
    inline: bool,
    orig_selected: bool,
    disabled: bool,
    responses: RwSignal<Vec<ProblemResponse>>,
    feedback: RwSignal<Option<ProblemFeedback>>,
    children: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    use leptos::either::{
        Either::Left,
        Either::Right,
        EitherOf3::{A, B, C},
    };
    use thaw::Icon;
    let bx = move || {
        feedback.with(|v| if let Some(feedback) = v.as_ref() {
            let err = || {
            tracing::error!("Answer to problem does not match solution:");
            C(view!(<div style="color:red;">"ERROR"</div>))
            };
            let Some(CheckedResult::MultipleChoice{selected,choices}) = feedback.data.get(block) else {return err()};
            let Some(selected) = selected.get(idx).copied() else { return err() };
            let Some(BlockFeedback{is_correct,..}) = choices.get(idx) else { return err() };
            let icon = if selected == *is_correct {
            view!(<Icon icon=icondata_ai::AiCheckCircleOutlined style="color:green;"/>)
            } else {
            view!(<Icon icon=icondata_ai::AiCloseCircleOutlined style="color:red;"/>)
            };
            let bx = if selected {
            Left(view!({icon}<input type="checkbox" checked disabled/>))
            } else {
            Right(view!({icon}<input type="checkbox" disabled/>))
            };
            A(bx)
        } else {
            let sig = create_write_slice(responses,
            move |resp,val| {
                let resp = resp.get_mut(block).expect("Signal error in problem");
                let ProblemResponse::MultipleChoice(_,v) = resp else { panic!("Signal error in problem")};
                v[idx] = val;
            }
            );
            sig.set(orig_selected);
            let rf = NodeRef::<leptos::html::Input>::new();
            let on_change = move |_| {
            let Some(ip) = rf.get_untracked() else {return};
            let nv = ip.checked();
            sig.set(nv);
            };
            B(view!{<input node_ref=rf type="checkbox" on:change=on_change checked=orig_selected disabled=disabled/>})
        })
    };
    let post = move || {
        feedback.with(|v| v.as_ref().map(|feedback| {
            let err = || {
            tracing::error!("Answer to problem does not match solution:");
            Right(view!(<div style="color:red;">"ERROR"</div>))
            };
            let Some(CheckedResult::MultipleChoice{choices,..}) = feedback.data.get(block) else {return err()};
            let Some(BlockFeedback{is_correct,verdict_str,feedback}) = choices.get(idx) else { return err() };
            let verdict = if *is_correct {
            Left(view!(<span style="color:green;" inner_html=verdict_str.clone().into_string()/>))
            } else {
            Right(view!(<span style="color:red;" inner_html=verdict_str.clone().into_string()/>))
            };
            Left(view!{
            " "{verdict}" "
            {if inline {None} else {Some(view!(<br/>))}}
            <span style="background-color:lightgray;" inner_html=feedback.clone().into_string()/>
            })
        }))
    };
    view! {<div style="display:inline;margin-right:5px;">
        {bx}
        {children()}
        {post}
    </div>}
}

#[allow(clippy::too_many_arguments)]
fn single_choice<V: IntoView + 'static>(
    idx: u16,
    block: usize,
    inline: bool,
    orig_selected: bool,
    disabled: bool,
    responses: RwSignal<Vec<ProblemResponse>>,
    uri: DocumentElementUri,
    feedback: RwSignal<Option<ProblemFeedback>>,
    children: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    use leptos::either::{
        Either::Left,
        Either::Right,
        EitherOf3::{A, B, C},
    };
    use thaw::Icon;
    let bx = move || {
        feedback.with(|v| if let Some(feedback) = v.as_ref() {
            let err = || {
              tracing::error!("Answer to problem does not match solution!");
              C(view!(<div style="color:red;">"ERROR"</div>))
            };
            let Some(CheckedResult::SingleChoice{selected,choices}) = feedback.data.get(block) else {return err()};
            let Some(BlockFeedback{is_correct,..}) = choices.get(idx as usize) else { return err() };
            let icon = if selected.is_some_and(|s| s ==  idx) && *is_correct {
              Some(Left(view!(<Icon icon=icondata_ai::AiCheckCircleOutlined style="color:green;"/>)))
            } else if selected.is_some_and(|s| s ==  idx) {
              Some(Right(view!(<Icon icon=icondata_ai::AiCloseCircleOutlined style="color:red;"/>)))
            } else {None};
            let bx = if selected.is_some_and(|s| s ==  idx) {
              Left(view!({icon}<input type="radio" checked disabled/>))
            } else {
              Right(view!({icon}<input type="radio" disabled/>))
            };
            A(bx)
        } else {
          let name = format!("{uri}_{block}");
          let sig = create_write_slice(responses,
            move |resp,()| {
              let resp = resp.get_mut(block).expect("Signal error in problem");
              let ProblemResponse::SingleChoice(_,i,_) = resp else { panic!("Signal error in problem")};
              *i = Some(idx);
            }
          );
          if orig_selected {sig.set(());}
          let rf = NodeRef::<leptos::html::Input>::new();
          let on_change = move |_| {
            let Some(ip) = rf.get_untracked() else {return};
            if ip.checked() { sig.set(()); }
          };
          B(view!{
            <input node_ref=rf type="radio" name=name on:change=on_change checked=orig_selected disabled=disabled/>
          })
        })
    };
    let post = move || {
        feedback.with(|v| v.as_ref().map(|feedback| {
            let err = || {
              tracing::error!("Answer to problem does not match solution!");
              Right(view!(<div style="color:red;">"ERROR"</div>))
            };
            let Some(CheckedResult::SingleChoice{choices,..}) = feedback.data.get(block) else {return err()};
            let Some(BlockFeedback{is_correct,verdict_str,feedback}) = choices.get(idx as usize) else { return err() };
            let verdict = if *is_correct {
              Left(view!(<span style="color:green;" inner_html=verdict_str.clone().into_string()/>))
            } else {
              Right(view!(<span style="color:red;" inner_html=verdict_str.clone().into_string()/>))
            };
            Left(view!{" "{verdict}" "
              {if inline {None} else {Some(view!(<br/>))}}
              <span style="background-color:lightgray;" inner_html=feedback.clone().into_string()/>
            })
        }))
    };
    view! {<div style="display:inline;margin-right:5px;">
        {bx}
        {children()}
        {post}
    </div>}
}

/*
#[allow(clippy::needless_pass_by_value)]
#[allow(unused_variables)]
pub(super) fn solution(
    _skip: usize,
    _elements: FTMLElements,
    orig: OriginalNode,
    _id: Option<Box<str>>,
) -> impl IntoView {
    let Some((solutions, feedback)) =
        with_context::<CurrentProblem, _>(|e| (e.solutions, e.feedback))
    else {
        tracing::error!("solution outside of problem!");
        return None;
    };
    let idx = solutions.get_untracked();
    solutions.update_untracked(|i| *i += 1);
    #[cfg(any(feature = "csr", feature = "hydrate"))]
    {
        if orig.child_element_count() == 0 {
            tracing::debug!("Solution removed!");
        } else {
            tracing::debug!("Solution exists!");
        }
        Some(move || {
            feedback.with(|f| {
                f.as_ref().and_then(|f| {
                    let Some(f) = f.solutions.get(idx as usize) else {
                        tracing::error!("No solution!");
                        return None;
                    };
                    Some(view! {
                      <div style="background-color:lawngreen;">
                        <span inner_html=f.to_string()/>
                      </div>
                    })
                })
            })
        })
        // TODO
    }
    #[cfg(not(any(feature = "csr", feature = "hydrate")))]
    {
        Some(())
    }
}


#[allow(clippy::needless_pass_by_value)]
#[allow(unused_variables)]
pub(super) fn gnote(_skip: usize, _elements: FTMLElements, orig: OriginalNode) -> impl IntoView {
    #[cfg(any(feature = "csr", feature = "hydrate"))]
    {
        if orig.child_element_count() == 0 {
            tracing::debug!("Grading note removed!");
        } else {
            tracing::debug!("Grading note exists!");
        }
        // TODO
    }
    #[cfg(not(any(feature = "csr", feature = "hydrate")))]
    {
        ()
    }
}

/*
  let feedback = ex.feedback;
  move || {
    if feedback.with(|f| f.is_some()) {}
    else {

    }
  }
*/


*/
