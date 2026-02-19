use ftml_ontology::{
    narrative::{
        documents::{DocumentCounter, DocumentStyle, TocElem},
        elements::SectionLevel,
    },
    utils::{RefTree, TreeIter},
};
use ftml_uris::{DocumentElementUri, DocumentUri, Id, IsNarrativeUri, NarrativeUri};
use leptos::prelude::*;

use crate::{
    DocumentState, FtmlViews,
    counters::{
        CurrentCounters, CurrentSlide, DynamicCounter, ParagraphCounters, ParagraphCountersI,
        SectionCounters,
    },
    document::CurrentUri,
    structure::{CurrentId, DocumentStructure},
    utils::local_cache::{SendBackend, WithLocalCache},
};

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

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub enum TocStyle {
    Extract,
    Get,
    #[default]
    None,
    Ready,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Toc {
    None,
    Extract {
        toc: RwSignal<Option<Vec<FinalTocEntry>>>,
    },
    Get {
        state: StoredValue<either::Either<TocStateGet, TocStateReady>>,
        in_level: RwSignal<Option<SectionLevel>>,
    },
    Ready(TocStateReady),
}

#[derive(Clone)]
pub(crate) struct InputrefTitle(either::Either<InputrefTitleI, Box<str>>);
#[derive(Clone)]
struct InputrefTitleI {
    at: SectionLevel,
    top: SectionLevel,
    counters: either::Either<
        Option<(SectionCounters, Option<SectionCounters>)>,
        RwSignal<Option<(SectionCounters, Option<SectionCounters>)>>,
    >,
    title: either::Either<Box<str>, ReadSignal<Box<str>>>,
}
impl InputrefTitle {
    pub fn default(uri: &DocumentUri) -> Self {
        Self(either::Right(
            uri.document_name().to_string().into_boxed_str(),
        ))
    }
    pub fn as_view<V: crate::FtmlViews>(&self) -> impl IntoView + use<V> {
        use leptos::either::Either::{Left as L, Right as R};
        fn do_counters(
            start: SectionCounters,
            end: Option<SectionCounters>,
            at: SectionLevel,
            top: SectionLevel,
        ) -> impl IntoView {
            view! {<span style="white-space:pre;">{
                if let Some(end) = end && !end.sub_of(start) {
                    R(view! {
                        {start.into_view(at,top)}
                        " — "
                        {end.into_view(at,top)}
                    })
                } else {
                    L(start.into_view(at,top))
                }
            }</span>}
        }
        let inner = move || match &self.0 {
            either::Right(s) => L(V::render_ftml(s.to_string(), None)),
            either::Left(ipr) => {
                let counters = match ipr.counters {
                    either::Left(None) => None,
                    either::Left(Some((start, end))) => {
                        Some(L(do_counters(start, end, ipr.at, ipr.top)))
                    }
                    either::Right(sig) => {
                        let at = ipr.at;
                        let top = ipr.top;
                        Some(R(move || {
                            sig.get()
                                .map(|(start, end)| do_counters(start, end, at, top))
                        }))
                    }
                };
                let text = match &ipr.title {
                    either::Left(s) => L(V::render_ftml(s.to_string(), None)),
                    either::Right(sig) => {
                        let sig = *sig;
                        R(move || V::render_ftml(sig.get().to_string(), None))
                    }
                };

                R(view! {{counters}{text}})
            }
        };
        view! {
            <div style="display:inline-flex;flex-direction:row;">{inner()}
            </div>
        }
    }
}

impl Toc {
    pub const fn style(&self) -> TocStyle {
        match self {
            Self::None => TocStyle::None,
            Self::Extract { .. } => TocStyle::Extract,
            Self::Get { .. } => TocStyle::Get,
            Self::Ready { .. } => TocStyle::Ready,
        }
    }
    pub fn inputref_title(
        &self,
        uri: &DocumentUri,
        id: &Id,
        at: SectionLevel,
        top: SectionLevel,
    ) -> InputrefTitle {
        match self {
            Self::Ready(state) => state
                .inputrefs
                .with_value(|ipr| ipr.get(id).map(|e| e.title(at, top)))
                .unwrap_or_else(|| InputrefTitle::default(uri)),
            Self::Get { state, .. } => match state.get_value() {
                either::Left(state) => state.inputref_title(id, uri, at, top),
                either::Right(state) => state
                    .inputrefs
                    .with_value(|i| i.get(id).map(|e| e.title(at, top)))
                    .unwrap_or_else(|| InputrefTitle::default(uri)),
            },
            Self::Extract { toc } => {
                toc.update_untracked(|toc| {
                    if let Some(toc) = toc
                        && let Some((v, _)) = get_toc_at(toc, id.as_ref())
                    {
                        v.push(FinalTocEntry::Inputref {
                            id: id.clone(),
                            children: Vec::new(),
                        });
                    }
                });
                InputrefTitle(either::Right(
                    uri.document_name().to_string().into_boxed_str(),
                ))
            }
            Self::None => InputrefTitle(either::Right(
                uri.document_name().to_string().into_boxed_str(),
            )),
        }
    }
    pub fn init(&self, lvl: SectionLevel) {
        match self {
            Self::Ready(state) => {
                if let Some((sig, ret)) = state.init(lvl) {
                    sig.set(Some(ret));
                }
                DocumentStructure::retry();
            }
            Self::Get { in_level, .. } => {
                in_level.set(Some(lvl));
            }
            _ => (),
        }
    }

    pub fn new<Be: SendBackend>(
        source: TocSource,
        styles: StoredValue<(Vec<DocumentStyle>, Vec<DocumentCounter>)>,
    ) -> Self {
        match source {
            TocSource::None => Self::None,
            TocSource::Extract => Self::Extract {
                toc: RwSignal::new(None),
            },
            TocSource::Get => {
                let (state, in_level) = TocStateGet::new::<Be>(styles);
                Self::Get { state, in_level }
            }
            TocSource::Ready(v) => Self::Ready(TocStateReady::new(v.into_boxed_slice())),
        }
    }

    pub fn set_section_title(&self, top: SectionLevel, get: impl FnOnce() -> String) {
        if let Self::Extract { toc } = self {
            let NarrativeUri::Element(uri) = expect_context::<CurrentUri>().0 else {
                return;
            };
            let CurrentId(Some(id)) = expect_context::<CurrentId>() else {
                return;
            };
            let counter = CurrentCounters::current();
            toc.update(|toc| {
                if let Some(toc) = toc
                    && let Some((v, lvl)) = get_toc_at(toc, id.as_ref())
                {
                    let level = lvl.map_or(top, SectionLevel::inc);
                    v.push(FinalTocEntry::Section {
                        level,
                        uri,
                        counter,
                        title: get().into_boxed_str(),
                        id,
                        children: Vec::new(),
                    });
                }
            });
        }
    }

    pub fn get_inputref_counters(
        &self,
        id: &Id,
        current_slides: CurrentSlide,
        current_paras: ParagraphCounters,
    ) -> Option<(CurrentSlide, ParagraphCountersI, ParagraphCounters)> {
        match self {
            Self::Extract { .. } | Self::None => None,
            Self::Ready(toc) => toc.inputrefs.with_value(|i| {
                i.get(id).map(|ipr| {
                    (
                        CurrentSlide::Static(ipr.slides),
                        ParagraphCountersI::Static(ipr.paras.clone()),
                        current_paras.take(),
                    )
                })
            }),
            Self::Get { state, .. } => {
                let state = state.get_value();
                match state {
                    either::Right(state) => state.inputrefs.with_value(|i| {
                        i.get(id).map(|ipr| {
                            (
                                CurrentSlide::Static(ipr.slides),
                                ParagraphCountersI::Static(ipr.paras.clone()),
                                current_paras.take(),
                            )
                        })
                    }),
                    either::Left(TocStateGet { inputrefs, .. }) => inputrefs
                        .with_value(|i| i.get(id).map(|i| (i.slides, i.paras)))
                        .map(|(slides, paras)| {
                            slides.update_untracked(|c| *c = current_slides);

                            let mut previous_paras = ParagraphCountersI::default();
                            current_paras.0.update_value(|v| {
                                std::mem::swap(v, &mut previous_paras);
                            });

                            paras.update_untracked(|p| *p = previous_paras.clone());
                            let new_paras = StoredValue::new(previous_paras);
                            (
                                CurrentSlide::AfterInputref {
                                    since: 0,
                                    previous: slides,
                                },
                                ParagraphCountersI::AfterInputref {
                                    previous: paras,
                                    since: rustc_hash::FxHashMap::default(),
                                },
                                ParagraphCounters(new_paras),
                            )
                        }),
                }
            }
        }
    }

    pub fn get_section_counter(&self, id: &Id, default: DynamicCounter) -> DynamicCounter {
        match self {
            Self::Extract { .. } | Self::None => default,
            Self::Ready(toc) => {
                // should only get called after Ready is initialized
                toc.toc.with_value(|toc| {
                    if let either::Right(v) = toc {
                        toc_find(v, |e| {
                            if let FinalTocEntry::Section { id: i, counter, .. } = e
                                && *i == *id
                            {
                                Some(*counter)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(default)
                    } else {
                        default
                    }
                })
            }
            Self::Get { state, .. } => {
                let state = state.get_value();
                match state {
                    either::Left(TocStateGet {
                        section_counters, ..
                    }) => section_counters
                        .with_value(|s| s.get(id).copied())
                        .unwrap_or_else(|| {
                            let new = DynamicCounter::Sig(RwSignal::new(default));
                            section_counters.update_value(|s| {
                                s.insert(id.clone(), new);
                            });
                            new
                        }),
                    // if it's ready, it's initialized
                    either::Right(TocStateReady {
                        toc, //: either::Right(v),
                        ..
                    }) => toc.with_value(|toc| match toc {
                        either::Right(v) => toc_find(v, |e| {
                            if let FinalTocEntry::Section { id: i, counter, .. } = e
                                && *i == *id
                            {
                                Some(*counter)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(default),
                        either::Left(_) => default,
                    }),
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn render_toc<
        V: FtmlViews,
        Cont: Fn(String, &DocumentElementUri, AnyView, Option<AnyView>) -> AnyView
            + Clone
            + Send
            + 'static,
        OnRender: Fn(&[FinalTocEntry]) + Clone + Send + 'static,
        FallBack: Fn() -> AnyView + Clone + Send + 'static,
    >(
        &self,
        cont: Cont,
        on_render: OnRender,
        fallback: FallBack,
        max_level: StoredValue<Option<SectionLevel>>,
    ) -> impl IntoView + use<V, Cont, OnRender, FallBack> {
        fn do_sig<
            V: FtmlViews,
            Cont: Fn(String, &DocumentElementUri, AnyView, Option<AnyView>) -> AnyView
                + Clone
                + Send
                + 'static,
            OnRender: Fn(&[FinalTocEntry]) + Clone + Send + 'static,
            FallBack: Fn() -> AnyView + Clone + Send + 'static,
        >(
            v: RwSignal<Option<Vec<FinalTocEntry>>>,
            cont: Cont,
            on_render: OnRender,
            fallback: FallBack,
            max_level: StoredValue<Option<SectionLevel>>,
        ) -> impl IntoView + use<V, Cont, OnRender, FallBack> {
            move || {
                v.with(|v| {
                    v.as_ref().map_or_else(
                        || leptos::either::Either::Left(fallback()),
                        |v| {
                            on_render(v);
                            leptos::either::Either::Right(render_toc::<V, _>(
                                v,
                                &cont,
                                max_level.get_value().unwrap_or(SectionLevel::Part),
                            ))
                        },
                    )
                })
            }
        }
        match self {
            Self::None => None,
            Self::Extract { toc } => Some(leptos::either::Either::Left(do_sig::<V, _, _, _>(
                *toc, cont, on_render, fallback, max_level,
            ))),
            Self::Ready(state) => {
                let mut toc = None;
                state.toc.update_value(|st| match st {
                    either::Left((Some(sig), _)) => toc = Some(either::Left(*sig)),
                    either::Left((n @ None, _)) => {
                        let sig = RwSignal::new(None);
                        *n = Some(sig);
                        toc = Some(either::Left(sig));
                    }
                    either::Right(v) => toc = Some(either::Right(v.clone())),
                });
                toc.map(|toc| match toc {
                    either::Left(sig) => leptos::either::Either::Left(do_sig::<V, _, _, _>(
                        sig, cont, on_render, fallback, max_level,
                    )),
                    either::Right(v) => {
                        on_render(&v);
                        leptos::either::Either::Right(render_toc::<V, _>(
                            &v,
                            &cont,
                            max_level.get_value().unwrap_or(SectionLevel::Part),
                        ))
                    }
                })
            }
            Self::Get { state, .. } => {
                let mut toc = None;
                let state = state.get_value();
                match state {
                    either::Left(TocStateGet { toc: t, .. }) => {
                        if let Some(sig) = t.get_value() {
                            toc = Some(either::Left(sig));
                        } else {
                            let sig = RwSignal::new(None);
                            t.set_value(Some(sig));
                            toc = Some(either::Left(sig));
                        }
                    }
                    either::Either::Right(TocStateReady { toc: t, .. }) => {
                        let newsig = RwSignal::new(None);
                        t.update_value(|t| match t {
                            either::Left((Some(sig), ..)) => {
                                toc = Some(either::Left(*sig));
                            }
                            either::Left((n @ None, ..)) => {
                                *n = Some(newsig);
                                toc = Some(either::Left(newsig));
                            }
                            either::Right(v) => toc = Some(either::Right(v.clone())),
                        });
                    }
                }
                toc.map(|toc| match toc {
                    either::Left(sig) => leptos::either::Either::Left(do_sig::<V, _, _, _>(
                        sig, cont, on_render, fallback, max_level,
                    )),
                    either::Right(v) => {
                        on_render(&v);
                        leptos::either::Either::Right(render_toc::<V, _>(
                            &v,
                            &cont,
                            max_level.get_value().unwrap_or(SectionLevel::Part),
                        ))
                    }
                })
            }
        }
    }
}

fn render_toc<
    V: FtmlViews,
    Cont: Fn(String, &DocumentElementUri, AnyView, Option<AnyView>) -> AnyView,
>(
    toc: &[FinalTocEntry],
    cont: &Cont,
    max_level: SectionLevel,
) -> impl IntoView + use<V, Cont> {
    toc.iter()
        .map(|e| e.render::<V, _>(cont, max_level))
        .collect_view()
}

#[derive(Debug, Clone)]
pub enum FinalTocEntry {
    Section {
        level: SectionLevel,
        uri: DocumentElementUri,
        counter: DynamicCounter,
        title: Box<str>,
        id: Id,
        children: Vec<Self>,
    },
    Inputref {
        id: Id,
        children: Vec<Self>,
    },
}
impl FinalTocEntry {
    fn render<
        V: FtmlViews,
        Cont: Fn(String, &DocumentElementUri, AnyView, Option<AnyView>) -> AnyView,
    >(
        &self,
        cont: &Cont,
        max_level: SectionLevel,
    ) -> AnyView {
        match self {
            Self::Inputref { children, .. } => children
                .iter()
                .map(|c| c.render::<V, _>(cont, max_level))
                .collect_view()
                .into_any(),
            Self::Section {
                level,
                counter,
                title,
                children,
                id,
                uri,
            } => {
                let header =
                    view! {{counter.into_view(*level,max_level)}" "{V::render_ftml(title.to_string(), None)}}
                        .into_any();
                let children = if rec_empty(children) {
                    None
                } else {
                    Some(
                        children
                            .iter()
                            .map(|c| c.render::<V, _>(cont, max_level))
                            .collect_view()
                            .into_any(),
                    )
                };
                cont(format!("#{id}"), uri, header, children)
            }
        }
    }
}

fn rec_empty(toc: &[FinalTocEntry]) -> bool {
    toc.is_empty()
        || toc.iter().all(|t| match t {
            FinalTocEntry::Section { .. } => false,
            FinalTocEntry::Inputref { children, .. } => rec_empty(children),
        })
}

impl RefTree for FinalTocEntry {
    type Child<'a> = &'a Self;
    fn tree_children(&self) -> impl Iterator<Item = Self::Child<'_>> {
        match self {
            Self::Section { children, .. } | Self::Inputref { children, .. } => children.iter(),
        }
    }
}

fn get_counters(toc: &[FinalTocEntry]) -> (Option<DynamicCounter>, Option<DynamicCounter>) {
    let mut start = None;
    let mut end = None;
    for t in toc.dfs() {
        if let FinalTocEntry::Section { counter, .. } = t {
            if start.is_none() {
                start = Some(*counter);
            } else {
                end = Some(*counter);
            }
        }
    }
    (start, end)
}

fn get_toc_at<'t>(
    toc: &'t mut Vec<FinalTocEntry>,
    id: &str,
) -> Option<(&'t mut Vec<FinalTocEntry>, Option<SectionLevel>)> {
    let mut path = id.split('/');
    let _ = path.next_back()?;
    let mut toc = toc;
    let mut level = None;
    loop {
        let Some(next) = path.next() else {
            return Some((toc, level));
        };
        if let Some(next) = toc.iter_mut().find_map(|t| {
            if let FinalTocEntry::Section { level: nlvl, .. } = t {
                level = Some(*nlvl);
            }
            match t {
                FinalTocEntry::Section { id, children, .. }
                | FinalTocEntry::Inputref { id, children, .. }
                    if id
                        .as_ref()
                        .rsplit_once('/')
                        .is_some_and(|(_, last)| last == next)
                        || id.as_ref() == next =>
                {
                    Some(children)
                }
                _ => None,
            }
        }) {
            toc = next;
        } else {
            return None;
        }
    }
}

fn toc_find<R>(toc: &[FinalTocEntry], pred: impl Fn(&FinalTocEntry) -> Option<R>) -> Option<R> {
    let mut curr: std::slice::Iter<FinalTocEntry> = toc.iter();
    let mut stack = smallvec::SmallVec::<std::slice::Iter<FinalTocEntry>, 2>::new();
    loop {
        if let Some(elem) = curr.next() {
            if let Some(r) = pred(elem) {
                return Some(r);
            }
            let children: &[_] = match elem {
                FinalTocEntry::Section { children, .. }
                | FinalTocEntry::Inputref { children, .. } => children,
            };
            stack.push(std::mem::replace(&mut curr, children.iter()));
        } else if let Some(s) = stack.pop() {
            curr = s;
        } else {
            return None;
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TocStateGet {
    section_counters: StoredValue<rustc_hash::FxHashMap<Id, DynamicCounter>>,
    inputrefs: StoredValue<rustc_hash::FxHashMap<Id, InputrefStateGet>>,
    toc: StoredValue<Option<RwSignal<Option<Vec<FinalTocEntry>>>>>,
}
impl TocStateGet {
    fn new<Be: SendBackend>(
        styles: StoredValue<(Vec<DocumentStyle>, Vec<DocumentCounter>)>,
    ) -> (
        StoredValue<either::Either<Self, TocStateReady>>,
        RwSignal<Option<SectionLevel>>,
    ) {
        let uri = DocumentState::document_uri();
        let orig_toc = RwSignal::new(None);
        // hack to make sure this happens client side only
        let csr = {
            #[cfg(not(any(feature = "csr", feature = "hydrate")))]
            {
                RwSignal::new(false)
            }
            #[cfg(any(feature = "csr", feature = "hydrate"))]
            {
                RwSignal::new(true)
            }
        };
        let _ = Effect::new(move || {
            csr.set(true);
        });
        let fut = send_wrapper::SendWrapper::new(std::cell::Cell::new(Some(
            WithLocalCache::<Be>::default().get_toc(uri),
        )));
        Effect::new(move || {
            if csr.get()
                && let Some(fut) = std::cell::Cell::take(&fut)
            {
                leptos::task::spawn_local(async move {
                    let r = fut.await;
                    if let Ok((_, _, elems)) = r {
                        orig_toc.set(Some(elems));
                    }
                });
            }
        });
        let in_level = RwSignal::new(None);
        let slf = StoredValue::new(either::Left(Self {
            section_counters: StoredValue::new(rustc_hash::FxHashMap::default()),
            inputrefs: StoredValue::new(rustc_hash::FxHashMap::default()),
            toc: StoredValue::new(None),
        }));
        let mut done = false;
        Effect::new(move || {
            orig_toc.track();
            in_level.track();
            if !done
                && let Some(lvl) = in_level.get()
                && let Some(orig) = orig_toc.update_untracked(Option::take)
            {
                done = true;
                let mut todos = Vec::new();
                let mut old = None;
                let (styles, counters) = styles.get_value();

                slf.update_value(|slf| {
                    if let either::Left(slfi) = slf {
                        let (s, td) = slfi.close(orig, lvl, &styles, &counters);
                        slfi.toc
                            .update_value(|v| old = v.take().map(|ft| (ft, s.toc)));
                        *slf = either::Right(s);
                        todos = td;
                    }
                });
                if let Some((old, new)) = old {
                    let new = new.with_value(|new| {
                        if let either::Right(new) = new {
                            Some(new.clone())
                        } else {
                            None
                        }
                    });
                    if let Some(new) = new {
                        old.set(Some(new));
                    }
                }
                for t in todos {
                    t.go();
                }
                DocumentStructure::retry();
            }
        });
        (slf, in_level)
    }

    fn close(
        &mut self,
        elems: Box<[TocElem]>,
        top: SectionLevel,
        styles: &[DocumentStyle],
        counters: &[DocumentCounter],
    ) -> (TocStateReady, Vec<Todo>) {
        let inputrefs = &mut self.inputrefs;
        let sections = &mut self.section_counters;
        let mut state = ConvState {
            get_inputref: Some(&mut |id| {
                let mut r = None;
                inputrefs.update_value(|v| r = v.remove(id));
                r
            }),
            get_section: Some(&mut |id| {
                let mut r = None;
                sections.update_value(|v| r = v.remove(id));
                r
            }),
            inputrefs: rustc_hash::FxHashMap::default(),
            counter: SectionCounters::default(),
            todos: Vec::new(),
            styles,
            counters,
            paras: rustc_hash::FxHashMap::default(),
            slides: 0,
        };
        let ret = convert(&mut state, elems, top);
        (
            TocStateReady {
                inputrefs: StoredValue::new(state.inputrefs),
                toc: StoredValue::new(either::Right(ret)),
            },
            state.todos,
        )
    }

    #[allow(clippy::option_if_let_else)]
    fn inputref_title(
        &self,
        id: &Id,
        uri: &DocumentUri,
        at: SectionLevel,
        top: SectionLevel,
    ) -> InputrefTitle {
        if let Some(v) = self
            .inputrefs
            .with_value(|i| i.get(id).map(|v| v.title(at, top)))
        {
            v
        } else {
            let ipr = InputrefStateGet {
                counter: RwSignal::new(None),
                doc_title: RwSignal::new(uri.document_name().to_string().into_boxed_str()),
                slides: RwSignal::new(CurrentSlide::Static(0)),
                paras: RwSignal::new(ParagraphCountersI::Static(rustc_hash::FxHashMap::default())),
            };
            let ret = ipr.title(at, top);
            self.inputrefs.update_value(|ip| {
                ip.insert(id.clone(), ipr);
            });
            ret
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TocStateReady {
    inputrefs: StoredValue<rustc_hash::FxHashMap<Id, InputrefStateReady>>,
    toc: StoredValue<
        either::Either<
            (Option<RwSignal<Option<Vec<FinalTocEntry>>>>, Box<[TocElem]>),
            Vec<FinalTocEntry>,
        >,
    >,
}
impl TocStateReady {
    fn new(v: Box<[TocElem]>) -> Self {
        Self {
            inputrefs: StoredValue::new(rustc_hash::FxHashMap::default()),
            toc: StoredValue::new(either::Left((None, v))),
        }
    }
    fn init(
        &self,
        top: SectionLevel,
    ) -> Option<(RwSignal<Option<Vec<FinalTocEntry>>>, Vec<FinalTocEntry>)> {
        let mut r = None;
        let (styles, counters) = with_context::<DocumentStructure, _>(|d| d.styles)
            .expect("Not in a document context")
            .get_value();
        self.toc.update_value(|toc| {
            if let either::Left((sig, elems)) = toc {
                let mut state = ConvState {
                    get_inputref: None,
                    get_section: None,
                    styles: &styles,
                    counters: &counters,
                    inputrefs: rustc_hash::FxHashMap::default(),
                    paras: rustc_hash::FxHashMap::default(),
                    counter: SectionCounters::default(),
                    todos: Vec::new(),
                    slides: 0,
                };
                let sig = *sig;
                let ret = convert(&mut state, std::mem::take(elems), top);
                if let Some(sig) = sig {
                    *toc = either::Right(ret.clone());
                    r = Some((sig, ret));
                } else {
                    *toc = either::Right(ret);
                }
            }
        });
        r
    }
}

struct ConvState<'s> {
    get_inputref: Option<&'s mut dyn FnMut(&Id) -> Option<InputrefStateGet>>,
    get_section: Option<&'s mut dyn FnMut(&Id) -> Option<DynamicCounter>>,
    inputrefs: rustc_hash::FxHashMap<Id, InputrefStateReady>,
    styles: &'s [DocumentStyle],
    counters: &'s [DocumentCounter],
    paras: rustc_hash::FxHashMap<Id, u32>,
    counter: SectionCounters,
    todos: Vec<Todo>,
    slides: u32,
}

#[allow(clippy::too_many_lines)]
fn convert(
    state: &mut ConvState,
    elems: Box<[TocElem]>,
    level: SectionLevel,
) -> Vec<FinalTocEntry> {
    let mut ret = Vec::new();
    for e in elems {
        match e {
            TocElem::SkippedSection { children } => {
                for c in state.counters {
                    if c.parent.is_some_and(|p| p <= level) {
                        state.paras.remove(&c.name);
                    }
                }
                ret.extend(convert(state, children.into_boxed_slice(), level.inc()));
            }
            TocElem::Inputref {
                uri,
                title,
                id,
                children,
            } => {
                if let Ok(id) = id.parse::<Id>() {
                    let children = convert(state, children.into_boxed_slice(), level);
                    let (start, end) = get_counters(&children);
                    let start = if let Some(DynamicCounter::Static(start)) = start {
                        Some(start)
                    } else {
                        None
                    };
                    let end = if let Some(DynamicCounter::Static(end)) = end {
                        Some(end)
                    } else {
                        None
                    };
                    if let Some(InputrefStateGet {
                        counter,
                        doc_title,
                        slides,
                        paras,
                    }) = state.get_inputref.as_mut().and_then(|f| f(&id))
                    {
                        if let Some(start) = start {
                            state.todos.push(Todo::Counter {
                                sig: counter,
                                value: (start, end),
                            });
                        }
                        if let Some(ttl) = &title {
                            state.todos.push(Todo::Title {
                                sig: doc_title,
                                value: ttl.clone(),
                            });
                        }
                        state.todos.extend([
                            Todo::Slides {
                                sig: slides,
                                value: state.slides,
                            },
                            Todo::Para {
                                sig: paras,
                                value: state.paras.clone(),
                            },
                        ]);
                    }
                    let doc_title =
                        title.unwrap_or_else(|| uri.document_name().to_string().into_boxed_str());
                    state.inputrefs.insert(
                        id.clone(),
                        InputrefStateReady {
                            start_counter: start,
                            end_counter: end,
                            doc_title,
                            slides: state.slides,
                            paras: state.paras.clone(),
                        },
                    );
                    ret.push(FinalTocEntry::Inputref { id, children });
                }
            }
            TocElem::Section {
                title,
                uri,
                id,
                children,
            } => {
                state.counter = state.counter.inc_at(level);
                for c in state.counters {
                    if c.parent.is_some_and(|p| p <= level) {
                        state.paras.remove(&c.name);
                    }
                }
                if let Ok(id) = id.parse::<Id>() {
                    if let Some(DynamicCounter::Sig(s)) =
                        state.get_section.as_mut().and_then(|f| f(&id))
                    {
                        state.todos.push(Todo::Section {
                            sig: s,
                            value: state.counter,
                        });
                    }
                    ret.push(FinalTocEntry::Section {
                        level,
                        uri,
                        counter: DynamicCounter::Static(state.counter),
                        title: title.unwrap_or_default(),
                        id,
                        children: convert(state, children.into_boxed_slice(), level.inc()),
                    });
                }
            }
            TocElem::Paragraph { styles, kind } => {
                let counter = {
                    let mut ret = None;
                    let mut found = false;
                    for s in &styles {
                        if let Some(style) = state
                            .styles
                            .iter()
                            .find(|style| style.kind == kind && style.name.as_ref() == Some(s))
                        {
                            if let Some(cname) = style.counter.as_ref() {
                                ret = Some(cname);
                            }
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        ret = state
                            .styles
                            .iter()
                            .find(|style| style.kind == kind && style.name.is_none())
                            .and_then(|style| style.counter.as_ref());
                    }
                    ret
                };
                if let Some(counter) = counter {
                    if let Some(v) = state.paras.get_mut(counter) {
                        *v += 1;
                    } else {
                        state.paras.insert(counter.clone(), 1);
                    }
                }
            }
            TocElem::Slide => state.slides += 1,
            //TocElem::Paragraph { styles, kind } => (), // TODO: paragraphs
            //_ => (),
        }
    }
    ret
}

#[derive(Debug)]
struct InputrefStateGet {
    counter: RwSignal<Option<(SectionCounters, Option<SectionCounters>)>>,
    doc_title: RwSignal<Box<str>>,
    slides: RwSignal<CurrentSlide>,
    paras: RwSignal<ParagraphCountersI>,
}
impl InputrefStateGet {
    fn title(&self, at: SectionLevel, top: SectionLevel) -> InputrefTitle {
        InputrefTitle(either::Left(InputrefTitleI {
            at,
            top,
            counters: either::Right(self.counter),
            title: either::Right(self.doc_title.read_only()),
        }))
    }
}

#[derive(Debug, Clone)]
struct InputrefStateReady {
    start_counter: Option<SectionCounters>,
    end_counter: Option<SectionCounters>,
    doc_title: Box<str>,
    slides: u32,
    paras: rustc_hash::FxHashMap<Id, u32>,
}
impl InputrefStateReady {
    fn title(&self, at: SectionLevel, top: SectionLevel) -> InputrefTitle {
        self.start_counter.map_or_else(
            || InputrefTitle(either::Right(self.doc_title.clone())),
            |start| {
                InputrefTitle(either::Left(InputrefTitleI {
                    at,
                    top,
                    counters: either::Left(Some((start, self.end_counter))),
                    title: either::Left(self.doc_title.clone()),
                }))
            },
        )
    }
}

enum Todo {
    Counter {
        sig: RwSignal<Option<(SectionCounters, Option<SectionCounters>)>>,
        value: (SectionCounters, Option<SectionCounters>),
    },
    Title {
        sig: RwSignal<Box<str>>,
        value: Box<str>,
    },
    Section {
        sig: RwSignal<DynamicCounter>,
        value: SectionCounters,
    },
    Slides {
        sig: RwSignal<CurrentSlide>,
        value: u32,
    },
    Para {
        sig: RwSignal<ParagraphCountersI>,
        value: rustc_hash::FxHashMap<Id, u32>,
    },
}
impl Todo {
    fn go(self) {
        match self {
            Self::Counter { sig, value } => sig.set(Some(value)),
            Self::Title { sig, value } => sig.set(value),
            Self::Section { sig, value } => sig.set(DynamicCounter::Static(value)),
            Self::Slides { sig, value } => sig.set(CurrentSlide::Static(value)),
            Self::Para { sig, value } => sig.set(ParagraphCountersI::Static(value)),
        }
    }
}
