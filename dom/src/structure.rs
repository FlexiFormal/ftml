use crate::{
    DocumentState,
    counters::LogicalLevel,
    document::CurrentUri,
    extractor::DomExtractor,
    utils::{
        actions::{OneShot, SetOneShotDone},
        css::CssExt,
        local_cache::{LocalCache, SendBackend},
    },
};
use ftml_ontology::{
    narrative::{
        documents::TocElem,
        elements::{SectionLevel, paragraphs::ParagraphKind},
    },
    utils::{IterCont, TreeIter},
};
use ftml_uris::{DocumentElementUri, DocumentUri, Id, IsNarrativeUri, NamedUri, NarrativeUri};
use leptos::prelude::*;
use smallvec::SmallVec;
use std::{collections::hash_map::Entry, hint::unreachable_unchecked, ops::ControlFlow};

#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum TocSource {
    None,
    #[default]
    Extract,
    Ready(Vec<TocElem>),
    Get,
}
impl ftml_js_utils::conversion::FromWasmBindgen for TocSource {}

impl leptos::wasm_bindgen::convert::TryFromJsValue for TocSource {
    fn try_from_js_value(
        value: leptos::wasm_bindgen::JsValue,
    ) -> Result<Self, leptos::wasm_bindgen::JsValue> {
        serde_wasm_bindgen::from_value(value.clone()).map_err(|_| value)
    }
    fn try_from_js_value_ref(value: &leptos::wasm_bindgen::JsValue) -> Option<Self> {
        serde_wasm_bindgen::from_value(value.clone()).ok()
    }
}

type CounterMap = std::collections::HashMap<
    Id,
    RwSignal<(
        Option<(CurrentCounter, SectionLevel)>,
        Option<(CurrentCounter, SectionLevel)>,
    )>,
    rustc_hash::FxBuildHasher,
>;

#[derive(Clone, Copy, Debug)]
pub struct DocumentStructure {
    pub(crate) toc: RwSignal<Vec<TocElem>>,
    top_level: RwSignal<SectionLevel>,
    document_titles: RwSignal<
        std::collections::HashMap<DocumentUri, RwSignal<Box<str>>, rustc_hash::FxBuildHasher>,
    >,
    counters: RwSignal<CounterMap>,
    ids: RwSignal<rustc_hash::FxHashMap<Id, SectionOrInputref>>,
    redo: RwSignal<Option<String>>,
    initialized: RwSignal<bool>,
    para_counters: RwSignal<Vec<(Id, SmartCounter<u16>)>>,
    resets: RwSignal<Vec<(SectionLevel, Vec<Id>)>>,
    for_paras: RwSignal<Vec<ParaStyle>>,
    slides: SmartCounter<u32>,
}

struct ParaData {
    initialized: bool,
    resets: Vec<(SectionLevel, Vec<Id>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParaStyle {
    kind: ParagraphKind,
    name: Option<Id>,
    counter: Option<Id>,
}

impl DocumentStructure {
    pub fn display_counter(id: &str) -> impl IntoView + use<> {
        let id: Id = id.parse().ok()?;
        let sig = use_context::<Self>().map(|slf| {
            slf.counters.update_untracked(|ctrs| match ctrs.entry(id) {
                Entry::Occupied(e) => *e.get(),
                Entry::Vacant(v) => {
                    let new = RwSignal::new((None, None));
                    *v.insert(new)
                }
            })
        })?;
        Some(move || sig.get().0.map(|(v, at)| v.get().into_view(at)))
    }
    pub fn with_toc<R>(then: impl FnOnce(&[TocElem]) -> R) -> Option<R> {
        use_context::<Self>().map(|slf| slf.toc.with(|v| then(v)))
    }
    pub fn set<Be: SendBackend>() {
        Self::set_toc::<Be>(use_context::<TocSource>().unwrap_or_default());
    }
    pub(crate) fn set_empty() {
        provide_context(Self {
            toc: RwSignal::new(Vec::new()),
            document_titles: RwSignal::new(std::collections::HashMap::default()),
            top_level: RwSignal::new(SectionLevel::Part),
            counters: RwSignal::new(std::collections::HashMap::default()),
            ids: RwSignal::new(std::collections::HashMap::default()),
            redo: RwSignal::new(None),
            initialized: RwSignal::new(false),
            para_counters: RwSignal::new(Vec::new()),
            resets: RwSignal::new(Vec::new()),
            for_paras: RwSignal::new(Vec::new()),
            slides: SmartCounter::default(),
        });
        provide_context(RwSignal::new(CurrentCounter::Fixed(
            SectionCounter::default(),
        )));
        provide_context(CurrentId(None));
        provide_context(LogicalLevel::None);
    }

    pub fn set_toc<Be: SendBackend>(toc: TocSource) {
        let document_titles = RwSignal::new(std::collections::HashMap::default());
        let counters = RwSignal::new(std::collections::HashMap::default());
        let ids = RwSignal::new(std::collections::HashMap::default());
        let toc = match toc {
            TocSource::Ready(v) => {
                Self::insert_toc(&v, document_titles, counters);
                RwSignal::new(v)
            }
            TocSource::Extract | TocSource::None => RwSignal::new(Vec::new()),
            TocSource::Get => {
                let sig = RwSignal::new(Vec::new());
                // hack to make sure this happens client side only
                let csr = RwSignal::new(false);
                let _ = Effect::new(move || {
                    csr.set(true);
                });
                let uri = DocumentState::document_uri();
                let mut go = Some(move || {
                    let maybe_erred = LocalCache::resource::<Be, _, _>(|b| b.get_toc(uri));
                    Effect::new(move || {
                        maybe_erred.track();
                        if let Some(Ok((c, v))) = maybe_erred.update_untracked(Option::take) {
                            for c in c {
                                c.inject();
                            }
                            Self::insert_toc(&v, document_titles, counters);
                            sig.set(v.into_vec());
                        }
                    });
                });
                Effect::new(move || {
                    if csr.get()
                        && let Some(f) = go.take()
                    {
                        f();
                    }
                });
                sig
            }
        };
        let top_level = RwSignal::new(SectionLevel::Part);
        provide_context(Self {
            toc,
            top_level,
            document_titles,
            counters,
            ids, //current: RwSignal::new(CurrentCounter::Fixed(SectionCounter::default())),
            redo: RwSignal::new(None),
            initialized: RwSignal::new(false),
            para_counters: RwSignal::new(Vec::new()),
            resets: RwSignal::new(Vec::new()),
            for_paras: RwSignal::new(Vec::new()),
            slides: SmartCounter::default(),
        });
        provide_context(RwSignal::new(CurrentCounter::Fixed(
            SectionCounter::default(),
        )));
        provide_context(CurrentId(None));
        provide_context(LogicalLevel::None);
    }

    fn init_paras(&self) {
        if self.initialized.get_untracked() {
            return;
        }

        self.initialized.update_untracked(|b| *b = true);
        let mut para_counters = Vec::new();
        let mut resets = Vec::<(SectionLevel, Vec<Id>)>::new();
        let mut for_paras = Vec::new();
        let styles = expect_context::<RwSignal<DomExtractor>>();
        styles.with_untracked(|e| {
            let ctrs = &e.state.counters;
            let styles = &e.state.styles;
            for c in ctrs {
                tracing::trace!("Doing {c:?}");
                para_counters.push((c.name.clone(), SmartCounter::default()));
                if let Some(p) = c.parent {
                    if let Some(res) = resets
                        .iter_mut()
                        .find_map(|(e, v)| if *e == p { Some(v) } else { None })
                    {
                        if !res.contains(&c.name) {
                            res.push(c.name.clone());
                        }
                    } else {
                        resets.push((p, vec![c.name.clone()]));
                    }
                }
            }
            for stl in styles {
                for_paras.push(ParaStyle {
                    kind: stl.kind,
                    name: stl.name.clone(),
                    counter: stl.counter.clone(),
                });
            }
        });
        self.para_counters.update_untracked(|p| *p = para_counters);
        self.resets.update_untracked(|p| *p = resets);
        self.for_paras.update_untracked(|p| *p = for_paras);
    }

    pub(crate) fn get_para(
        &self,
        kind: ParagraphKind,
        styles: &[Id],
    ) -> (Memo<String>, Option<String>) {
        self.init_paras();
        let cnt = self
            .for_paras
            .with_untracked(|all_styles| Self::get_counter(all_styles, kind, styles));
        let memo = cnt.map_or_else(
            || Memo::new(|_| String::new()),
            |cntname| {
                let cnt = self.para_counters.with_untracked(|cntrs| {
                    *cntrs
                        .iter()
                        .find(|(a, _)| *a == cntname)
                        .map(|(_, r)| r)
                        .expect("counter does not exist; this is a bug")
                });
                cnt.inc_memo(move |i| format!("counter-set:ftml-{cntname} {i};"))
            },
        );
        let prefix = match kind {
            ParagraphKind::Assertion => Some("ftml-assertion"),
            ParagraphKind::Definition => Some("ftml-definition"),
            ParagraphKind::Example => Some("ftml-example"),
            ParagraphKind::Paragraph => Some("ftml-paragraph"),
            _ => None,
        };
        let cls = prefix.map(|p| {
            let mut s = String::new();
            s.push_str(p);
            for style in styles {
                s.push(' ');
                s.push_str(p);
                s.push('-');
                s.push_str(style.as_ref());
            }
            s
        });
        provide_context(LogicalLevel::Paragraph);
        (memo, cls)
    }

    pub(crate) fn get_problem(&self, styles: &[Id]) -> (Memo<String>, String) {
        self.init_paras();
        provide_context(LogicalLevel::Paragraph);
        let cls = {
            let mut s = "ftml-problem".to_string();
            for style in styles {
                s.push(' ');
                s.push_str("ftml-problem-");
                s.push_str(style.as_ref());
            }
            s
        };
        (Memo::new(|_| String::new()), cls)
    }

    pub fn get_slide() -> Memo<u32> {
        let slf: Self = expect_context();
        slf.init_paras();
        slf.slides.memo(|i| i)
    }

    pub(crate) fn slide_inc() {
        let slf: Self = expect_context();
        slf.init_paras();
        slf.slides.inc();
        provide_context(LogicalLevel::BeamerSlide);
    }

    fn get_counter(all: &[ParaStyle], kind: ParagraphKind, styles: &[Id]) -> Option<Id> {
        styles
            .iter()
            .rev()
            .find_map(|s| {
                all.iter().find_map(
                    |ParaStyle {
                         kind: k,
                         name,
                         counter,
                     }| {
                        if *k == kind && name.as_ref().is_some_and(|n| *n == *s) {
                            Some(counter.as_ref())
                        } else {
                            None
                        }
                    },
                )
            })
            .unwrap_or_else(|| {
                all.iter()
                    .find_map(
                        |ParaStyle {
                             kind: k,
                             name,
                             counter,
                         }| {
                            if name.is_none() && *k == kind {
                                Some(counter.as_ref())
                            } else {
                                None
                            }
                        },
                    )
                    .flatten()
            })
            .cloned()
    }

    #[allow(clippy::too_many_lines)]
    fn insert_toc(
        toc: &[TocElem],
        document_titles: RwSignal<
            std::collections::HashMap<DocumentUri, RwSignal<Box<str>>, rustc_hash::FxBuildHasher>,
        >,
        counters: RwSignal<CounterMap>,
    ) {
        struct State {
            sections: SectionCounter,
            inputs: Vec<(Option<SectionCounter>, Option<SectionCounter>)>,
        }
        document_titles.update_untracked(|document_titles| {
            toc.dfs_with_state(
                SectionLevel::Part,
                State {
                    sections: SectionCounter::default(),
                    inputs: Vec::new(),
                },
                |e, current, state| {
                    let next = match e {
                        TocElem::SkippedSection { .. } => current.inc(),
                        TocElem::Section { uri, id, .. } => {
                            state.sections = state.sections + *current;
                            if let Some((start, end)) = state.inputs.last_mut() {
                                if start.is_none() {
                                    *start = Some(state.sections);
                                } else {
                                    *end = Some(state.sections);
                                }
                            }
                            if let Ok(id) = id.parse::<Id>() {
                                counters.update_untracked(|counters| {
                                    if let Some(ctr) = counters.get_mut(&id) {
                                        ctr.update(|(first, _)| {
                                            *first = Some((
                                                CurrentCounter::Fixed(state.sections),
                                                *current,
                                            ));
                                        });
                                    } else {
                                        counters.insert(
                                            id,
                                            RwSignal::new((
                                                Some((
                                                    CurrentCounter::Fixed(state.sections),
                                                    *current,
                                                )),
                                                None,
                                            )),
                                        );
                                    }
                                });
                            }
                            current.inc()
                        }
                        TocElem::Inputref { uri, title, id, .. } => {
                            state.inputs.push((None, None));

                            if let Some(title) = title {
                                if let Some(dt) = document_titles.get(uri) {
                                    dt.update(|ttl| *ttl = title.clone());
                                } else {
                                    document_titles
                                        .insert(uri.clone(), RwSignal::new(title.clone()));
                                }
                            }
                            *current
                        }
                        TocElem::Paragraph { .. } | TocElem::Slide => *current,
                    };
                    IterCont::Recurse(next)
                },
                |e, lvl, state| {
                    match e {
                        TocElem::Inputref { id, .. } => {
                            let mut ctrs = state.inputs.pop().expect("bug");
                            if let Some((start, end)) = state.inputs.last_mut()
                                && let Some(new_end) = ctrs.1.or(ctrs.0)
                            {
                                if start.is_none() {
                                    *start = Some(new_end);
                                } else {
                                    *end = Some(new_end);
                                }
                            }
                            if ctrs
                                .1
                                .is_some_and(|end| ctrs.0.is_some_and(|start| end.sub_of(start)))
                            {
                                ctrs.1 = None;
                            }

                            if let Ok(id) = id.parse::<Id>() {
                                counters.update_untracked(|counters| {
                                    if let Some(ctr) = counters.get_mut(&id) {
                                        ctr.update(|ctr| {
                                            *ctr = (
                                                ctrs.0.map(|c| (CurrentCounter::Fixed(c), lvl)),
                                                ctrs.1.map(|c| (CurrentCounter::Fixed(c), lvl)),
                                            );
                                        });
                                    } else {
                                        counters.insert(
                                            id,
                                            RwSignal::new((
                                                ctrs.0.map(|c| (CurrentCounter::Fixed(c), lvl)),
                                                ctrs.1.map(|c| (CurrentCounter::Fixed(c), lvl)),
                                            )),
                                        );
                                    }
                                });
                            }
                        }
                        _ => (),
                    }
                    ControlFlow::<()>::Continue(())
                },
            );
        });
        Self::retry();
    }

    pub(crate) fn set_max_level(lvl: SectionLevel) {
        if !DocumentState::in_inputref() {
            with_context::<Self, _>(|slf| {
                slf.top_level.set(lvl);
            });
        }
    }

    pub fn in_inputref() -> bool {
        with_context::<InInputref, _>(|ipr| ipr.inner.is_some()).is_some_and(|b| b)
    }

    pub fn get_title(uri: DocumentUri) -> RwSignal<Box<str>> {
        expect_context::<Self>().get_title_i(uri)
    }

    fn get_title_i(&self, uri: DocumentUri) -> RwSignal<Box<str>> {
        self.document_titles
            .update_untracked(|dts| match dts.entry(uri) {
                std::collections::hash_map::Entry::Occupied(e) => *e.get(),
                std::collections::hash_map::Entry::Vacant(e) => {
                    let name = e.key().document_name().to_string().into_boxed_str();
                    *e.insert(RwSignal::new(name))
                }
            })
    }

    pub(crate) fn new_inputref(uri: DocumentElementUri, target: DocumentUri) -> Inputref {
        let id = Self::new_id(uri.name().last());
        let (os, done) = OneShot::new();
        let extract =
            with_context::<TocSource, _>(|s| matches!(s, TocSource::Extract)).is_some_and(|b| b);
        let current = use_context::<RwSignal<CurrentCounter>>()
            .unwrap_or_else(|| RwSignal::new(CurrentCounter::Fixed(SectionCounter::default())));
        let curr = current.get_untracked();
        let slf = expect_context::<Self>();
        let (ipr, title) = {
            let maybe_new = RwSignal::new((None, None));
            let start_end = slf.counters.update_untracked(|ctrs| {
                ctrs.get(&id).copied().unwrap_or_else(|| {
                    ctrs.insert(id.clone(), maybe_new);
                    maybe_new
                })
            });
            let title = slf.get_title_i(target.clone());

            if extract {
                slf.toc.update(|toc| {
                    let Some(ch) = Self::get_toc_at(toc, id.as_ref()) else {
                        return;
                    };
                    ch.push(TocElem::Inputref {
                        uri: target.clone(),
                        title: None,
                        id: id.to_string(),
                        children: Vec::new(),
                    });
                });
            }
            (start_end, title)
        };

        provide_context(RwSignal::new(curr));
        slf.ids
            .update_untracked(|ids| ids.insert(id.clone(), SectionOrInputref::Inputref(os)));

        let parent = use_context::<InInputref>().map(Box::new);
        provide_context(InInputref {
            inner: Some(ipr),
            parent,
        });
        Inputref {
            uri,
            target,
            id,
            replace: os,
            done,
            title_counter: ipr,
            title,
        }
    }

    pub(crate) fn skip_section() {
        let slf = expect_context::<Self>();
        // TODO: get_toc_at doesn't know about skipped sections

        let extract =
            with_context::<TocSource, _>(|s| matches!(s, TocSource::Extract)).is_some_and(|b| b);
        if extract {
            let cid = expect_context::<CurrentId>();
            slf.toc.update(|toc| {
                if let Some(ch) = (match cid.0 {
                    Some(id) => Self::get_toc_at(toc, &format!("{id}/foo")),
                    None => Self::get_toc_at(toc, "foo"),
                }) {
                    ch.push(TocElem::SkippedSection {
                        children: Vec::new(),
                    });
                }
            });
        }
        let current_level = expect_context::<LogicalLevel>();
        let new_level = {
            if let LogicalLevel::Section(s) = current_level {
                LogicalLevel::Section(s.inc())
            } else if current_level == LogicalLevel::None {
                LogicalLevel::Section(SectionLevel::Part)
            } else {
                current_level
            }
        };
        provide_context(new_level);
    }

    pub(crate) fn insert_section_title(get: impl FnOnce() -> String) -> &'static str {
        let lvl = use_context::<LogicalLevel>().unwrap_or(LogicalLevel::None);
        let cls = lvl.title_class();
        //tracing::warn!("Section title at {lvl:?}");
        if !with_context::<TocSource, _>(|s| matches!(s, TocSource::Extract)).is_some_and(|b| b) {
            return cls;
        }
        let Some(slf) = use_context::<Self>() else {
            return cls;
        };
        let Some(CurrentUri(NarrativeUri::Element(uri))) = use_context::<CurrentUri>() else {
            return cls;
        };
        slf.toc.update(|toc| {
            if let Some(e) = Self::toc_find_mut(
                toc,
                |e| matches!(e,TocElem::Section { uri:u, .. } if *u == uri),
            ) {
                let TocElem::Section { title: t, .. } = e else {
                    // SAFETY: match above
                    unsafe { unreachable_unchecked() }
                };
                *t = Some(get().into_boxed_str());
            }
        });
        cls
    }

    pub(crate) fn new_section(uri: DocumentElementUri) -> SectionInfo {
        let id = Self::new_id(uri.name().last());
        provide_context(CurrentUri(uri.clone().into()));
        // self.init_paras()
        let current_level = expect_context::<LogicalLevel>();
        let slf = expect_context::<Self>();
        slf.ids
            .update_untracked(|ids| ids.insert(id.clone(), SectionOrInputref::Section));
        let new_level = {
            if let LogicalLevel::Section(s) = current_level {
                LogicalLevel::Section(
                    //((slf.top_level.get_untracked() as u8) + s as u8)
                    //    .try_into()
                    //    .unwrap_or(SectionLevel::Subparagraph)
                    s.inc(),
                )
            } else if current_level == LogicalLevel::None {
                LogicalLevel::Section(slf.top_level.get_untracked())
            } else {
                current_level
            }
        };
        //tracing::warn!("New section at {new_level:?}");
        provide_context(new_level);
        let extract =
            with_context::<TocSource, _>(|s| matches!(s, TocSource::Extract)).is_some_and(|b| b);

        let sect_level = match new_level {
            LogicalLevel::None => slf.top_level.get_untracked(),
            LogicalLevel::Section(s) => s,
            _ => SectionLevel::Subparagraph,
        };
        let mut next = {
            let curr_count = use_context::<RwSignal<CurrentCounter>>();
            let current = curr_count.map_or_else(
                || CurrentCounter::Fixed(SectionCounter::default()),
                |c| c.get_untracked(),
            );
            let next = current.inc_at(sect_level);
            if let Some(c) = curr_count {
                c.update_untracked(|e| *e = next);
            }
            next
        };
        slf.counters.update_untracked(|ctrs| {
            if let Some(v) = ctrs.get(&id) {
                if let Some(first) = v.get_untracked().0 {
                    next = first.0;
                } else {
                    v.update(|v| v.0 = Some((next, sect_level)));
                }
            } else {
                let newsig = RwSignal::new((Some((next, sect_level)), None));
                next = CurrentCounter::Relative(Signal::derive(move || {
                    newsig.get().0.expect("bug").0.get()
                }));
                ctrs.insert(id.clone(), newsig);
            }
        });
        provide_context(RwSignal::new(next));
        if let Some(ipr) = use_context::<InInputref>() {
            ipr.update(next, sect_level);
        }
        if extract {
            slf.toc.update(|toc| {
                let Some(ch) = Self::get_toc_at(toc, id.as_ref()) else {
                    return;
                };
                ch.push(TocElem::Section {
                    title: None,
                    uri: uri.clone(),
                    id: id.to_string(),
                    children: Vec::new(),
                });
            });
            slf.counters.update_untracked(|ctrs| {
                if let Some(c) = ctrs.get(&id) {
                    c.update(|v| {
                        v.0 = Some((next, sect_level));
                    });
                } else {
                    ctrs.insert(id.clone(), RwSignal::new((Some((next, sect_level)), None)));
                }
            });
        }
        SectionInfo {
            uri,
            id,
            lvl: new_level,
            counter: next,
        }
    }

    pub fn navigate_to_fragment() {
        let fragment = RwSignal::new(String::new());
        tracing::trace!("Setting up navigation system");

        #[cfg(any(feature = "csr", feature = "hydrate"))]
        {
            if let Ok(mut frag) = window().location().hash()
                && frag.starts_with('#')
            {
                frag.remove(0);
                tracing::warn!("Current fragment: {frag}");
                fragment.set(frag);
            }
            fragment_listener(fragment);
        }

        let done = RwSignal::new(false);
        Effect::new(move || {
            done.set(true);
        });
        Effect::new(move || {
            fragment.track();
            if done.get() {
                let fragment = fragment.get();
                if !fragment.is_empty() {
                    tracing::warn!("Navigating to {fragment}");
                    Self::navigate_to(&fragment);
                }
            }
        });
    }

    fn retry() {
        if let Some(slf) = use_context::<Self>()
            && let Some(s) = slf.redo.update_untracked(std::mem::take)
        {
            Self::navigate_to(&s);
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn navigate_to(_id: &str) {
        #[cfg(any(feature = "csr", feature = "hydrate"))]
        {
            #[allow(clippy::used_underscore_binding)]
            let Ok(id) = _id.parse::<Id>() else { return };
            let Some(slf) = use_context::<Self>() else {
                return;
            };
            tracing::trace!("Looking for #{id}");
            let mut curr = id.clone();
            slf.ids.with_untracked(move |ids| {
                loop {
                    match ids.get(&curr) {
                        None => {
                            tracing::debug!("navigation id {curr} not known (yet)\n{:?}!", slf.ids);
                            slf.redo.update_untracked(|r| *r = Some(id.to_string()));
                        }
                        Some(SectionOrInputref::Section) => {
                            tracing::debug!("Navigating to #{curr}");
                            #[allow(unused_variables)]
                            if let Some(e) = document().get_element_by_id(curr.as_ref()) {
                                tracing::trace!("scrolling to #{curr}");
                                #[cfg(target_family = "wasm")]
                                {
                                    let options = web_sys::ScrollIntoViewOptions::new();
                                    options.set_behavior(web_sys::ScrollBehavior::Smooth);
                                    options.set_block(web_sys::ScrollLogicalPosition::Start);
                                    e.scroll_into_view_with_scroll_into_view_options(&options);
                                }
                            } else {
                                tracing::warn!("section with id {curr} not found!");
                            }
                            return;
                        }
                        Some(SectionOrInputref::Inputref(a)) => {
                            if !a.is_done_untracked() {
                                tracing::trace!("expanding inputref {curr}");
                                let id = id.to_string();
                                a.on_set(move || {
                                    tracing::trace!("resuming navigation to {id}");
                                    Self::navigate_to(&id);
                                });
                                a.activate();
                            }
                        }
                    }
                    if let Some((a, _)) = curr.as_ref().rsplit_once('/') {
                        // SAFETY: id prefixes are valid ids
                        unsafe {
                            curr = a.parse().unwrap_unchecked();
                        }
                    } else {
                        return;
                    }
                }
            });
        }
    }

    fn new_id(end: &str) -> Id {
        let id = with_context::<CurrentId, _>(|s| {
            s.0.as_ref()
                .and_then(|s| format!("{s}/{end}").parse::<Id>().ok())
        })
        .flatten()
        .unwrap_or_else(|| end.parse().expect("needs to be valid id"));
        provide_context(CurrentId(Some(id.clone())));
        id
    }

    fn get_toc_at<'t>(toc: &'t mut Vec<TocElem>, id: &str) -> Option<&'t mut Vec<TocElem>> {
        let mut path = id.split('/');
        let _ = path.next_back()?;
        let mut toc = toc;
        loop {
            let Some(next) = path.next() else {
                return Some(toc);
            };
            if let Some(next) = toc.iter_mut().find_map(|t| match t {
                TocElem::Section { id, children, .. } | TocElem::Inputref { id, children, .. }
                    if id.rsplit_once('/').is_some_and(|(_, last)| last == next) || id == next =>
                {
                    Some(children)
                }
                _ => None,
            }) {
                toc = next;
            } else {
                return None;
            }
        }
    }

    fn toc_find_mut(toc: &mut [TocElem], pred: impl Fn(&TocElem) -> bool) -> Option<&mut TocElem> {
        let mut curr: std::slice::IterMut<TocElem> = toc.iter_mut();
        let mut stack: SmallVec<std::slice::IterMut<TocElem>, 2> = SmallVec::new();
        loop {
            if let Some(elem) = curr.next() {
                if pred(elem) {
                    return Some(elem);
                }
                let children: &mut [_] = match elem {
                    TocElem::Section { children, .. }
                    | TocElem::Inputref { children, .. }
                    | TocElem::SkippedSection { children } => children,
                    _ => return Some(elem),
                };
                stack.push(std::mem::replace(&mut curr, children.iter_mut()));
            } else if let Some(s) = stack.pop() {
                curr = s;
            } else {
                return None;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum SectionOrInputref {
    Section,
    Inputref(OneShot),
}

#[derive(Debug, Clone)]
struct InInputref {
    inner: Option<
        RwSignal<(
            Option<(CurrentCounter, SectionLevel)>,
            Option<(CurrentCounter, SectionLevel)>,
        )>,
    >,
    parent: Option<Box<Self>>,
}
impl InInputref {
    fn update(&self, ctr: CurrentCounter, lvl: SectionLevel) {
        if let Some(inner) = &self.inner {
            inner.update(|inner| {
                if inner.0.is_none() {
                    inner.0 = Some((ctr, lvl));
                } else if let Some((i, _)) = inner.0
                    && !ctr.get_untracked().sub_of(i.get_untracked())
                {
                    inner.1 = Some((ctr, lvl));
                }
            });
            if let Some(p) = &self.parent {
                p.update(ctr, lvl);
            }
        }
    }
}

#[derive(Clone)]
pub struct Inputref {
    pub uri: DocumentElementUri,
    pub target: DocumentUri,
    pub id: Id,
    pub replace: OneShot,
    pub done: SetOneShotDone,
    title_counter: RwSignal<(
        Option<(CurrentCounter, SectionLevel)>,
        Option<(CurrentCounter, SectionLevel)>,
    )>,
    title: RwSignal<Box<str>>,
}

impl std::hash::Hash for Inputref {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
    }
}
impl PartialEq for Inputref {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
    }
}
impl Inputref {
    pub fn title<V: crate::FtmlViews>(&self) -> impl IntoView + use<V> {
        use leptos::either::EitherOf3::{A, B, C};
        let title = self.title;
        let ctr = self.title_counter;
        move || {
            view! {
                <div style="display:inline-flex;flex-direction:row;">{
                    match ctr.get() {
                        (Some((s,slvl)),Some((e,elvl))) => A(view!{<span style="white-space:pre;">{s.get().into_view(slvl)}" — "{e.get().into_view(elvl)}" "</span>}),
                        (Some((s,lvl)),_) => B(view!{<span style="white-space:pre;">{s.get().into_view(lvl)}" "</span>}),
                        _ => C(())
                    }
                }
                {V::render_ftml(title.with(ToString::to_string),None)}
                </div>
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct SectionInfo {
    pub uri: DocumentElementUri,
    pub id: Id,
    lvl: LogicalLevel,
    counter: CurrentCounter,
}
impl SectionInfo {
    pub const fn level(&self) -> LogicalLevel {
        self.lvl
    }
    pub fn style(&self) -> Memo<Option<String>> {
        let top = use_context::<DocumentStructure>().map(|s| s.top_level);
        let counter = self.counter;
        let lvl = self.lvl;
        Memo::new(move |_| {
            let LogicalLevel::Section(lvl) = lvl else {
                return None;
            };
            let counter =
                top.map_or_else(|| counter.get(), |top| counter.get().shift_by(top.get()));
            let sects = counter.values;
            Some(match lvl {
                SectionLevel::Part => format!("counter-set:ftml-part {};", sects[0]),
                SectionLevel::Chapter => format!(
                    "counter-set:ftml-part {} ftml-chapter {}",
                    sects[0], sects[1]
                ),
                SectionLevel::Section => format!(
                    "counter-set:ftml-part {} ftml-chapter {} ftml-section {}",
                    sects[0], sects[1], sects[2]
                ),
                SectionLevel::Subsection => format!(
                    "counter-set:ftml-part {} ftml-chapter {} ftml-section {} ftml-subsection {}",
                    sects[0], sects[1], sects[2], sects[3],
                ),
                SectionLevel::Subsubsection => format!(
                    "counter-set:ftml-part {} ftml-chapter {} ftml-section {} ftml-subsection {} ftml-subsubsection {}",
                    sects[0], sects[1], sects[2], sects[3], sects[4],
                ),
                _ => return None,
            })
        })
    }
    pub fn class(&self) -> Memo<Option<&'static str>> {
        let top = use_context::<DocumentStructure>().map(|s| s.top_level);
        let lvl = self.lvl;
        Memo::new(move |_| {
            let LogicalLevel::Section(lvl) = lvl else {
                return None;
            };
            /*let lvl = top.map_or(lvl, |top| {
                ((top.get() as u8) + (lvl as u8))
                    .try_into()
                    .unwrap_or(SectionLevel::Part)
            });*/
            Some(match lvl {
                SectionLevel::Part => "ftml-part",
                SectionLevel::Chapter => "ftml-chapter",
                SectionLevel::Section => "ftml-section",
                SectionLevel::Subsection => "ftml-subsection",
                SectionLevel::Subsubsection => "ftml-subsubsection",
                SectionLevel::Paragraph => "ftml-paragraph",
                SectionLevel::Subparagraph => "ftml-subparagraph",
            })
        })
    }
}

#[derive(Clone)]
struct CurrentId(Option<Id>);

trait IsCounter:
    Copy
    + PartialEq
    + std::ops::Add<Self, Output = Self>
    + std::ops::AddAssign<Self>
    + Default
    + Clone
    + Send
    + Sync
    + std::fmt::Debug
    + std::fmt::Display
    + 'static
{
    fn one() -> Self;
}

impl IsCounter for u16 {
    #[inline]
    fn one() -> Self {
        1
    }
}
impl IsCounter for u32 {
    #[inline]
    fn one() -> Self {
        1
    }
}

#[derive(Copy, Clone, Debug)]
enum CurrentCounter {
    Fixed(SectionCounter),
    Relative(Signal<SectionCounter>),
}
impl CurrentCounter {
    fn get(&self) -> SectionCounter {
        match self {
            Self::Fixed(c) => *c,
            Self::Relative(r) => r.get(),
        }
    }
    fn get_untracked(&self) -> SectionCounter {
        match self {
            Self::Fixed(c) => *c,
            Self::Relative(r) => r.get_untracked(),
        }
    }
    fn inc_at(self, lvl: SectionLevel) -> Self {
        match self {
            Self::Fixed(s) => Self::Fixed(s.inc_at(lvl)),
            Self::Relative(sig) => Self::Relative(Signal::derive(move || sig.get().inc_at(lvl))),
        }
    }
    fn inc_signal(self, lvl: Signal<LogicalLevel>) -> Self {
        Self::Relative(Signal::derive(move || match self {
            Self::Fixed(s) => {
                if let LogicalLevel::Section(lvl) = lvl.get() {
                    s.inc_at(lvl)
                } else {
                    s
                }
            }
            Self::Relative(s) => {
                if let LogicalLevel::Section(lvl) = lvl.get() {
                    s.get().inc_at(lvl)
                } else {
                    s.get()
                }
            }
        }))
    }
}

/// part, chapter, section, subsection, subsubsection, paragraph
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
struct SectionCounter {
    values: [u16; 7],
}

impl SectionCounter {
    pub fn inc_at(mut self, lvl: SectionLevel) -> Self {
        let idx: u8 = lvl.into();
        self.values[idx as usize] += 1;
        for i in ((idx + 1) as usize)..=6 {
            self.values[i] = 0;
        }
        self
    }

    pub fn sub_of(self, other: Self) -> bool {
        for i in 0..=6 {
            if self.values[i] != other.values[i] && other.values[i] > 0 {
                return false;
            }
        }
        true
    }
    pub fn into_view(self, at: SectionLevel) -> impl IntoView {
        if let Some(sig) = with_context::<DocumentStructure, _>(|ds| ds.top_level) {
            let max = sig.get();
            let at = ((max as u8) + (at as u8))
                .try_into()
                .unwrap_or(SectionLevel::Subparagraph);
            leptos::either::Either::Left(move || {
                self.shift_by(max).into_view_directly(at, Some(max))
            })
        } else {
            leptos::either::Either::Right(self.into_view_directly(at, None))
        }
    }

    fn into_view_directly(self, at: SectionLevel, max: Option<SectionLevel>) -> impl IntoView {
        use std::fmt::Write;
        struct Roman(u16);
        const ROMANS: &[(u16, &str); 13] = &[
            (1000, "M"),
            (900, "CM"),
            (500, "D"),
            (400, "CD"),
            (100, "C"),
            (90, "XC"),
            (50, "L"),
            (40, "XL"),
            (10, "X"),
            (9, "IX"),
            (5, "V"),
            (4, "IV"),
            (1, "I"),
        ];
        impl std::fmt::Display for Roman {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut n = self.0;
                write!(f, "Part ")?;
                while let Some((v, numeral)) = ROMANS.iter().find(|(v, _)| n >= *v) {
                    write!(f, "{numeral}")?;
                    n -= *v;
                }
                Ok(())
            }
        }

        if self.values[0] > 0 && at == SectionLevel::Part {
            Roman(self.values[0]).to_string()
        } else {
            // Chapter
            if self.values[1] > 0 && at == SectionLevel::Chapter
            //&& self.values[2..].iter().all(|v| *v == 0)
            //&& max.is_none_or(|v| v <= SectionLevel::Chapter)
            {
                format!("Chapter {}", self.values[1])
            } else {
                let mut ret = if max.is_none_or(|lvl| lvl >= SectionLevel::Chapter) {
                    format!("{}.", self.values[1])
                } else {
                    String::new()
                };
                if self.values[2..].iter().any(|v| *v > 0) {
                    let _ = write!(ret, "{}", self.values[2]);
                    if self.values[3..].iter().any(|v| *v > 0) {
                        let _ = write!(ret, ".{}", self.values[3]);
                        if self.values[4..].iter().any(|v| *v > 0) {
                            let _ = write!(ret, ".{}", self.values[4]);
                            if self.values[5..].iter().any(|v| *v > 0) {
                                let _ = write!(ret, ".{}", self.values[5]);
                                if self.values[6] > 0 {
                                    let _ = write!(ret, ".{}", self.values[6]);
                                }
                            }
                        }
                    }
                }
                ret
            }
        }
    }

    fn shift_by(mut self, lvl: SectionLevel) -> Self {
        if lvl == SectionLevel::Part {
            return self;
        }
        let by = (lvl as u8) as usize;
        self.values[6] = self.values[6 - by..].iter().sum();
        let mut curr = by + 1;
        while curr <= 6 {
            self.values[(6 - curr) + by] = self.values[6 - curr];
            curr += 1;
        }
        for i in 0..by {
            self.values[i] = 0;
        }

        self
    }
}
impl std::fmt::Display for SectionCounter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{} {} {} {} {} {} {}]",
            self.values[0],
            self.values[1],
            self.values[2],
            self.values[3],
            self.values[4],
            self.values[5],
            self.values[6]
        )
    }
}

impl IsCounter for SectionCounter {
    fn one() -> Self {
        panic!("That's not how sectioning works")
    }
}

struct AllCounters {
    current: LogicalLevel,
    max: SectionLevel,
    sections: SectionCounter,
    initialized: bool,
    names: RwSignal<Vec<Id>>,
}

impl std::ops::Add<SectionLevel> for SectionCounter {
    type Output = Self;
    fn add(self, rhs: SectionLevel) -> Self::Output {
        let idx: u8 = rhs.into();
        let mut s = Self::default();
        s.values[idx as usize] = 1;
        self + s
    }
}

impl std::ops::Add<Self> for SectionCounter {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut changed = false;
        Self {
            values: [
                {
                    if rhs.values[0] > 0 {
                        changed = true;
                    }
                    self.values[0] + rhs.values[0]
                },
                {
                    if rhs.values[1] > 0 {
                        changed = true;
                    }
                    self.values[1] + rhs.values[1]
                },
                {
                    if changed {
                        0
                    } else {
                        if rhs.values[2] > 0 {
                            changed = true;
                        }
                        self.values[2] + rhs.values[2]
                    }
                },
                {
                    if changed {
                        0
                    } else {
                        if rhs.values[3] > 0 {
                            changed = true;
                        }
                        self.values[3] + rhs.values[3]
                    }
                },
                {
                    if changed {
                        0
                    } else {
                        if rhs.values[4] > 0 {
                            changed = true;
                        }
                        self.values[4] + rhs.values[4]
                    }
                },
                {
                    if changed {
                        0
                    } else {
                        if rhs.values[5] > 0 {
                            changed = true;
                        }
                        self.values[5] + rhs.values[5]
                    }
                },
                {
                    if changed {
                        0
                    } else {
                        self.values[6] + rhs.values[6]
                    }
                },
            ],
        }
    }
}

impl std::ops::AddAssign<Self> for SectionCounter {
    fn add_assign(&mut self, rhs: Self) {
        let mut changed = rhs.values[0] > 0;
        self.values[0] += rhs.values[0];
        if rhs.values[1] > 0 {
            changed = true;
        }
        self.values[1] += rhs.values[1];
        if changed {
            self.values[2] = 0;
        } else {
            if rhs.values[2] > 0 {
                changed = true;
            }
            self.values[2] += rhs.values[2];
        }
        if changed {
            self.values[3] = 0;
        } else {
            if rhs.values[3] > 0 {
                changed = true;
            }
            self.values[3] += rhs.values[3];
        }
        if changed {
            self.values[4] = 0;
        } else {
            if rhs.values[4] > 0 {
                changed = true;
            }
            self.values[4] += rhs.values[4];
        }
        if changed {
            self.values[5] = 0;
        } else {
            if rhs.values[5] > 0 {
                changed = true;
            }
            self.values[5] += rhs.values[5];
        }
        if changed {
            self.values[6] = 0;
        } else {
            self.values[6] += rhs.values[6];
        }
    }
}

/*
#[derive(Default)]
pub struct CurrentTOC {
    pub toc: Option<Vec<TocElem>>,
}
impl CurrentTOC {
    pub fn set(toc: Vec<TocElem>) {
        let ctw = expect_context::<RwSignal<Self>>();
        ctw.update(|ctw| ctw.toc = Some(toc));
    }
    pub(crate) fn set_title(&mut self, uri: &DocumentElementUri, title: Box<str>) {
        if let Some(e) = self.find_mut(|e| matches!(e,TocElem::Section { uri:u, .. } if u == uri)) {
            let TocElem::Section { title: t, .. } = e else {
                // SAFETY: match above
                unsafe { unreachable_unchecked() }
            };
            *t = Some(title);
            return;
        }
        tracing::warn!("Entry with uri {uri} not found!");
    }

    pub(crate) fn insert_section(&mut self, id: String, uri: DocumentElementUri) {
        let Some(ch) = self.get_toc_at(&id) else {
            tracing::warn!("Entry with id {id} not found!");
            return;
        };
        ch.push(TocElem::Section {
            title: None,
            uri,
            id,
            children: Vec::new(),
        });
    }
    pub fn iter_dfs(&self) -> Option<impl Iterator<Item = &TocElem>> {
        struct TOCIterator<'b> {
            curr: std::slice::Iter<'b, TocElem>,
            stack: SmallVec<std::slice::Iter<'b, TocElem>, 2>,
        }
        impl<'b> Iterator for TOCIterator<'b> {
            type Item = &'b TocElem;
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    if let Some(elem) = self.curr.next() {
                        let children: &'b [_] = match elem {
                            TocElem::Section { children, .. }
                            | TocElem::Inputref { children, .. }
                            | TocElem::SkippedSection { children } => children,
                            _ => return Some(elem),
                        };
                        self.stack
                            .push(std::mem::replace(&mut self.curr, children.iter()));
                        return Some(elem);
                    } else if let Some(s) = self.stack.pop() {
                        self.curr = s;
                    } else {
                        return None;
                    }
                }
            }
        }
        self.toc.as_deref().map(|t| TOCIterator {
            curr: t.iter(),
            stack: SmallVec::new(),
        })
    }

    pub fn find_mut(&mut self, pred: impl Fn(&TocElem) -> bool) -> Option<&mut TocElem> {
        let mut curr: std::slice::IterMut<TocElem> = self.toc.as_mut()?.iter_mut();
        let mut stack: SmallVec<std::slice::IterMut<TocElem>, 2> = SmallVec::new();
        loop {
            if let Some(elem) = curr.next() {
                if pred(elem) {
                    return Some(elem);
                }
                let children: &mut [_] = match elem {
                    TocElem::Section { children, .. }
                    | TocElem::Inputref { children, .. }
                    | TocElem::SkippedSection { children } => children,
                    _ => return Some(elem),
                };
                stack.push(std::mem::replace(&mut curr, children.iter_mut()));
            } else if let Some(s) = stack.pop() {
                curr = s;
            } else {
                return None;
            }
        }
    }
}

pub struct NavElems {
    //initialized: RwSignal<bool>,
    titles: rustc_hash::FxHashMap<DocumentUri, RwSignal<String>>,
}




impl NavElems {
    pub(crate) fn new() -> Self {
        Self {
            ids: std::collections::HashMap::default(),
            titles: std::collections::HashMap::default(),
            redo: None, //initialized: RwSignal::new(false),
        }
    }

    pub fn update_untracked<R>(f: impl FnOnce(&mut Self) -> R) -> R {
        expect_context::<RwSignal<Self>>().update_untracked(f)
    }

    pub(crate) fn new_section(id: &str) -> String {
        Self::update_untracked(|ne| {
            let id = Self::new_id(id);
            ne.ids.insert(id.clone(), SectionOrInputref::Section);
            id
        })
    }

    fn new_id(s: &str) -> String {
        with_context::<CurrentId, _>(|id| {
            if id.0.is_empty() {
                s.to_string()
            } else {
                format!("{}/{s}", id.0)
            }
        })
        .unwrap_or_else(|| s.to_string())
    }

    pub fn set_title(&mut self, uri: DocumentUri, title: String) {
        tracing::debug!("Setting title of {uri} to \n{title}");
        match self.titles.entry(uri) {
            std::collections::hash_map::Entry::Occupied(e) => e.get().set(title),
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(RwSignal::new(title));
            }
        }
    }

    pub fn retry() {
        if let Some(selfie) = use_context::<RwSignal<Self>>()
            && let Some(s) = selfie.update_untracked(|s| std::mem::take(&mut s.redo))
        {
            Self::navigate_to(selfie, &s);
        }
    }




}
 */

#[cfg(any(feature = "csr", feature = "hydrate"))]
fn fragment_listener(signal: RwSignal<String>) {
    use leptos::wasm_bindgen::JsCast;
    fn get_anchor(e: leptos::web_sys::Element) -> Option<leptos::web_sys::Element> {
        let mut curr = e;
        loop {
            if curr.tag_name().to_uppercase() == "A" {
                return Some(curr);
            }
            if curr.tag_name().to_uppercase() == "BODY" {
                return None;
            }
            if let Some(parent) = curr.parent_element() {
                curr = parent;
            } else {
                return None;
            }
        }
    }
    tracing::info!("Setting up fragment listener");
    let on_hash_change =
        leptos::wasm_bindgen::prelude::Closure::wrap(Box::new(move |_e: leptos::web_sys::Event| {
            if let Ok(mut frag) = window().location().hash()
                && frag.starts_with('#')
            {
                frag.remove(0);
                tracing::trace!("Updating URL fragment to {frag}");
                signal.try_set(frag);
            }
        }) as Box<dyn FnMut(_)>);

    let on_anchor_click = leptos::wasm_bindgen::prelude::Closure::wrap(Box::new(
        move |e: leptos::web_sys::MouseEvent| {
            if let Some(e) = e
                .target()
                .and_then(|t| t.dyn_into::<leptos::web_sys::Element>().ok())
                && let Some(e) = get_anchor(e)
                && let Some(mut href) = e.get_attribute("href")
                && href.starts_with('#')
            {
                href.remove(0);
                tracing::trace!("Updating URL fragment as {href}");
                signal.try_set(href);
            }
        },
    ) as Box<dyn FnMut(_)>);

    tracing::trace!("Setting URL listeners");

    let _ = window()
        .add_event_listener_with_callback("hashchange", on_hash_change.as_ref().unchecked_ref());
    let _ = window()
        .add_event_listener_with_callback("popstate", on_hash_change.as_ref().unchecked_ref());
    let _ = window()
        .add_event_listener_with_callback("click", on_anchor_click.as_ref().unchecked_ref());
    on_hash_change.forget();
    on_anchor_click.forget();
}

#[derive(Clone, Default, Copy)]
struct SmartCounter<N: IsCounter>(RwSignal<SmartCounterI<N>>);
impl<N: IsCounter> std::fmt::Debug for SmartCounter<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
            .with_untracked(|s| f.debug_struct("SmartCounter").field("inner", s).finish())
    }
}

#[derive(Clone)]
struct Cutoff<N: IsCounter> {
    previous: Option<std::sync::Arc<Cutoff<N>>>,
    since: N,
    set: RwSignal<N>,
}

impl<N: IsCounter> Cutoff<N> {
    fn get(&self) -> N {
        let r = self.since + self.set.get();
        self.previous.as_ref().map_or(r, |p| p.get() + r)
    }
    fn depth(&self) -> u16 {
        self.previous.as_ref().map_or(1, |p| p.depth() + 1)
    }
    fn get_untracked(&self) -> N {
        let r = self.since + self.set.get_untracked();
        self.previous.as_ref().map_or(r, |p| p.get_untracked() + r)
    }
}

impl<N: IsCounter> std::fmt::Debug for Cutoff<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cutoff")
            .field("depth", &self.depth())
            .field(
                "previous",
                &self.previous.as_ref().map(|p| p.get_untracked()),
            )
            .field("since", &self.since)
            .field("set", &self.set.get_untracked())
            .finish()
    }
}

#[derive(Debug, Clone, Default)]
struct SmartCounterI<N: IsCounter> {
    cutoff: Option<Cutoff<N>>,
    since: N,
}
impl<N: IsCounter> SmartCounterI<N> {
    fn get(&self) -> N {
        self.cutoff
            .as_ref()
            .map_or(self.since, |cutoff| cutoff.get() + self.since)
    }
}

impl<N: IsCounter> SmartCounter<N> {
    fn inc_memo<T: Send + Sync + 'static + PartialEq>(
        &self,
        f: impl Fn(N) -> T + Send + Sync + 'static,
    ) -> Memo<T> {
        self.0.update_untracked(|SmartCounterI { cutoff, since }| {
            *since += N::one();
            let since = *since;
            if let Some(cutoff) = cutoff {
                let cutoff = cutoff.clone();
                Memo::new(move |_| f(cutoff.get() + since))
            } else {
                Memo::new(move |_| f(since))
            }
        })
    }

    fn inc(&self) {
        self.0
            .update_untracked(|SmartCounterI { since, .. }| *since += N::one());
    }
    fn memo<T: Send + Sync + 'static + PartialEq>(
        &self,
        f: impl Fn(N) -> T + Send + Sync + 'static,
    ) -> Memo<T> {
        self.0.with_untracked(|SmartCounterI { cutoff, since }| {
            let since = *since;
            if let Some(cutoff) = cutoff {
                let cutoff = cutoff.clone();
                Memo::new(move |_| f(cutoff.get() + since))
            } else {
                Memo::new(move |_| f(since))
            }
        })
    }
    fn reset(&self) {
        self.0.update_untracked(|x| *x = SmartCounterI::default());
    }

    fn split(&self) -> Self {
        let SmartCounterI { cutoff, since } = self.0.get_untracked();
        let ret = Self(RwSignal::new(SmartCounterI {
            cutoff: cutoff.clone(),
            since,
        }));

        let previous = cutoff.map(std::sync::Arc::new);
        let new_cutoff = Cutoff {
            previous,
            since,
            set: RwSignal::new(N::default()),
        };
        self.0.update_untracked(
            |SmartCounterI {
                 cutoff: nctf,
                 since: snc,
             }| {
                *nctf = Some(new_cutoff);
                *snc = N::default();
            },
        );
        ret
    }
}
