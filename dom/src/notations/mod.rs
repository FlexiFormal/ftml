mod terms;

use crate::ClonableView;
use crate::document::CurrentUri;
use crate::terms::{ReactiveApplication, ReactiveTerm, TopTerm};
use crate::{DocumentState, FtmlViews};
use ftml_ontology::terms::VarOrSym;
use ftml_ontology::{
    narrative::elements::{
        Notation,
        notations::{NodeOrText, NotationComponent, NotationNode},
    },
    terms::ArgumentMode,
};
use ftml_parser::FtmlKey;
use ftml_parser::extraction::ArgumentPosition;
use leptos::attr::AttributeValue;
use leptos::attr::custom::CustomAttributeKey;
use leptos::math::mtext;
use leptos::tachys::view::any_view::AnyViewWithAttrs;
use leptos::{either::Either, prelude::*};
use std::num::NonZeroU8;
pub use terms::*;

pub trait NotationExt {
    fn with_arguments<Views: FtmlViews, R: ArgumentRender + ?Sized>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
        args: &R,
    ) -> impl IntoView + use<Self, Views, R>;
    fn with_arguments_safe<Views: FtmlViews, R: ArgumentRender + ?Sized>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
        args: &R,
    ) -> impl IntoView + use<Self, Views, R> {
        DocumentState::with_head(head.clone(), move || {
            provide_context(None::<TopTerm>);
            provide_context(None::<ReactiveTerm>);
            self.with_arguments::<Views, R>(head, this, args)
        })
    }
    fn as_view<Views: FtmlViews>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
    ) -> impl IntoView + use<Self, Views>;
    fn as_view_safe<Views: FtmlViews>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
    ) -> impl IntoView + use<Self, Views> {
        DocumentState::with_head(head.clone(), move || self.as_view::<Views>(head, this))
    }
    fn as_op<Views: FtmlViews>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
    ) -> impl IntoView + use<Self, Views>;
    fn as_op_safe<Views: FtmlViews>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
    ) -> impl IntoView + use<Self, Views> {
        DocumentState::with_head(head.clone(), move || self.as_op::<Views>(head, this))
    }
}

pub trait ArgumentRender {
    #[inline]
    fn is_empty(&self) -> bool {
        self.num_args() > 0
    }
    fn num_args(&self) -> usize;
    fn render_arg<Views: FtmlViews>(&self, index: u8, mode: ArgumentMode) -> AnyView;
    fn render_arg_at<Views: FtmlViews>(
        &self,
        index: u8,
        seq_index: usize,
        mode: ArgumentMode,
    ) -> AnyView;
    fn length_at(&self, index: u8) -> usize;
}

fn error() -> AnyView {
    mtext().style("color:red").child("ERROR").into_any()
}

impl ArgumentRender for Vec<Either<ClonableView, Vec<ClonableView>>> {
    #[inline]
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
    #[inline]
    fn num_args(&self) -> usize {
        self.len()
    }
    fn render_arg<Views: FtmlViews>(&self, index: u8, mode: ArgumentMode) -> AnyView {
        use leptos::either::Either::{Left, Right};
        tracing::trace!("rendering arg {index}@{mode:?} of {}", self.len());
        match self.get(index as usize) {
            Some(Either::Left(v)) => do_arg::<Views>(v.clone(), index, None, mode),
            Some(Either::Right(v)) => {
                let len = v.len();
                if len == 0 {
                    return ().into_any();
                }
                // SAFETY: len > 0
                let first = || {
                    do_arg::<Views>(
                        unsafe { v.first().unwrap_unchecked() }.clone(),
                        index,
                        Some(0),
                        mode,
                    )
                };
                if len == 1 {
                    return first();
                }
                view! {
                    <mrow>
                        {first()}
                    {
                        v.iter().enumerate().skip(1).map(|(i,v)| view!{
                            {

                                if with_context::<CurrentUri, _>(|_| ()).is_some() {
                                    Left(Views::comp(false,ClonableView::new(true,|| leptos::math::mo().child(","))))
                                } else {
                                    Right(leptos::math::mo().child(","))
                                }
                            }
                            {do_arg::<Views>(v.clone(), index, Some(i), mode)}
                        }).collect_view()
                    }
                    </mrow>
                }
                .into_any()
            }
            _ => error(),
        }
    }
    fn render_arg_at<Views: FtmlViews>(
        &self,
        index: u8,
        seq_index: usize,
        mode: ArgumentMode,
    ) -> AnyView {
        tracing::trace!(
            "rendering arg sequence@{mode:?} {index}/{seq_index} of {}",
            self.len()
        );
        match self.get(index as usize) {
            Some(Either::Right(v)) => v.get(seq_index).map_or_else(error, |v| {
                do_arg::<Views>(v.clone(), index, Some(seq_index), mode)
            }),
            Some(Either::Left(t)) if seq_index == 0 => {
                do_arg::<Views>(t.clone(), index, Some(seq_index), mode)
            }
            _ => error(),
        }
    }
    fn length_at(&self, index: u8) -> usize {
        self.get(index as usize)
            .map_or(0, |l| if let Either::Right(v) = l { v.len() } else { 1 })
    }
}

fn do_arg<Views: FtmlViews>(
    v: ClonableView,
    index: u8,
    seq_index: Option<usize>,
    mode: ArgumentMode,
) -> AnyView {
    if let Some(r) = use_context::<Option<ReactiveTerm>>().flatten() {
        let position = seq_index.map_or_else(
            || {
                // SAFETY: +1 > 0
                unsafe { ArgumentPosition::Simple(NonZeroU8::new_unchecked(index + 1), mode) }
            },
            |i| unsafe {
                ArgumentPosition::Sequence {
                    argument_number: NonZeroU8::new_unchecked(index + 1),
                    #[allow(clippy::cast_possible_truncation)]
                    sequence_index: NonZeroU8::new_unchecked((i + 1) as u8),
                    mode,
                }
            },
        );
        r.add_argument::<Views>(position, v).into_any()
    } else {
        v.into_view::<Views>()
    }
}

impl NotationExt for Notation {
    fn with_arguments<Views: FtmlViews, R: ArgumentRender + ?Sized>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
        args: &R,
    ) -> impl IntoView + use<Views, R> {
        use leptos::either::Either::{Left, Right};
        if args.is_empty() {
            return Left(self.as_op::<Views>(head, this));
        }
        Right(/*owned(move ||*/ {
            let h = head.to_string();
            //provide_context(WithHead(Some(head.clone())));
            let r = view_component_with_args::<Views>(&self.component, args, this)
                .attr(FtmlKey::Term.attr_name(), "OMBIND")
                .attr(FtmlKey::Head.attr_name(), h);
            ReactiveApplication::close();
            r
        }) //)
    }

    fn as_op<Views: FtmlViews>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
    ) -> impl IntoView + use<Views> {
        use leptos::either::Either::Left;
        attr(
            attr(
                self.op.as_ref().map_or_else(
                    || view_component_with_args::<Views>(&self.component, &DummyRender, this),
                    |op| {
                        let op = op.clone();
                        if with_context::<CurrentUri, _>(|_| ()).is_some() {
                            Left(
                                Views::comp(
                                    false,
                                    ClonableView::new(true, move || {
                                        attr(view_node(&op), "data-ftml-comp", "")
                                    }),
                                )
                                .into_any(),
                            )
                        } else {
                            attr(view_node(&op), "data-ftml-comp", "")
                        }
                    },
                ),
                FtmlKey::Term.attr_name(),
                "OMID",
            ),
            FtmlKey::Head.attr_name(),
            head.to_string(),
        )
    }

    fn as_view<Views: FtmlViews>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
    ) -> impl IntoView + use<Views> {
        //owned(move || {
        let h = head.to_string();
        //provide_context(WithHead(Some(head.clone())));
        attr(
            attr(
                view_component_with_args::<Views>(&self.component, &DummyRender, this),
                FtmlKey::Term.attr_name(),
                "OMID",
            ),
            FtmlKey::Head.attr_name(),
            h,
        )
        //})
    }
}

fn attr(
    e: leptos::either::Either<AnyView, AnyViewWithAttrs>,
    k: impl CustomAttributeKey,
    v: impl AttributeValue,
) -> leptos::either::Either<AnyView, AnyViewWithAttrs> {
    use leptos::either::Either::{Left, Right};
    Right(match e {
        Left(a) => a.attr(k, v),
        Right(a) => a.attr(k, v),
    })
}

pub(crate) fn view_component_with_args<Views: FtmlViews>(
    comp: &NotationComponent,
    args: &(impl ArgumentRender + ?Sized),
    this: Option<&ClonableView>,
) -> leptos::either::Either<AnyView, AnyViewWithAttrs> {
    use leptos::either::Either::Left;
    match comp {
        NotationComponent::Text(s) => Left(s.to_string().into_any()),
        NotationComponent::Node {
            tag,
            attributes,
            children,
        } => attributes.iter().fold(
            Left(html_from_tag(
                tag.as_ref(),
                children
                    .iter()
                    .map(|c| view_component_with_args::<Views>(c, args, this))
                    .collect_view(),
            )),
            |n, (k, v)| attr(n, k.to_string(), v.to_string()),
        ),
        NotationComponent::MainComp(n) if this.is_some() => {
            // SAFETY: defined
            let this = unsafe { this.unwrap_unchecked().clone() };
            let n = n.clone();
            let inner = if with_context::<CurrentUri, _>(|_| ()).is_some() {
                leptos::either::Either::Left(Views::comp(
                    false,
                    ClonableView::new(true, move || view_node(&n).attr("data-ftml-comp", "")),
                ))
            } else {
                leptos::either::Either::Right(view_node(&n).attr("data-ftml-comp", ""))
            };
            Left(view!(<msub>{inner}{this.into_view::<Views>()}</msub>).into_any())
        }
        NotationComponent::Comp(n) | NotationComponent::MainComp(n) => {
            let n = n.clone();
            if with_context::<CurrentUri, _>(|_| ()).is_some() {
                Left(
                    Views::comp(
                        false,
                        ClonableView::new(true, move || view_node(&n).attr("data-ftml-comp", "")),
                    )
                    .into_any(),
                )
            } else {
                Left(view_node(&n).attr("data-ftml-comp", "").into_any())
            }
        }
        NotationComponent::Argument { index, mode } => {
            Left(args.render_arg::<Views>(*index, *mode))
        }
        NotationComponent::ArgSep { index, mode, sep } => {
            let len = args.length_at(*index);
            if len == 0 {
                return Left(().into_any());
            }
            if len == 1 {
                return Left(args.render_arg::<Views>(*index, *mode));
            }

            Left(view! {
                <mrow>
                    {args.render_arg_at::<Views>(*index, 0, *mode)}
                {
                    (1..len).map(|i| view!{
                        {sep.iter().map(|s| view_component_with_args::<Views>(s,args,this)).collect_view()}
                        {args.render_arg_at::<Views>(*index, i, *mode)}
                    }).collect_view()
                }
                </mrow>
            }
            .into_any())
        }
        NotationComponent::ArgMap { .. } => todo!(),
    }
}

struct DummyRender;
impl ArgumentRender for DummyRender {
    #[inline]
    fn is_empty(&self) -> bool {
        true
    }
    #[inline]
    fn num_args(&self) -> usize {
        0
    }
    fn render_arg<Views: FtmlViews>(&self, index: u8, mode: ArgumentMode) -> AnyView {
        view!(<msub><mi>{mode.as_char()}</mi><mn>{index + 1}</mn></msub>).into_any()
    }
    #[inline]
    fn length_at(&self, _index: u8) -> usize {
        3
    }
    fn render_arg_at<Views: FtmlViews>(
        &self,
        index: u8,
        seq_index: usize,
        mode: ArgumentMode,
    ) -> AnyView {
        match seq_index {
            0 => {
                view!(<msubsup><mi>{mode.as_char()}</mi><mn>{index + 1}</mn><mn>{1}</mn></msubsup>)
                    .into_any()
            }
            1 => view!(<mo>"…"</mo>).into_any(),
            _ => view!(<msubsup>
                 <mi>{mode.as_char()}</mi>
                 <mn>{index + 1}</mn>
                 <msub>
                     <mn>"ℓ"</mn>
                     <mn>{index + 1}</mn>
                 </msub>
             </msubsup>)
            .into_any(),
        }
    }
}

pub(crate) fn view_node(n: &NotationNode) -> leptos::either::Either<AnyView, AnyViewWithAttrs> {
    use leptos::either::Either::Left;
    let NotationNode {
        tag,
        attributes,
        children,
    } = n;
    attributes.iter().fold(
        Left(html_from_tag(
            tag.as_ref(),
            children.iter().map(node_or_text).collect_view(),
        )),
        |n, (k, v)| attr(n, k.to_string(), v.to_string()),
    )
}

fn node_or_text(n: &NodeOrText) -> leptos::either::Either<AnyView, AnyViewWithAttrs> {
    match n {
        NodeOrText::Node(n) => view_node(n),
        NodeOrText::Text(t) => leptos::either::Either::Left(t.to_string().into_any()),
    }
}

pub(crate) fn html_from_tag(id: &str, children: impl IntoView) -> AnyView {
    macro_rules! tags {
        ( $(  {$($name:ident $($actual:ident)? ),* $(,)? } )*) => {
            match id {
                $( $(
                    stringify!($name) => view!(<$name>{children}</$name>)/* leptos::tachys::html::element::$name()
                        .child(children)*/.into_any(),//tags!(@NAME $name $($actual)?)::TAG,
                )*  )*
                _ => mtext().child("ERROR").into_any()
            }
        };
    }

    tags! {
        {
            //area,base,br,col,embed,hr,img,input,link,meta,source,track,wbr
        }
        {
            a,abbr,address,article,aside,audio,b,bdi,bdo,blockquote,body,
            button,canvas,caption,cite,code,colgroup,data,datalist,dd,
            del,details,dfn,dialog,div,dl,dt,em,fieldset,figcaption,figure,
            footer,form,h1,h2,h3,h4,h5,h6,head,header,hgroup,html,i,iframe,ins,
            kbd,label,legend,li,main,map,mark,menu,meter,nav,noscript,object,
            ol,optgroup,output,p,picture,portal,pre,progress,q,rp,rt,ruby,s,samp,
            script,search,section,select,slot,small,span,strong,style,sub,summary,
            sup,table,tbody,td,template,textarea,tfoot,th,thead,time,title,tr,u,
            ul,var,video
        }
        {option}
        {
            math,mi,mn,mo,ms,mspace,mtext,menclose,merror,mfenced,mfrac,mpadded,
            mphantom,mroot,mrow,msqrt,mstyle,mmultiscripts,mover,mprescripts,
            msub,msubsup,msup,munder,munderover,mtable,mtd,mtr,maction,annotation,
            semantics
        }
        /*{
            a,animate,animateMotion,animateTransform,circle,clipPath,defs,desc,
            discard,ellipse,feBlend,feColorMatrix,feComponentTransfer,
            feComposite,feConvolveMatrix,feDiffuseLighting,feDisplacementMap,
            feDistantLight,feDropShadow,feFlood,feFuncA,feFuncB,feFuncG,feFuncR,
            feGaussianBlur,feImage,feMerge,feMergeNode,feMorphology,feOffset,
            fePointLight,feSpecularLighting,feSpotLight,feTile,feTurbulence,
            filter,foreignObject,g,hatch,hatchpath,image,line,linearGradient,
            marker,mask,metadata,mpath,path,pattern,polygon,polyline,radialGradient,
            rect,script,set,stop,style,svg,switch,symbol,text,textPath,title,
            tspan,view
        }*/
    }
}
/*
pub enum AnyV {
    Any(AnyView),
    Attr(AnyViewWithAttrs),
}
impl AnyV {
    fn into_view(self) -> impl IntoView {
        match self {
            Self::Any(a) => leptos::either::Either::Left(a),
            Self::Attr(a) => leptos::either::Either::Right(a),
        }
    }
}
impl AddAnyAttr for AnyV {
    type Output<SomeNewAttr: leptos::attr::Attribute> = Self;
    fn add_any_attr<NewAttr: leptos::attr::Attribute>(self, attr: NewAttr) -> Self::Output<NewAttr>
    where
        Self::Output<NewAttr>: RenderHtml,
    {
        Self::Attr(match self {
            Self::Any(a) => a.add_any_attr(attr),
            Self::Attr(a) => a.add_any_attr(attr),
        })
    }
}
impl From<AnyView> for AnyV {
    #[inline]
    fn from(value: AnyView) -> Self {
        Self::Any(value)
    }
}
impl RenderHtml for AnyV {
    type AsyncOutput =
    type State = leptos::either::Either<<AnyView as RenderHtml>::AsyncOutput,<AnyViewWithAttrs as RenderHtml>::AsyncOutput>;
    type Owned =
    type State = leptos::either::Either<<AnyView as RenderHtml>::Owned,<AnyViewWithAttrs as RenderHtml>::Owned>;
}
impl Render for AnyV {
    type State = leptos::either::Either<<AnyView as Render>::State,<AnyViewWithAttrs as Render>::State>;
}
*/
