use crate::structure::DocumentStructure;
use crate::utils::actions::OneShot;
use ftml_ontology::narrative::elements::{paragraphs::ParagraphKind, sections::SectionLevel};
use ftml_uris::Id;
use leptos::prelude::*;
use leptos::wasm_bindgen;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(tag = "type")]
pub enum LogicalLevel {
    None,
    Section(SectionLevel),
    Paragraph,
    BeamerSlide,
}
impl ftml_js_utils::conversion::FromWasmBindgen for LogicalLevel {}
//#[cfg(feature = "typescript")]
impl wasm_bindgen::convert::TryFromJsValue for LogicalLevel {
    fn try_from_js_value(value: wasm_bindgen::JsValue) -> Result<Self, wasm_bindgen::JsValue> {
        serde_wasm_bindgen::from_value(value.clone()).map_err(|_| value)
    }
    fn try_from_js_value_ref(value: &wasm_bindgen::JsValue) -> Option<Self> {
        serde_wasm_bindgen::from_value(value.clone()).ok()
    }
}
impl LogicalLevel {
    pub fn into_view(self, capitalize: bool) -> impl IntoView {
        match (self, capitalize) {
            (Self::None, true) => "Document",
            (Self::None, _) => "document",
            (Self::Section(SectionLevel::Part), true) => "Part",
            (Self::Section(SectionLevel::Part), _) => "part",
            (Self::Section(SectionLevel::Chapter), true) => "Chapter",
            (Self::Section(SectionLevel::Chapter), _) => "chapter",
            (Self::Section(SectionLevel::Section), true) => "Section",
            (Self::Section(SectionLevel::Section), _) => "section",
            (Self::Section(SectionLevel::Subsection), true) => "Subsection",
            (Self::Section(SectionLevel::Subsection), _) => "subsection",
            (Self::Section(SectionLevel::Subsubsection), true) => "Subsubsection",
            (Self::Section(SectionLevel::Subsubsection), _) => "subsubsection",
            (Self::BeamerSlide, true) => "Slide",
            (Self::BeamerSlide, _) => "slide",
            (_, true) => "Paragraph",
            (_, _) => "paragraph",
        }
    }
    pub const fn title_class(self) -> &'static str {
        match self {
            Self::Section(l) => match l {
                SectionLevel::Part => "ftml-title-part",
                SectionLevel::Chapter => "ftml-title-chapter",
                SectionLevel::Section => "ftml-title-section",
                SectionLevel::Subsection => "ftml-title-subsection",
                SectionLevel::Subsubsection => "ftml-title-subsubsection",
                SectionLevel::Paragraph => "ftml-title-paragraph",
                SectionLevel::Subparagraph => "ftml-title-subparagraph",
            },
            Self::BeamerSlide => "ftml-title-slide",
            Self::Paragraph => "ftml-title-paragraph",
            Self::None => "ftml-title",
        }
    }
    pub const fn section_level(self) -> SectionLevel {
        match self {
            Self::Section(l) => l,
            _ => SectionLevel::Subparagraph,
        }
    }
}

impl Ord for LogicalLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::{Equal, Greater, Less};
        if self == other {
            return Equal;
        }
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::None, _) => Greater,
            (_, Self::None) => Less,
            (Self::Section(s1), Self::Section(s2)) => s1.cmp(s2),
            (Self::Section(_), _) => Greater,
            (_, Self::Section(_)) => Less,
            (Self::Paragraph, Self::BeamerSlide) => Greater,
            _ // (Self::BeamerSlide, Self::Paragraph)
                => Less,
        }
    }
}
impl PartialOrd for LogicalLevel {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CurrentCounters(StoredValue<CurrentSectionCounters>);

#[derive(Clone, Copy)]
pub(crate) enum CurrentSectionCounters {
    Static(SectionCounters),
    AfterInputref {
        since: SectionCounters,
        previous: RwSignal<Self>,
    },
}
impl CurrentSectionCounters {
    fn get_rec(self) -> SectionCounters {
        match self {
            Self::Static(s) => s,
            Self::AfterInputref { since, previous } => previous.get().get_rec() + since,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum CurrentSlide {
    Static(u32),
    AfterInputref {
        since: u32,
        previous: RwSignal<Self>,
    },
}
impl CurrentSlide {
    fn get(&self) -> u32 {
        match self {
            Self::Static(u) => *u,
            Self::AfterInputref { since, previous } => previous.get().get() + since,
        }
    }
}
impl CurrentCounters {
    pub fn init() {
        provide_context(Self(StoredValue::new(CurrentSectionCounters::Static(
            SectionCounters::default(),
        ))));
        provide_context(StoredValue::new(CurrentSlide::Static(0)));
        provide_context(ParagraphCounters::default());
    }
    pub fn inc(lvl: SectionLevel, id: &Id) -> DynamicCounter {
        let current: Self = expect_context();
        let paras: ParagraphCounters = expect_context();
        paras.reset(lvl);
        let r = match current.0.get_value() {
            // no inputrefs yet
            CurrentSectionCounters::Static(sections) => {
                let sections = sections.inc_at(lvl);
                current
                    .0
                    .set_value(CurrentSectionCounters::Static(sections));
                DynamicCounter::Static(sections)
            }
            CurrentSectionCounters::AfterInputref { since, previous } => {
                let since = since.inc_at(lvl);
                current
                    .0
                    .set_value(CurrentSectionCounters::AfterInputref { since, previous });
                let d = DynamicCounter::AfterInputref {
                    since,
                    previous: previous.read_only(),
                };
                let Some(toc) = with_context::<DocumentStructure, _>(|s| s.toc) else {
                    return d;
                };
                toc.get_section_counter(id, d)
            }
        };
        if lvl >= SectionLevel::Section {
            // counter will be reset at next Chapter (or other ancestor)
            // => "localize" inputref barriers
            provide_context(current);
        }
        r
    }
    #[allow(clippy::option_if_let_else)]
    pub fn inputref(expanded: OneShot, id: &Id) {
        let current: Self = expect_context();
        let slides: StoredValue<CurrentSlide> = expect_context();
        let paras: ParagraphCounters = expect_context();
        let old = current.0.get_value();
        let old_slides = slides.get_value();
        let old_stored = StoredValue::new(old);
        let old_slides_stored = StoredValue::new(old_slides);
        provide_context(Self(old_stored));
        provide_context(old_slides_stored);
        let previous = RwSignal::new(old);
        current.0.set_value(CurrentSectionCounters::AfterInputref {
            since: SectionCounters::default(),
            previous,
        });
        let toc = with_context::<DocumentStructure, _>(|s| s.toc);
        let nslides = if let Some((v, new_current, new_local)) =
            toc.and_then(|toc| toc.get_inputref_counters(id, old_slides, paras))
        {
            paras.0.update_value(|pi| *pi = new_current);
            provide_context(new_local);
            v
        } else {
            let previous_slides = RwSignal::new(old_slides);
            let mut old_paras = ParagraphCountersI::default();
            paras.0.update_value(|inner| {
                std::mem::swap(inner, &mut old_paras);
            });
            let previous_paras = RwSignal::new(old_paras.clone());
            paras.0.update_value(|p| {
                *p = ParagraphCountersI::AfterInputref {
                    previous: previous_paras,
                    since: rustc_hash::FxHashMap::default(),
                }
            });
            let old_paras = ParagraphCounters(StoredValue::new(old_paras));
            provide_context(old_paras);

            expanded.on_set(move || {
                previous.set(old_stored.get_value());
                previous_paras.set(old_paras.0.get_value());
                previous_slides.set(old_slides_stored.get_value());
            });
            CurrentSlide::AfterInputref {
                since: 0,
                previous: previous_slides,
            }
        };
        slides.set_value(nslides);
    }

    pub fn current() -> DynamicCounter {
        let current: Self = expect_context();
        match current.0.get_value() {
            CurrentSectionCounters::Static(sections) => DynamicCounter::Static(sections),
            CurrentSectionCounters::AfterInputref { since, previous } => {
                DynamicCounter::AfterInputref {
                    since,
                    previous: previous.read_only(),
                }
            }
        }
    }
    pub fn slide() -> SlideNumber {
        let current: StoredValue<CurrentSlide> = expect_context();
        match current.get_value() {
            // no inputrefs yet
            CurrentSlide::Static(ctr) => SlideNumber::Static(ctr),
            CurrentSlide::AfterInputref { since, previous } => SlideNumber::AfterInputref {
                since,
                previous: previous.read_only(),
            },
        }
    }
    pub fn slide_inc() {
        let current: StoredValue<CurrentSlide> = expect_context();
        match current.get_value() {
            // no inputrefs yet
            CurrentSlide::Static(old) => {
                current.set_value(CurrentSlide::Static(old + 1));
            }
            CurrentSlide::AfterInputref { since, previous } => {
                current.set_value(CurrentSlide::AfterInputref {
                    since: since + 1,
                    previous,
                });
            }
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) enum SlideNumber {
    Static(u32),
    AfterInputref {
        since: u32,
        previous: ReadSignal<CurrentSlide>,
    },
}
impl SlideNumber {
    pub fn into_view(self) -> AnyView {
        match self {
            Self::Static(u) => u.into_any(),
            Self::AfterInputref { since, previous } => {
                (move || previous.get().get() + since).into_any()
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum DynamicCounter {
    Static(SectionCounters),
    AfterInputref {
        since: SectionCounters,
        previous: ReadSignal<CurrentSectionCounters>,
    },
    Sig(RwSignal<Self>),
}
impl DynamicCounter {
    pub fn get(self) -> SectionCounters {
        match self {
            Self::Static(s) => s,
            Self::AfterInputref { since, previous } => previous.get().get_rec() + since,
            Self::Sig(s) => s.get().get(),
        }
    }
    pub fn into_view(self, at: SectionLevel, max_level: SectionLevel) -> impl IntoView {
        match self {
            Self::Static(c) => leptos::either::EitherOf3::A(c.into_view(at, max_level)),
            Self::AfterInputref { since, previous } => leptos::either::EitherOf3::B(move || {
                (previous.get().get_rec() + since).into_view(at, max_level)
            }),
            Self::Sig(s) => {
                leptos::either::EitherOf3::C(move || s.get().into_view(at, max_level).into_any())
            }
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub(crate) struct ParagraphCounters(pub(crate) StoredValue<ParagraphCountersI>);
#[derive(Clone, Debug)]
pub(crate) enum ParagraphCountersI {
    Static(rustc_hash::FxHashMap<Id, u32>),
    AfterInputref {
        previous: RwSignal<Self>,
        since: rustc_hash::FxHashMap<Id, either::Either<u32, u32>>,
    },
}
impl Default for ParagraphCountersI {
    fn default() -> Self {
        Self::Static(rustc_hash::FxHashMap::default())
    }
}
impl ParagraphCountersI {
    fn next_value(
        &mut self,
        counter: &Id,
    ) -> either::Either<u32, impl Fn() -> u32 + 'static + use<>> {
        match self {
            Self::Static(map) => either::Left(Self::next_static(counter, map)),
            Self::AfterInputref { previous, since } => Self::next_ipr(counter, since, *previous),
        }
    }
    fn get_value(&self, counter: &Id) -> either::Either<u32, impl Fn() -> u32 + 'static + use<>> {
        match self {
            Self::Static(map) => either::Left(Self::get_static(counter, map)),
            Self::AfterInputref { previous, since } => Self::get_ipr(counter, since, *previous),
        }
    }

    #[allow(clippy::option_if_let_else)]
    fn next_static(counter: &Id, map: &mut rustc_hash::FxHashMap<Id, u32>) -> u32 {
        if let Some(v) = map.get_mut(counter) {
            *v += 1;
            *v
        } else {
            map.insert(counter.clone(), 1);
            1
        }
    }

    #[allow(clippy::option_if_let_else)]
    fn get_static(counter: &Id, map: &rustc_hash::FxHashMap<Id, u32>) -> u32 {
        if let Some(v) = map.get(counter) {
            *v
        } else {
            0
        }
    }

    fn make_memo(v: u32, id: Id, previous: RwSignal<Self>) -> impl Fn() -> u32 {
        thread_local! {
            static COUNTER : std::cell::Cell<usize> = const{std::cell::Cell::new(0)};
        }
        let counter = COUNTER.get();
        COUNTER.set(counter + 1);
        move || match previous.with(|v| v.get_value(&id)) {
            either::Left(n) => n + v,
            either::Right(f) => {
                let n = f();
                n + v
            }
        }
    }

    fn next_ipr(
        cname: &Id,
        map: &mut rustc_hash::FxHashMap<Id, either::Either<u32, u32>>,
        previous: RwSignal<Self>,
    ) -> either::Either<u32, impl Fn() -> u32 + 'static + use<>> {
        match map.get_mut(cname) {
            Some(either::Left(v)) => {
                *v += 1;
                either::Left(*v)
            }
            Some(either::Right(v)) => {
                *v += 1;
                let v = *v;
                either::Right(Self::make_memo(v, cname.clone(), previous))
            }
            None => {
                map.insert(cname.clone(), either::Right(1));
                either::Right(Self::make_memo(1, cname.clone(), previous))
            }
        }
    }

    fn get_ipr(
        cname: &Id,
        map: &rustc_hash::FxHashMap<Id, either::Either<u32, u32>>,
        previous: RwSignal<Self>,
    ) -> either::Either<u32, impl Fn() -> u32 + 'static + use<>> {
        match map.get(cname) {
            Some(either::Left(v)) => either::Left(*v),
            Some(either::Right(v)) => either::Right(Self::make_memo(*v, cname.clone(), previous)),
            None => either::Right(Self::make_memo(0, cname.clone(), previous)),
        }
    }
}
impl ParagraphCounters {
    pub fn take(self) -> Self {
        let mut ret = ParagraphCountersI::default();
        self.0.update_value(|s| std::mem::swap(s, &mut ret));
        Self(StoredValue::new(ret))
    }
    fn next_value(
        self,
        kind: ParagraphKind,
        styles: &[Id],
    ) -> Option<(Id, either::Either<u32, impl Fn() -> u32 + 'static + use<>>)> {
        let style_decls =
            with_context::<DocumentStructure, _>(|d| d.styles).expect("Not in a document context");
        style_decls.with_value(|style_decls| {
            let counter = {
                let mut ret = None;
                let mut found = false;
                for s in styles {
                    if let Some(style) = style_decls
                        .0
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
                    ret = style_decls
                        .0
                        .iter()
                        .find(|style| style.kind == kind && style.name.is_none())
                        .and_then(|style| style.counter.as_ref());
                }
                ret
            };
            counter.and_then(|ctr| {
                let mut ret = None;
                self.0.update_value(|slf| {
                    ret = Some((ctr.clone(), slf.next_value(ctr)));
                });
                ret
            })
        })
        //.map(|(id, v)| (id, v.map_right(|f| Memo::new(move |_| f()))))
    }

    #[allow(clippy::option_if_let_else)]
    pub fn get_style(
        self,
        kind: ParagraphKind,
        styles: &[Id],
    ) -> either::Either<String, Memo<String>> {
        self.next_value(kind, styles).map_or_else(
            || either::Left(String::new()),
            |(id, v)| match v {
                either::Left(v) => either::Left(format!("counter-set:ftml-{id} {v};")),
                either::Right(f) => either::Right(Memo::new(move |_| {
                    format!("counter-set:ftml-{id} {};", f())
                })),
            },
        )
    }

    pub(crate) fn reset(self, lvl: SectionLevel) {
        use ParagraphCountersI as PI;
        let styles =
            with_context::<DocumentStructure, _>(|d| d.styles).expect("Not in a document context");
        self.0.update_value(|slf| match slf {
            PI::Static(map) => styles.with_value(|(_, counters)| {
                for c in counters {
                    if c.parent.is_some_and(|p| p <= lvl) {
                        map.remove(&c.name);
                    }
                }
            }),
            PI::AfterInputref { since, .. } => {
                styles.with_value(|(_, counters)| {
                    for c in counters {
                        if c.parent.is_some_and(|p| p <= lvl) {
                            since.insert(c.name.clone(), either::Left(0));
                        }
                    }
                });
            }
        });
    }
}

/// part, chapter, section, subsection, subsubsection, paragraph
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub(crate) struct SectionCounters {
    pub values: [u16; 7],
}

impl SectionCounters {
    pub fn inc_at(mut self, lvl: SectionLevel) -> Self {
        let idx: u8 = lvl.into();
        self.values[idx as usize] += 1;
        if idx == 0 {
            for i in 2..=6 {
                self.values[i] = 0;
            }
        } else {
            for i in ((idx + 1) as usize)..=6 {
                self.values[i] = 0;
            }
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

    pub fn into_string(self, at: SectionLevel, max_level: SectionLevel) -> String {
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
                let mut ret = if max_level >= SectionLevel::Chapter {
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

    #[inline]
    pub fn into_view(self, at: SectionLevel, max_level: SectionLevel) -> impl IntoView {
        self.into_string(at, max_level)
    }
}
impl std::fmt::Display for SectionCounters {
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

impl std::ops::Add<Self> for SectionCounters {
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

impl std::ops::AddAssign<Self> for SectionCounters {
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
