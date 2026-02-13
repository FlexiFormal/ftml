use crate::{
    FtmlViews,
    counters::{CurrentCounters, DynamicCounter, LogicalLevel, ParagraphCounters},
    document::CurrentUri,
    extractor::DomExtractor,
    toc::{FinalTocEntry, InputrefTitle, TocSource, TocStyle},
    utils::{
        actions::{OneShot, SetOneShotDone},
        local_cache::SendBackend,
    },
};
use ftml_ontology::narrative::{
    documents::{DocumentCounter, DocumentStyle},
    elements::{SectionLevel, paragraphs::ParagraphKind},
};
use ftml_uris::{DocumentElementUri, DocumentUri, Id};
use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct SectionInfo {
    pub uri: DocumentElementUri,
    pub id: Id,
    lvl: LogicalLevel,
    counter: DynamicCounter,
}

impl SectionInfo {
    pub const fn level(&self) -> LogicalLevel {
        self.lvl
    }
    pub fn style(&self) -> impl leptos::tachys::html::style::IntoStyle + 'static + use<> {
        fn mk_string(sects: [u16; 7], lvl: SectionLevel) -> String {
            match lvl {
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
                _ => String::new(),
            }
        }
        // invariant: section implies top_level is defined
        let lvl = match self.lvl {
            LogicalLevel::None => {
                with_context::<DocumentStructure, _>(|ds| ds.top_level.get_value())
                    .flatten()
                    .unwrap_or(SectionLevel::Part)
            }
            LogicalLevel::Section(s) => s,
            _ => SectionLevel::Subparagraph,
        };
        match self.counter {
            DynamicCounter::Static(s) => utils::Style(either::Left(mk_string(s.values, lvl))),
            dc => utils::Style(either::Right(Memo::new(move |_| {
                mk_string(dc.get().values, lvl)
            }))),
        }
    }
    pub fn class(&self) -> impl leptos::tachys::html::class::IntoClass + 'static + use<> {
        let lvl = match self.lvl {
            LogicalLevel::None => {
                with_context::<DocumentStructure, _>(|ds| ds.top_level.get_value())
                    .flatten()
                    .unwrap_or(SectionLevel::Part)
            }
            LogicalLevel::Section(s) => s,
            _ => SectionLevel::Subparagraph,
        };
        match lvl {
            SectionLevel::Part => "ftml-part",
            SectionLevel::Chapter => "ftml-chapter",
            SectionLevel::Section => "ftml-section",
            SectionLevel::Subsection => "ftml-subsection",
            SectionLevel::Subsubsection => "ftml-subsubsection",
            SectionLevel::Paragraph => "ftml-paragraph",
            SectionLevel::Subparagraph => "ftml-subparagraph",
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
    title: InputrefTitle,
}

#[derive(Copy, Clone)]
struct InInputref(bool);

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
    #[inline]
    pub fn title<V: crate::FtmlViews>(&self) -> impl IntoView + use<V> {
        self.title.as_view::<V>()
    }
}

#[derive(Clone)]
pub(crate) struct CurrentId(pub(crate) Option<Id>);

#[derive(Debug)]
pub struct DocumentStructure {
    top_level: StoredValue<Option<SectionLevel>>,
    pub(crate) toc: crate::toc::Toc,
    has_top: StoredValue<bool>,
    ids: StoredValue<rustc_hash::FxHashMap<Id, SectionOrInputref>>,
    redo: StoredValue<Option<String>>,
    pub(crate) styles: StoredValue<(Vec<DocumentStyle>, Vec<DocumentCounter>)>,
}

impl DocumentStructure {
    pub fn set<Be: SendBackend>(source: TocSource) {
        let styles = StoredValue::new((Vec::new(), Vec::new()));
        let toc = crate::toc::Toc::new::<Be>(source, styles);
        Self::set_i(toc, styles);
    }
    pub fn set_empty() {
        Self::set_i(
            crate::toc::Toc::None,
            StoredValue::new((Vec::new(), Vec::new())),
        );
    }
    fn set_i(
        toc: crate::toc::Toc,
        styles: StoredValue<(Vec<DocumentStyle>, Vec<DocumentCounter>)>,
    ) {
        provide_context(Self {
            top_level: StoredValue::new(None),
            toc,
            has_top: StoredValue::new(false),
            ids: StoredValue::new(rustc_hash::FxHashMap::default()),
            redo: StoredValue::new(None),
            styles,
        });
        provide_context(CurrentId(None));
        provide_context(LogicalLevel::None);
        CurrentCounters::init();
    }

    pub fn toc_style() -> TocStyle {
        with_context::<Self, _>(|s| s.toc.style()).unwrap_or_default()
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

    fn force_top(&self) -> SectionLevel {
        if self.has_top.get_value() {
            // SAFETY: has_top.get() implies self.top_level.get().is_some()
            return unsafe { self.top_level.get_value().unwrap_unchecked() };
        }
        let top = self.top_level.get_value().unwrap_or_else(|| {
            self.top_level.set_value(Some(SectionLevel::Part));
            SectionLevel::Part
        });
        self.has_top.set_value(true);

        let (counters, styles) = expect_context::<RwSignal<DomExtractor>>()
            .with_untracked(|ext| (ext.state.counters.clone(), ext.state.styles.clone()));
        //leptos::logging::log!("Finalized styles:{styles:#?}\ncounters:{counters:#?}");
        self.styles.update_value(|(st, ctr)| {
            *st = styles;
            *ctr = counters;
        });
        self.toc.init(top);
        top
    }

    fn init_top() {
        with_context::<Self, _>(|slf| {
            let _ = slf.force_top();
        });
    }

    pub(crate) fn set_max_level(lvl: SectionLevel) {
        with_context::<Self, _>(|slf| {
            if !slf.has_top.get_value() {
                slf.top_level.set_value(Some(lvl));
            }
        });
    }

    pub(crate) fn new_section(uri: DocumentElementUri) -> SectionInfo {
        let id = Self::new_id(uri.name().last());
        provide_context(CurrentUri(uri.clone().into()));
        let current_level = expect_context::<LogicalLevel>();
        let (new_level, sect_level) = with_context::<Self, _>(|slf| {
            let top = slf.force_top();
            let new_level = if let LogicalLevel::Section(s) = current_level {
                LogicalLevel::Section(s.inc())
            } else if current_level == LogicalLevel::None {
                LogicalLevel::Section(top)
            } else {
                current_level
            };

            slf.ids.update_value(|ids| {
                ids.insert(id.clone(), SectionOrInputref::Section);
            });

            let sect_level = match new_level {
                LogicalLevel::None => top,
                LogicalLevel::Section(s) => s,
                _ => SectionLevel::Subparagraph,
            };
            (new_level, sect_level)
        })
        .expect("document context missing");
        //tracing::warn!("New section at {new_level:?}");
        provide_context(new_level);
        let counter = CurrentCounters::inc(sect_level, &id);
        SectionInfo {
            uri,
            id,
            lvl: new_level,
            counter,
        }
    }

    pub(crate) fn skip_section() {
        let current_level = expect_context::<LogicalLevel>();
        let (new_level, top) = with_context::<Self, _>(|slf| {
            let top = slf.force_top();
            (
                if let LogicalLevel::Section(s) = current_level {
                    LogicalLevel::Section(s.inc())
                } else if current_level == LogicalLevel::None {
                    LogicalLevel::Section(top)
                } else {
                    current_level
                },
                top,
            )
        })
        .expect("document context missing");
        let paras: ParagraphCounters = expect_context();
        let sect_level = match new_level {
            LogicalLevel::None => top,
            LogicalLevel::Section(s) => s,
            _ => SectionLevel::Subparagraph,
        };
        paras.reset(sect_level);
        provide_context(new_level);
    }

    /// ### Panics
    pub fn render_toc<
        V: FtmlViews,
        Cont: Fn(String, &DocumentElementUri, AnyView, Option<AnyView>) -> AnyView
            + Clone
            + Send
            + 'static,
        OnRender: Fn(&[FinalTocEntry]) + Clone + Send + 'static,
        FallBack: Fn() -> AnyView + Clone + Send + 'static,
    >(
        cont: Cont,
        on_render: OnRender,
        fallback: FallBack,
    ) -> impl IntoView {
        let (max, toc) = with_context::<Self, _>(|slf| (slf.top_level, slf.toc))
            .expect("Not in a document context!");
        toc.render_toc::<V, _, _, _>(cont, on_render, fallback, max)
    }

    pub(crate) fn insert_section_title(get: impl FnOnce() -> String) -> &'static str {
        let lvl = use_context::<LogicalLevel>().unwrap_or(LogicalLevel::None);
        let cls = lvl.title_class();
        let (top, toc) = with_context::<Self, _>(|slf| (slf.force_top(), slf.toc))
            .expect("Not in a document context!");
        toc.set_section_title(top, get);
        cls
    }

    pub(crate) fn slide_inc() {
        Self::init_top();
        CurrentCounters::slide_inc();
        provide_context(LogicalLevel::BeamerSlide);
    }

    pub(crate) fn new_inputref(uri: DocumentElementUri, target: DocumentUri) -> Inputref {
        let id = Self::new_id(uri.name().last());
        let (os, done) = OneShot::new();
        //let tocstyle = with_context::<Self, _>(|slf| slf.toc.style()).unwrap_or_default();
        let level = use_context::<LogicalLevel>().and_then(|lvl| match lvl {
            LogicalLevel::Section(s) => Some(s),
            LogicalLevel::BeamerSlide | LogicalLevel::Paragraph => Some(SectionLevel::Subparagraph),
            LogicalLevel::None => None,
        });
        provide_context(InInputref(true));
        let (toc, top) = with_context::<Self, _>(|slf| {
            let top = slf.force_top();
            slf.ids.update_value(|ids| {
                ids.insert(id.clone(), SectionOrInputref::Inputref(os));
            });
            (slf.toc, top)
        })
        .expect("Not in a document context");
        let level = level.unwrap_or(top);
        let title = toc.inputref_title(&target, &id, level, top);
        CurrentCounters::inputref(os, &id);

        Inputref {
            uri,
            target,
            id,
            replace: os,
            done,
            //title_counter: ipr,
            title,
        }
    }

    pub fn in_inputref() -> bool {
        use_context::<InInputref>().is_some_and(|b| b.0)
    }

    pub(crate) fn get_para(kind: ParagraphKind, styles: &[Id]) -> (utils::Style, Option<String>) {
        Self::init_top();
        provide_context(LogicalLevel::Paragraph);
        let paras: ParagraphCounters = expect_context();
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
        (utils::Style(paras.get_style(kind, styles)), cls)
    }

    pub(crate) fn get_problem(styles: &[Id]) -> (Memo<String>, String) {
        Self::init_top();
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

    #[allow(clippy::missing_const_for_fn)]
    pub fn navigate_to(_id: &str) {
        #[cfg(any(feature = "csr", feature = "hydrate"))]
        {
            #[allow(clippy::used_underscore_binding)]
            let Ok(id) = _id.parse::<Id>() else { return };
            with_context::<Self, _>(|slf| {
                tracing::trace!("Looking for #{id}");
                let mut curr = id.clone();
                slf.ids.with_value(move |ids| {
                    loop {
                        match ids.get(&curr) {
                            None => {
                                tracing::debug!(
                                    "navigation id {curr} not known (yet)\n{:?}!",
                                    slf.ids
                                );
                                slf.redo.update_value(|r| *r = Some(id.to_string()));
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
            });
        }
    }

    pub(crate) fn retry() {
        let v = with_context::<Self, _>(|slf| {
            let mut ret = None;
            slf.redo.update_value(|v| ret = v.take());
            ret
        })
        .flatten();
        if let Some(s) = v {
            Self::navigate_to(&s);
        }
    }
}

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

pub mod utils {
    use std::hint::unreachable_unchecked;
    use std::sync::Arc;

    use leptos::prelude::*;
    use leptos::tachys::html::style::IntoStyle as IS;
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Style(pub either::Either<String, Memo<String>>);
    macro_rules! asIs {
        ($i:ident) => {either::Either<<String as IS>::$i,<Memo<String> as IS>::$i>}
    }
    macro_rules! asAs {
        ($i:ident) => {either::Either<<Arc<str> as IS>::$i,<Memo<String> as IS>::$i>}
    }
    #[derive(Clone)]
    pub struct ArcStyle(either::Either<Arc<str>, Memo<String>>);
    impl IS for ArcStyle {
        type AsyncOutput = Self;
        type State = asAs!(State);
        type Cloneable = Self;
        type CloneableOwned = Self;
        fn to_html(self, style: &mut String) {
            match self.0 {
                either::Either::Left(s) => IS::to_html(s, style),
                either::Either::Right(s) => IS::to_html(s, style),
            }
        }
        fn hydrate<const FROM_SERVER: bool>(
            self,
            el: &leptos::tachys::renderer::types::Element,
        ) -> Self::State {
            self.0.map_either(
                |s| IS::hydrate::<FROM_SERVER>(s, el),
                |s| IS::hydrate::<FROM_SERVER>(s, el),
            )
        }
        fn build(self, el: &leptos::tachys::renderer::types::Element) -> Self::State {
            self.0
                .map_either(|s| IS::build(s, el), |s| IS::build(s, el))
        }
        fn rebuild(self, state: &mut Self::State) {
            match (self.0, state) {
                (either::Left(sl), either::Left(st)) => IS::rebuild(sl, st),
                (either::Right(sl), either::Right(st)) => IS::rebuild(sl, st),
                // SAFETY: left/right used consistently
                _ => unsafe { unreachable_unchecked() },
            }
        }
        fn into_cloneable(self) -> Self::Cloneable {
            self
        }
        fn into_cloneable_owned(self) -> Self::CloneableOwned {
            self
        }
        fn dry_resolve(&mut self) {
            self.0.as_mut().map_either(IS::dry_resolve, IS::dry_resolve);
        }
        async fn resolve(self) -> Self::AsyncOutput {
            self
        }
        fn reset(state: &mut Self::State) {
            state
                .as_mut()
                .map_either(<Arc<str> as IS>::reset, <Memo<String> as IS>::reset);
        }
    }

    impl IS for Style {
        type AsyncOutput = Self;
        type State = asIs!(State);
        type Cloneable = ArcStyle;
        type CloneableOwned = ArcStyle;
        fn to_html(self, style: &mut String) {
            match self.0 {
                either::Either::Left(s) => IS::to_html(s, style),
                either::Either::Right(s) => IS::to_html(s, style),
            }
        }
        fn hydrate<const FROM_SERVER: bool>(
            self,
            el: &leptos::tachys::renderer::types::Element,
        ) -> Self::State {
            self.0.map_either(
                |s| IS::hydrate::<FROM_SERVER>(s, el),
                |s| IS::hydrate::<FROM_SERVER>(s, el),
            )
        }
        fn build(self, el: &leptos::tachys::renderer::types::Element) -> Self::State {
            self.0
                .map_either(|s| IS::build(s, el), |s| IS::build(s, el))
        }
        fn rebuild(self, state: &mut Self::State) {
            match (self.0, state) {
                (either::Left(sl), either::Left(st)) => IS::rebuild(sl, st),
                (either::Right(sl), either::Right(st)) => IS::rebuild(sl, st),
                // SAFETY: left/right used consistently
                _ => unsafe { unreachable_unchecked() },
            }
        }
        fn into_cloneable(self) -> Self::Cloneable {
            ArcStyle(self.0.map_either(IS::into_cloneable, IS::into_cloneable))
        }
        fn into_cloneable_owned(self) -> Self::CloneableOwned {
            ArcStyle(
                self.0
                    .map_either(IS::into_cloneable_owned, IS::into_cloneable_owned),
            )
        }
        fn dry_resolve(&mut self) {
            self.0.as_mut().map_either(IS::dry_resolve, IS::dry_resolve);
        }
        async fn resolve(self) -> Self::AsyncOutput {
            self
        }
        fn reset(state: &mut Self::State) {
            state
                .as_mut()
                .map_either(<String as IS>::reset, <Memo<String> as IS>::reset);
        }
    }
}

#[derive(Debug, Clone)]
pub enum SectionOrInputref {
    Section,
    Inputref(OneShot),
}
