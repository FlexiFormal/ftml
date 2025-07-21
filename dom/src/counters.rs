use crate::{
    document::{CurrentTOC, DocumentState},
    toc::TOCElem,
};
use ftml_ontology::narrative::elements::{paragraphs::ParagraphKind, sections::SectionLevel};
use ftml_uris::{DocumentUri, Id};
use leptos::prelude::*;
use smallvec::SmallVec;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(tag = "type")]
pub enum LogicalLevel {
    None,
    Section(SectionLevel),
    Paragraph,
    BeamerSlide,
}

#[derive(Debug, Clone)]
pub struct SectionCounters {
    pub current: LogicalLevel,
    pub max: SectionLevel,
    sections: SmartCounter<AllSections>,
    initialized: RwSignal<bool>,
    counters: RwSignal<Vec<(Id, SmartCounter<u16>)>>,
    resets: RwSignal<Vec<(SectionLevel, Vec<Id>)>>,
    #[allow(clippy::type_complexity)]
    for_paras: RwSignal<Vec<(ParagraphKind, Option<Id>, Option<Id>)>>,
    slides: SmartCounter<u32>,
}

impl Default for SectionCounters {
    #[inline]
    fn default() -> Self {
        Self {
            current: LogicalLevel::None,
            max: SectionLevel::Part,
            sections: SmartCounter::default(),
            counters: RwSignal::new(Vec::new()),
            resets: RwSignal::new(Vec::new()),
            for_paras: RwSignal::new(Vec::new()),
            initialized: RwSignal::new(false),
            slides: SmartCounter::default(),
        }
    }
}

/// part, chapter, section, subsection, subsubsection, paragraph
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
struct AllSections(pub [u16; 7]);
impl std::fmt::Display for AllSections {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{} {} {} {} {} {} {}]",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6]
        )
    }
}

#[derive(Clone)]
struct Cutoff<N: CounterTrait> {
    previous: Option<std::sync::Arc<Cutoff<N>>>,
    since: N,
    set: RwSignal<N>,
}

#[derive(Clone, Default, Copy)]
struct SmartCounter<N: CounterTrait>(RwSignal<SmartCounterI<N>>);
impl<N: CounterTrait> std::fmt::Debug for SmartCounter<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
            .with_untracked(|s| f.debug_struct("SmartCounter").field("inner", s).finish())
    }
}

#[derive(Debug, Clone, Default)]
struct SmartCounterI<N: CounterTrait> {
    cutoff: Option<Cutoff<N>>,
    since: N,
}
impl<N: CounterTrait> SmartCounterI<N> {
    fn get(&self) -> N {
        self.cutoff
            .as_ref()
            .map_or(self.since, |cutoff| cutoff.get() + self.since)
    }
}

trait CounterTrait:
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
impl CounterTrait for u16 {
    fn one() -> Self {
        1
    }
}
impl CounterTrait for u32 {
    fn one() -> Self {
        1
    }
}

impl std::ops::Add<SectionLevel> for AllSections {
    type Output = Self;
    fn add(self, rhs: SectionLevel) -> Self::Output {
        let idx: u8 = rhs.into();
        let mut s = Self::default();
        s.0[idx as usize] = 1;
        self + s
    }
}

impl std::ops::Add<Self> for AllSections {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut changed = false;
        Self([
            {
                if rhs.0[0] > 0 {
                    changed = true;
                }
                self.0[0] + rhs.0[0]
            },
            {
                if rhs.0[1] > 0 {
                    changed = true;
                }
                self.0[1] + rhs.0[1]
            },
            {
                if changed {
                    0
                } else {
                    if rhs.0[2] > 0 {
                        changed = true;
                    }
                    self.0[2] + rhs.0[2]
                }
            },
            {
                if changed {
                    0
                } else {
                    if rhs.0[3] > 0 {
                        changed = true;
                    }
                    self.0[3] + rhs.0[3]
                }
            },
            {
                if changed {
                    0
                } else {
                    if rhs.0[4] > 0 {
                        changed = true;
                    }
                    self.0[4] + rhs.0[4]
                }
            },
            {
                if changed {
                    0
                } else {
                    if rhs.0[5] > 0 {
                        changed = true;
                    }
                    self.0[5] + rhs.0[5]
                }
            },
            { if changed { 0 } else { self.0[6] + rhs.0[6] } },
        ])
    }
}

impl std::ops::AddAssign<Self> for AllSections {
    fn add_assign(&mut self, rhs: Self) {
        let mut changed = rhs.0[0] > 0;
        self.0[0] += rhs.0[0];
        if rhs.0[1] > 0 {
            changed = true;
        }
        self.0[1] += rhs.0[1];
        if changed {
            self.0[2] = 0;
        } else {
            if rhs.0[2] > 0 {
                changed = true;
            }
            self.0[2] += rhs.0[2];
        }
        if changed {
            self.0[3] = 0;
        } else {
            if rhs.0[3] > 0 {
                changed = true;
            }
            self.0[3] += rhs.0[3];
        }
        if changed {
            self.0[4] = 0;
        } else {
            if rhs.0[4] > 0 {
                changed = true;
            }
            self.0[4] += rhs.0[4];
        }
        if changed {
            self.0[5] = 0;
        } else {
            if rhs.0[5] > 0 {
                changed = true;
            }
            self.0[5] += rhs.0[5];
        }
        if changed {
            self.0[6] = 0;
        } else {
            self.0[6] += rhs.0[6];
        }
    }
}

impl CounterTrait for AllSections {
    fn one() -> Self {
        panic!("That's not how sectioning works")
    }
}

impl SmartCounter<AllSections> {
    fn inc_at(&self, lvl: SectionLevel) {
        let idx: u8 = lvl.into();
        let mut s = AllSections::default();
        s.0[idx as usize] = 1;
        self.0
            .update_untracked(|SmartCounterI { since, .. }| *since += s);
    }
}

impl<N: CounterTrait> Cutoff<N> {
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
impl<N: CounterTrait> std::fmt::Debug for Cutoff<N> {
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

impl<N: CounterTrait> SmartCounter<N> {
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
    /*
    fn get_untracked(&self) -> N {
        self.0.with_untracked(|SmartCounterI { cutoff, since }| {
            cutoff.as_ref().map_or(*since, |c| c.get() + *since)
        })
    }

    fn set_cutoff(&self, v: N) {
        self.0.update_untracked(|SmartCounterI { cutoff, .. }| {
            if let Some(c) = cutoff.as_ref() {
                c.set.set(v);
            }
        });
    }
     */

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

impl SectionCounters {
    fn init_paras(&self) {
        if self.initialized.get_untracked() {
            return;
        }
        self.initialized.update_untracked(|b| *b = true);
        let mut counters = Vec::default();
        let mut resets = Vec::<(SectionLevel, Vec<Id>)>::default();
        let mut for_paras = Vec::default();
        DocumentState::with_styles_untracked(|ctrs, styles| {
            for c in ctrs {
                tracing::trace!("Doing {c:?}");
                counters.push((c.name.clone(), SmartCounter::default()));
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
                for_paras.push((stl.kind, stl.name.clone(), stl.counter.clone()));
            }
        });
        self.counters.update_untracked(|p| *p = counters);
        self.resets.update_untracked(|p| *p = resets);
        self.for_paras.update_untracked(|p| *p = for_paras);
    }

    #[inline]
    pub const fn current_level(&self) -> LogicalLevel {
        self.current
    }

    pub fn next_section(&mut self) -> (Option<Memo<String>>, Option<&'static str>) {
        self.init_paras();
        let lvl = if let LogicalLevel::Section(s) = self.current {
            s.inc()
        } else if self.current == LogicalLevel::None {
            self.max
        } else {
            return (Some(Memo::new(|_| "display:content;".into())), None);
        };
        tracing::trace!("New section at level {lvl:?}");
        self.set_section(lvl);
        let sections = self.sections.0.get_untracked();
        match lvl {
            SectionLevel::Part => (
                Some(Memo::new(move |_| {
                    let sects = sections.get().0;
                    format!("counter-set:ftml-part {};", sects[0])
                })),
                Some("ftml-part"),
            ),
            SectionLevel::Chapter => (
                Some(Memo::new(move |_| {
                    let sects = sections.get().0;
                    format!(
                        "counter-set:ftml-part {} ftml-chapter {}",
                        sects[0], sects[1]
                    )
                })),
                Some("ftml-chapter"),
            ),
            SectionLevel::Section => (
                Some(Memo::new(move |_| {
                    let sects = sections.get().0;
                    format!(
                        "counter-set:ftml-part {} ftml-chapter {} ftml-section {}",
                        sects[0], sects[1], sects[2]
                    )
                })),
                Some("ftml-section"),
            ),
            SectionLevel::Subsection => (
                Some(Memo::new(move |_| {
                    let sects = sections.get().0;
                    format!(
                        "counter-set:ftml-part {} ftml-chapter {} ftml-section {} ftml-subsection {}",
                        sects[0], sects[1], sects[2], sects[3],
                    )
                })),
                Some("ftml-subsection"),
            ),
            SectionLevel::Subsubsection => (
                Some(Memo::new(move |_| {
                    let sects = sections.get().0;
                    format!(
                        "counter-set:ftml-part {} ftml-chapter {} ftml-section {} ftml-subsection {} ftml-subsubsection {}",
                        sects[0], sects[1], sects[2], sects[3], sects[4],
                    )
                })),
                Some("ftml-subsubsection"),
            ),
            SectionLevel::Paragraph => (None, Some("ftml-paragraph")),
            SectionLevel::Subparagraph => (None, Some("ftml-subparagraph")),
        }
    }

    pub fn set_section(&mut self, lvl: SectionLevel) {
        self.init_paras();
        self.sections.inc_at(lvl);
        self.resets.with_untracked(|rs| {
            for (l, r) in rs {
                if *l >= lvl {
                    for n in r {
                        self.counters.with_untracked(|c| {
                            if let Some((_, c)) = c.iter().find(|(i, _)| i == n) {
                                c.reset();
                            }
                        });
                    }
                }
            }
        });
        self.current = LogicalLevel::Section(lvl);
    }

    fn get_counter(
        all: &[(ParagraphKind, Option<Id>, Option<Id>)],
        kind: ParagraphKind,
        styles: &[Id],
    ) -> Option<Id> {
        styles
            .iter()
            .rev()
            .find_map(|s| {
                all.iter().find_map(|(k, n, v)| {
                    if *k == kind && n.as_ref().is_some_and(|n| *n == *s) {
                        Some(v.as_ref())
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| {
                all.iter()
                    .find(|(k, s, _)| s.is_none() && *k == kind)
                    .and_then(|(_, _, o)| o.as_ref())
            })
            .cloned()
    }

    /// ### Panics
    /// Outside of a document context
    pub fn get_para(&mut self, kind: ParagraphKind, styles: &[Id]) -> Memo<String> {
        self.init_paras();
        self.current = LogicalLevel::Paragraph;
        let cnt = self
            .for_paras
            .with_untracked(|all_styles| Self::get_counter(all_styles, kind, styles));
        if let Some(cntname) = cnt {
            let cnt = self.counters.with_untracked(|cntrs| {
                *cntrs
                    .iter()
                    .find(|(a, _)| *a == cntname)
                    .map(|(_, r)| r)
                    .expect("counter does not exist; this is a bug")
            });
            cnt.inc_memo(move |i| format!("counter-set:ftml-{cntname} {i};"))
        } else {
            Memo::new(|_| String::new())
        }
    }

    pub fn get_problem(&mut self, _styles: &[Id]) -> Memo<String> {
        self.init_paras();
        self.current = LogicalLevel::Paragraph;
        Memo::new(|_| String::new())
    }

    pub fn get_slide() -> Memo<u32> {
        let counters: Self = expect_context();
        counters.init_paras();
        counters.slides.memo(|i| i)
    }
    pub fn slide_inc() -> Self {
        let mut counters: Self = expect_context();
        counters.init_paras();
        counters.slides.inc();
        counters.current = LogicalLevel::BeamerSlide;
        counters
    }

    /// ### Panics
    /// Outside of a document context
    pub fn inputref(uri: DocumentUri, id: String) -> Self {
        let mut counters: Self = expect_context();
        counters.init_paras();

        //tracing::warn!("inputref: {uri}@{id}");

        let old_slides = counters.slides; //.0.get_untracked();
        counters.slides = counters.slides.split();
        let old_slides = old_slides
            .0
            .get_untracked()
            .cutoff
            .expect("slides cutoff should be set; this is a bug")
            .set;

        let old_sections = counters.sections;
        counters.sections = counters.sections.split();
        let old_sections = old_sections
            .0
            .get_untracked()
            .cutoff
            .expect("section cutoff should be set; this is a bug")
            .set;

        let mut new_paras = Vec::new();

        let old_paras = counters.counters.with_untracked(|v| {
            v.iter()
                .map(|(n, e)| {
                    //leptos::logging::log!("Cloning {n}");
                    let r = *e;
                    let since = r.0.update_untracked(|e| {
                        let r = e.since;
                        e.since = 0;
                        r
                    });
                    new_paras.push((n.clone(), e.split()));
                    (
                        n.clone(),
                        r.0.get_untracked()
                            .cutoff
                            .expect("paragraph cutoff should be set; this is a bug")
                            .set,
                        since,
                    )
                })
                .collect::<Vec<_>>()
        });
        counters.counters = RwSignal::new(new_paras);

        let ctw = expect_context::<RwSignal<CurrentTOC>>();
        let uricl = uri.clone();
        let idcl = id.clone();
        let children = Memo::new(move |_| {
            let uri = &uricl;
            let id = &idcl;
            ctw.with(|v| {
                if let Some(v) = v.iter_dfs() {
                    for e in v {
                        if let TOCElem::Inputref {
                            uri: u,
                            id: i,
                            children: chs,
                            ..
                        } = e
                        {
                            if u == uri && i == id {
                                return Some(chs.clone());
                            }
                        }
                    }
                }
                None
            })
        });

        let current = counters.current;
        let max = counters.max;
        let para_map = counters.for_paras;

        Effect::new(move || {
            children.with(|ch| {
                if let Some(ch) = ch.as_ref() {
                    tracing::trace!("Updating {uri}@{id}");
                    para_map.with_untracked(|m| {
                        update(ch, current, max, &old_slides, &old_sections, &old_paras, m);
                    });
                }
            });
        });

        tracing::trace!("Returning {counters:?}");

        counters
    }
}

fn update(
    ch: &[TOCElem],
    mut current: LogicalLevel,
    max: SectionLevel,
    old_slides: &RwSignal<u32>,
    old_sections: &RwSignal<AllSections>,
    old_paras: &[(Id, RwSignal<u16>, u16)],
    para_map: &[(ParagraphKind, Option<Id>, Option<Id>)],
) {
    let mut curr = ch.iter();
    let mut stack = SmallVec::<_, 4>::new();

    let mut n_slides = 0;
    let mut n_sections = AllSections::default();
    let mut n_counters = old_paras
        .iter()
        .map(|(n, _, i)| (n.clone(), *i))
        .collect::<Vec<_>>();

    tracing::trace!("Updating inputref: {ch:?} in level {current:?}");

    loop {
        if let Some(c) = curr.next() {
            match c {
                TOCElem::Slide => n_slides += 1,
                TOCElem::SkippedSection { children } => {
                    let lvl = if let LogicalLevel::Section(s) = current {
                        s.inc()
                    } else if current == LogicalLevel::None {
                        max
                    } else {
                        continue;
                    };
                    let old = std::mem::replace(&mut current, LogicalLevel::Section(lvl));
                    stack.push((std::mem::replace(&mut curr, children.iter()), old));
                }
                TOCElem::Section { children, .. } => {
                    let lvl = if let LogicalLevel::Section(s) = current {
                        s.inc()
                    } else if current == LogicalLevel::None {
                        max
                    } else {
                        continue;
                    };
                    n_sections = n_sections + lvl;
                    let old = std::mem::replace(&mut current, LogicalLevel::Section(lvl));
                    stack.push((std::mem::replace(&mut curr, children.iter()), old));
                }
                TOCElem::Inputref { children, .. } => {
                    stack.push((std::mem::replace(&mut curr, children.iter()), current));
                }
                TOCElem::Paragraph { styles, kind, .. } => {
                    if let Some(n) = SectionCounters::get_counter(para_map, *kind, styles) {
                        tracing::trace!("Increasing counter {n}");
                        if let Some((_, c)) = n_counters.iter_mut().find(|(i, _)| *i == n) {
                            *c += 1;
                        } else {
                            n_counters.push((n, 1));
                        }
                    }
                }
            }
        } else if let Some((next, lvl)) = stack.pop() {
            curr = next;
            current = lvl;
        } else {
            break;
        }
    }

    tracing::trace!("Setting inpuref sections to {n_sections:?}");
    old_slides.set(n_slides);
    old_sections.set(n_sections);
    for (n, v) in n_counters {
        tracing::trace!("Patching counter {n} as {v}");
        if let Some((_, s, _)) = old_paras.iter().find(|(i, _, _)| *i == n) {
            s.set(v);
        }
    }
}
