use crate::{
    DocumentState, FtmlViews,
    counters::LogicalLevel,
    document::{CurrentUri, WithHead},
    extractor::{DomExtractor, FtmlDomElement},
    markers::{Marker, MarkerList},
    terms::ReactiveTerm,
    utils::ContextChain,
};
use ftml_core::extraction::{
    ArgumentPosition, FtmlExtractor, OpenDomainElement, OpenNarrativeElement, VarOrSym,
};
use ftml_ontology::narrative::elements::SectionLevel;
use leptos::prelude::*;
use leptos_posthoc::OriginalNode;

#[derive(Debug)]
pub struct ClonableView(ClonableNode);

impl ClonableView {
    pub(crate) fn add_owner(&self, owner: Owner) {
        match &self.0 {
            ClonableNode::Node(n) => n.add_owner(owner),
            ClonableNode::Fn { owner: o, .. } => *o.lock() = Some(owner),
        }
    }
}

enum ClonableNode {
    Node(MarkedNode),
    Fn {
        is_math: bool,
        owner: std::sync::Arc<parking_lot::Mutex<Option<Owner>>>,
        f: Box<dyn ClonableViewT>,
    },
}

impl std::fmt::Debug for ClonableNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node(_) => f.write_str("Node"),
            Self::Fn { .. } => f.write_str("Fn"),
        }
    }
}

#[derive(Clone)]
pub struct MarkedNode {
    no_arg: bool,
    is_math: bool,
    markers: MarkerList,
    state: std::sync::Arc<parking_lot::Mutex<NodeState>>,
}

struct NodeState {
    orig: OriginalNode,
    owner: Option<Owner>,
    child: Option<MarkedNode>,
    last_domain: Option<OpenDomainElement<FtmlDomElement>>,
    last_narrative: Option<OpenNarrativeElement<FtmlDomElement>>,
    was_rendered: bool,
}

impl MarkedNode {
    fn add_owner(&self, owner: Owner) {
        let mut state = self.state.lock();
        owner.with(|| {
            if self.no_arg {
                ContextChain::provide(None::<ArgumentPosition>);
            }
            //state.owner = Some(owner.clone());
            let sig = expect_context::<RwSignal<DomExtractor>>();
            state.last_domain = sig.with_untracked(|e| e.iterate_domain().next().cloned());
            state.last_narrative = sig.with_untracked(|e| e.iterate_narrative().next().cloned());
        });
        state.owner = Some(owner);
    }

    #[inline]
    pub const fn is_math(&self) -> bool {
        self.is_math
    }
    pub fn first_pass(&self) -> bool {
        self.state.lock().owner.is_none()
    }
    pub fn into_view<Views: FtmlViews + ?Sized>(self) -> AnyView {
        /*let first_pass = self.first_pass();
        if !first_pass {
            tracing::debug!("Rerendering already rendered node");
        }*/
        let Some(marker) = self.markers.last().cloned() else {
            return self.render::<Views>().into_any();
        };
        //let owner = self.owner();
        let child = move |b| -> ClonableView { self.child(b).into() };
        //owner.with(move || { owned(||
        match marker {
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
            } => {
                provide_context(WithHead(Some(VarOrSym::S(uri.clone()))));
                Views::symbol_reference(uri, notation, in_term, child(true)).into_any()
            }
            Marker::VariableReference {
                in_term,
                var,
                notation,
            } => {
                provide_context(WithHead(Some(VarOrSym::V(var.clone()))));
                Views::variable_reference(var, notation, in_term, child(true)).into_any()
            }
            Marker::OMA {
                head,
                notation,
                uri,
            } => {
                provide_context(WithHead(Some(head.clone())));
                if let Some(uri) = &uri {
                    provide_context(CurrentUri(uri.clone().into()));
                }
                Views::application(head, notation, uri, child(true)).into_any()
            }
            Marker::OMBIND {
                head,
                notation,
                uri,
            } => {
                provide_context(WithHead(Some(head.clone())));
                if let Some(uri) = &uri {
                    provide_context(CurrentUri(uri.clone().into()));
                }
                Views::binder_application(head, notation, uri, child(true)).into_any()
            }
            Marker::Argument(pos) => {
                provide_context(WithHead(None));
                use_context::<Option<ReactiveTerm>>().flatten().map_or_else(
                    || child(false).into_view::<Views>(),
                    |r| r.add_argument::<Views>(pos, child(false)).into_any(),
                )
            }
            //Marker::Argument(_) => child(false).into_view::<Views>().into_any(),
            Marker::Comp => Views::comp(false, child(true)).into_any(),
            Marker::DefComp => Views::comp(true, child(true)).into_any(),
            _ => ftml_core::TODO!(),
        }
        //    )})
        .into_any()
    }
    pub(crate) fn new(
        markers: MarkerList,
        orig: OriginalNode,
        is_math: bool,
        no_arg: bool,
    ) -> Self {
        Self {
            markers,
            is_math,
            no_arg,
            state: std::sync::Arc::new(parking_lot::Mutex::new(NodeState {
                orig,
                last_domain: None,
                last_narrative: None,
                owner: None,
                child: None,
                was_rendered: false,
            })),
        }
    }
    fn child(&self, no_arg: bool) -> Self {
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
            no_arg,
            //owner: Owner::current().expect("not in a reactive context"),
            state: std::sync::Arc::new(parking_lot::Mutex::new(NodeState {
                orig,
                last_domain: None,
                last_narrative: None,
                owner: None,
                child: None,
                was_rendered: false,
            })),
        };
        state.child = Some(next.clone());
        next
    }

    #[deprecated(note = "clean up")]
    fn owner(&self) -> Owner {
        /*let mut state = self.state.lock();
        let (owner, set_state) = {
            state.owner.clone().map_or_else(
                || {
                    let owner = Owner::current().expect("not in a reactive context");
                    if self.no_arg {
                        ContextChain::provide(None::<ArgumentPosition>);
                    }
                    //state.owner = Some(owner.clone());
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
        //tracing::warn!("Current owner ancestry: {:?}", owner.ancestry());
        //owner
        */
        let state = self.state.lock();
        if let Some(owner) = &state.owner {
            return Owner::current().expect("exists"); //owner.clone();
        }
        drop(state);
        let owner = Owner::current().expect("exists");
        self.add_owner(owner);
        //owner
        Owner::current().expect("exists")
    }

    fn render<Views: FtmlViews + ?Sized>(&self) -> impl IntoView + use<Views> {
        let owner = self.owner();
        let mut state = self.state.lock();
        let was_rendered = state.was_rendered;
        state.was_rendered = true;
        let orig = state.orig.deep_clone();
        let orig = std::mem::replace(&mut state.orig, orig);
        let dom = if was_rendered {
            state.last_domain.clone()
        } else {
            None
        };
        let narr = if was_rendered {
            state.last_narrative.clone()
        } else {
            None
        };

        drop(state);

        if was_rendered && (dom.is_some() || narr.is_some()) {
            let sig = owner.with(|| expect_context::<RwSignal<DomExtractor>>());
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
        owner.with(|| {
            leptos_posthoc::DomCont(leptos_posthoc::DomContProps {
                orig,
                cont: |e: &_| super::iterate::<Views>(e),
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
            owner: std::sync::Arc::new(parking_lot::Mutex::new(None)),
            f: f.into_boxed(),
        })
    }

    /// ### Panics
    pub fn into_view<Views: FtmlViews + ?Sized>(self) -> AnyView {
        match self.0 {
            ClonableNode::Node(n) => n.into_view::<Views>(),
            ClonableNode::Fn { f, owner, .. } => {
                let owner = owner.lock().clone();
                owner.map_or_else(|| f.as_view(), |owner| owner.with(|| f.as_view()))
            }
        }
    }
}
impl Clone for ClonableView {
    fn clone(&self) -> Self {
        match &self.0 {
            ClonableNode::Node(n) => Self(ClonableNode::Node(n.clone())),
            ClonableNode::Fn { is_math, f, owner } => Self(ClonableNode::Fn {
                is_math: *is_math,
                owner: owner.clone(),
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
