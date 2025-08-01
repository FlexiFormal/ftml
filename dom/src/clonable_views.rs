use crate::{
    DocumentState, FtmlViews,
    counters::LogicalLevel,
    extractor::{DomExtractor, FtmlDomElement},
    markers::{Marker, MarkerList},
    terms::{ReactiveApplication, ReactiveTerm},
};
use ftml_core::extraction::{FtmlExtractor, OpenDomainElement, OpenNarrativeElement};
use ftml_ontology::narrative::elements::SectionLevel;
use leptos::prelude::*;
use leptos_posthoc::OriginalNode;

pub struct ClonableView(ClonableNode);

enum ClonableNode {
    Node(MarkedNode),
    Fn {
        is_math: bool,
        f: Box<dyn ClonableViewT>,
    },
}

#[derive(Clone)]
pub struct MarkedNode {
    markers: MarkerList,
    state: std::sync::Arc<parking_lot::Mutex<NodeState>>,
    is_math: bool,
    //owner: leptos::prelude::Owner,
}

struct NodeState {
    orig: OriginalNode,
    owner: Option<Owner>,
    child: Option<MarkedNode>,
    last_domain: Option<OpenDomainElement<FtmlDomElement>>,
    last_narrative: Option<OpenNarrativeElement<FtmlDomElement>>,
}

impl MarkedNode {
    #[inline]
    pub const fn is_math(&self) -> bool {
        self.is_math
    }
    pub fn first_pass(&self) -> bool {
        self.state.lock().owner.is_none()
    }
    pub fn into_view<Views: FtmlViews + ?Sized>(self) -> AnyView {
        let first_pass = self.first_pass();
        if !first_pass {
            tracing::debug!("Rerendering already rendered node");
        }
        let Some(marker) = self.markers.last().cloned() else {
            return self.render::<Views>().into_any();
        };
        let owner = self.owner();
        let child = move || -> ClonableView { self.child().into() };
        owner.with(move || match marker {
            Marker::CurrentSectionLevel(cap) => {
                let lvl = DocumentState::current_section_level();
                (match (lvl, cap) {
                    (LogicalLevel::None, true) => "Document",
                    (LogicalLevel::None, _) => "document",
                    (LogicalLevel::Section(SectionLevel::Part), true) => "Part",
                    (LogicalLevel::Section(SectionLevel::Part), _) => "part",
                    (LogicalLevel::Section(SectionLevel::Chapter), true) => "Chapter",
                    (LogicalLevel::Section(SectionLevel::Chapter), _) => "chapter",
                    (LogicalLevel::Section(SectionLevel::Section), true) => "Section",
                    (LogicalLevel::Section(SectionLevel::Section), _) => "section",
                    (LogicalLevel::Section(SectionLevel::Subsection), true) => "Subsection",
                    (LogicalLevel::Section(SectionLevel::Subsection), _) => "subsection",
                    (LogicalLevel::Section(SectionLevel::Subsubsection), true) => "Subsubsection",
                    (LogicalLevel::Section(SectionLevel::Subsubsection), _) => "subsubsection",
                    (LogicalLevel::BeamerSlide, true) => "Slide",
                    (LogicalLevel::BeamerSlide, _) => "slide",
                    (_, true) => "Paragraph",
                    (_, _) => "paragraph",
                })
                .into_any()
            }
            Marker::SymbolReference {
                in_term,
                uri,
                notation,
            } => Views::symbol_reference(uri, notation, in_term, child()).into_any(),
            Marker::VariableReference {
                in_term,
                var,
                notation,
            } => Views::variable_reference(var, notation, in_term, child()).into_any(),
            Marker::OMA {
                head,
                notation,
                uri,
            } => Views::application(head, notation, uri, child()).into_any(),
            Marker::OMBIND {
                head,
                notation,
                uri,
            } => Views::binder_application(head, notation, uri, child()).into_any(),
            Marker::Argument(pos) if first_pass => {
                with_context::<Option<ReactiveTerm>, _>(|t| t.as_ref().map(|t| t.app))
                    .flatten()
                    .map_or_else(
                        || child().into_view::<Views>(),
                        |r| ReactiveApplication::add_argument::<Views>(r, pos, child()).into_any(),
                    )
            }
            Marker::Argument(_) => child().into_view::<Views>().into_any(),
            Marker::Comp => Views::comp(child()).into_any(),
            _ => ftml_core::TODO!(),
        })
    }
    pub(crate) fn new(markers: MarkerList, orig: OriginalNode, is_math: bool) -> Self {
        Self {
            markers,
            is_math,
            state: std::sync::Arc::new(parking_lot::Mutex::new(NodeState {
                orig,
                last_domain: None,
                last_narrative: None,
                owner: None,
                child: None,
            })),
        }
    }
    fn child(&self) -> Self {
        // by construction, self.markers.len() > 0
        let mut state = self.state.lock();
        if let Some(c) = &state.child {
            return c.clone();
        }
        let orig = state.orig.deep_clone();
        let orig = std::mem::replace(&mut state.orig, orig);
        let next = Self {
            markers: self.markers[..self.markers.len() - 1]
                .iter()
                .cloned()
                .collect(),
            is_math: self.is_math,
            //owner: Owner::current().expect("not in a reactive context"),
            state: std::sync::Arc::new(parking_lot::Mutex::new(NodeState {
                orig,
                last_domain: None,
                last_narrative: None,
                owner: None,
                child: None,
            })),
        };
        state.child = Some(next.clone());
        next
    }

    fn owner(&self) -> Owner {
        let mut state = self.state.lock();
        let (owner, set_state) = {
            state.owner.clone().map_or_else(
                || {
                    let owner = Owner::current().expect("not in a reactive context");
                    state.owner = Some(owner.clone());
                    let sig = expect_context::<RwSignal<DomExtractor>>();
                    state.last_domain = sig.with_untracked(|e| e.iterate_domain().next().cloned());
                    state.last_narrative =
                        sig.with_untracked(|e| e.iterate_narrative().next().cloned());
                    (owner, false)
                },
                |o| (o, true),
            )
        };
        if set_state {
            let dom = state.last_domain.clone();
            let narr = state.last_narrative.clone();
            let has_dom = dom.is_some();
            let has_narr = narr.is_some();
            drop(state);
            let sig = owner.with(|| expect_context::<RwSignal<DomExtractor>>());
            if has_dom || has_narr {
                sig.update_untracked(|e| {
                    if let Some(dom) = dom {
                        tracing::debug!("Setting last domain to {dom:?}");
                        match dom {
                            OpenDomainElement::Argument {
                                position, terms, ..
                            } => e.state.domain.push(OpenDomainElement::Argument {
                                position,
                                terms,
                                node: FtmlDomElement::new((*self.state.lock().orig).clone()),
                            }),
                            o => e.state.domain.push(o),
                        }
                    }
                    if let Some(narr) = narr {
                        tracing::debug!("Setting last narrative to {narr:?}");
                        e.state.narrative.push(narr);
                    }
                });
            }
        }
        owner
    }

    fn render<Views: FtmlViews + ?Sized>(&self) -> impl IntoView + use<Views> {
        let owner = self.owner();
        let mut state = self.state.lock();
        let orig = state.orig.deep_clone();
        let orig = std::mem::replace(&mut state.orig, orig);
        drop(state);
        owner.with(|| {
            leptos_posthoc::DomCont(leptos_posthoc::DomContProps {
                orig,
                cont: super::iterate::<Views>,
                skip_head: true,
                class: None::<String>.into(),
                style: None::<String>.into(),
            })
        })
    }
}

impl ClonableView {
    pub const fn is_math(&self) -> bool {
        match &self.0 {
            ClonableNode::Node(n) => n.is_math,
            ClonableNode::Fn { is_math, .. } => *is_math,
        }
    }
    pub fn new<V: IntoView>(
        is_math: bool,
        f: impl Fn() -> V + Clone + 'static + Send + Sync,
    ) -> Self {
        Self(ClonableNode::Fn {
            is_math,
            f: f.into_boxed(),
        })
    }
    pub fn into_view<Views: FtmlViews + ?Sized>(self) -> AnyView {
        match self.0 {
            ClonableNode::Node(n) => n.into_view::<Views>(),
            ClonableNode::Fn { f, .. } => f.as_view(),
        }
    }
}
impl Clone for ClonableView {
    fn clone(&self) -> Self {
        match &self.0 {
            ClonableNode::Node(n) => Self(ClonableNode::Node(n.clone())),
            ClonableNode::Fn { is_math, f } => Self(ClonableNode::Fn {
                is_math: *is_math,
                f: f.as_boxed(),
            }),
        }
    }
}
impl From<MarkedNode> for ClonableView {
    fn from(value: MarkedNode) -> Self {
        Self(ClonableNode::Node(value))
    }
}

trait ClonableViewT: Send + Sync {
    fn as_boxed(&self) -> Box<dyn ClonableViewT>;
    fn into_boxed(self) -> Box<dyn ClonableViewT>
    where
        Self: Sized;
    fn as_view(&self) -> AnyView;
}
impl<V: IntoView, F: Fn() -> V + Clone + 'static + Send + Sync> ClonableViewT for F {
    fn as_boxed(&self) -> Box<dyn ClonableViewT> {
        Box::new(self.clone())
    }
    fn into_boxed(self) -> Box<dyn ClonableViewT>
    where
        Self: Sized,
    {
        Box::new(self)
    }
    fn as_view(&self) -> AnyView {
        self().into_any()
    }
}
