use crate::{
    DocumentState, FtmlViews,
    document::{CurrentUri, WithHead},
    extractor::{DomExtractor, FtmlDomElement},
    markers::{Marker, MarkerList},
    terms::ReactiveTerm,
    utils::ContextChain,
};
use ftml_ontology::terms::VarOrSym;
use ftml_parser::extraction::{
    ArgumentPosition, FtmlExtractor, OpenDomainElement, OpenNarrativeElement,
};
use leptos::prelude::*;
use leptos_posthoc::OriginalNode;

#[derive(Debug)]
pub struct ClonableView(ClonableNode);

impl ClonableView {
    pub(crate) fn set_state(&self) {
        match &self.0 {
            ClonableNode::Node(n) => n.set_state(),
            ClonableNode::Fn { .. } => (),
        }
    }
}

#[allow(clippy::large_enum_variant)]
enum ClonableNode {
    Node(MarkedNode),
    Fn {
        is_math: bool,
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
    saved_state: bool,
    child: Option<MarkedNode>,
    last_domain: Option<OpenDomainElement<FtmlDomElement>>,
    last_narrative: Option<OpenNarrativeElement<FtmlDomElement>>,
    was_rendered: bool,
}

impl MarkedNode {
    fn set_state(&self) {
        let mut state = self.state.lock();
        if self.no_arg {
            ContextChain::provide(None::<ArgumentPosition>);
        }
        //state.owner = Some(owner.clone());
        let sig = expect_context::<RwSignal<DomExtractor>>();
        state.last_domain = sig.with_untracked(|e| e.iterate_domain().next().cloned());
        state.last_narrative = sig.with_untracked(|e| e.iterate_narrative().next().cloned());
        state.saved_state = true;
    }

    #[inline]
    pub const fn is_math(&self) -> bool {
        self.is_math
    }
    pub fn first_pass(&self) -> bool {
        !self.state.lock().saved_state
    }
    pub fn into_view<Views: FtmlViews + ?Sized>(self) -> AnyView {
        let Some(marker) = self.markers.last().cloned() else {
            return self.render::<Views>().into_any();
        };
        let child = move |b| -> ClonableView { self.child(b).into() };
        match marker {
            Marker::CurrentSectionLevel(cap) => {
                let lvl = DocumentState::current_section_level();
                lvl.into_view(cap).into_any()
            }
            Marker::SymbolReference {
                in_term,
                uri,
                notation,
            } => {
                provide_context(WithHead(Some(VarOrSym::Sym(uri.clone()))));
                Views::symbol_reference(uri, notation, in_term, child(true)).into_any()
            }
            Marker::VariableReference {
                in_term,
                var,
                notation,
            } => {
                provide_context(WithHead(Some(VarOrSym::Var(var.clone()))));
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
            Marker::Comp => Views::comp(child(true)).into_any(),
            Marker::DefComp(u) => Views::def_comp(u, child(true)).into_any(),
            _ => ftml_parser::TODO!(),
        }
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
                saved_state: false,
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
            state: std::sync::Arc::new(parking_lot::Mutex::new(NodeState {
                orig,
                last_domain: None,
                last_narrative: None,
                saved_state: false,
                child: None,
                was_rendered: false,
            })),
        };
        state.child = Some(next.clone());
        next
    }

    fn maybe_set_state(&self) {
        let state = self.state.lock();
        if state.saved_state {
            return;
        }
        drop(state);
        self.set_state();
    }

    fn render<Views: FtmlViews + ?Sized>(&self) -> impl IntoView + use<Views> {
        self.maybe_set_state();
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
            let sig = expect_context::<RwSignal<DomExtractor>>();
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
        leptos_posthoc::DomCont(leptos_posthoc::DomContProps {
            orig,
            cont: |e: &_| super::iterate::<Views>(e),
            skip_head: true,
            class: None::<String>.into(),
            style: None::<String>.into(),
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

    /// ### Panics
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
