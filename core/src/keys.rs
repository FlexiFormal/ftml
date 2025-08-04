macro_rules! ftml {
    () => {
        "data-ftml-"
    };
    ($l:literal) => {
        concat!(ftml!(), $l)
    };
}
pub const PREFIX: &str = "data-ftml-";
pub const NUM_KEYS: u8 = 119;

macro_rules! do_keys {
    (@DOC $prefix:literal $tag:ident
        $(-? $otp:ty;)?
        $(@ $tp:ty)?
        =$val:literal
        $(+ $($other:ident),+ ;)?
        $(>> $($children:ident),+ ;)?
        $(&>> $($nchildren:ident),+ ;)?
        $(<= $($parents:ident),+ ;)?
        $(- $($req:ident),+ ;)?
        $(! $only:literal;)?
        $(-! $not:literal;)?
    ) => {
        concat!(
            "<div class=\"ftml-syntax\">\n\n`","data-ftml-",
                $val
                $(,"`[`=\"`<[",stringify!($otp),"]>`\"`]`")?
                $(,"=\"`<[",stringify!($tp),"]>`\"")?
                ,"`"
                $(,
                    "\n\nAdditional attributes: " $(,
                        "[" ,stringify!($other),"](",$prefix,stringify!($other), "), "
                    )*
                    ,""
                )?
                $(,
                    "\n\nChild nodes: " $(,
                        "[" ,stringify!($children),"](",$prefix,stringify!($children), "), "
                    )*
                    ,""
                )?
                $(,
                    "\n\n</div><div class=\"ftml-syntax\"><div></div>\n\nChild nodes: " $(,
                        "[" ,stringify!($nchildren),"](",$prefix,stringify!($nchildren), "), "
                    )*
                    ,""
                )?
                $(,
                    "\n\nAttribute of: " $(,
                        "[" ,stringify!($req),"](",$prefix,stringify!($req), "), "
                    )*
                    ,""
                )?
                $(,
                    "\n\nOnly allowed in: " $(,
                        "[" ,stringify!($parents),"](FTMLKey::",stringify!($parents), "), "
                    )*
                    ,""
                )?
            ,"\n\n</div>\n\n"
            $(
                , "<div class=\"warning\">\n\n*Only allowed "
                , $only,
                "*\n\n</div>\n\n"
            )?
            $(
                , "<div class=\"warning\">\n\n*Not allowed "
                , $not,
                "*\n\n</div>\n\n"
            )?
        )
    };
    ( $(
        $(#[$meta:meta])*
        $tag:ident
        $(-? $otp:ty;)?
        $(@ $tp:ty)?
        =$val:literal
        $(+ $($other:ident),+ ;)?
        $(>> $($children:ident),+ ;)?
        $(&>> $($nchildren:ident),+ ;)?
        $(<= $($parents:ident),+ ;)?
        $(- $($req:ident),+ ;)?
        $(! $only:literal;)?
        $(-! $not:literal;)?
        := $fun:ident
    )*
    ) => {

        #[derive(Copy,Clone,PartialEq, Eq,Hash,serde::Serialize, serde::Deserialize)]
        #[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
        #[repr(u8)]
        pub enum FtmlKey {
            $(
                #[doc = do_keys!(@DOC "FtmlKey::" $tag
                    $(-? $otp)?
                    $(@ $tp)?
                    = $val
                    $(+ $($other),+ ;)?
                    $(>> $($children),+ ;)?
                    $(&>> $($nchildren),+ ;)?
                    $(<= $($parents),+ ;)?
                    $(- $($req),+ ;)?
                    $(! $only;)?
                    $(-! $not;)?
                )]
                $(#[$meta])*
                $tag
            ),*
        }

        static AS_STRS: [&str;NUM_KEYS as usize] = [$($val),*];
        static ATTR_STRS: [&str;NUM_KEYS as usize] = [$(ftml!($val)),*];

        paste::paste! {
            // /// All attribute key names
            /*pub mod attrstrings {$(
                pub const [<$tag:snake:upper>]:&'static str
                    = ftml!($val);
            )*}*/
            impl FtmlKey {
                #[must_use]#[inline]
                pub const fn as_str(self) -> &'static str {
                    AS_STRS[(self as u8) as usize]
                }

                #[must_use]
                pub fn from_attr(s:&str) -> Option<Self> {
                    match s {
                        $( ftml!($val) => Some(Self::$tag) ),*,
                        _ => None
                    }
                }

                #[inline]#[must_use]
                pub const fn as_u8(self) -> u8 {
                    self as _
                }

                #[inline]#[must_use]
                pub const fn from_u8(b:u8) -> Option<Self> {
                    $(
                        if b == Self::$tag as u8 { return Some(Self::$tag);}
                    )*
                    None
                }

                #[must_use]
                pub const fn all_rules<E:crate::extraction::FtmlExtractor>() -> crate::extraction::FtmlRuleSet<E> {
                    crate::extraction::FtmlRuleSet([$(
                        do_keys!(@fun $tag $fun)
                    ),*])
                }

                #[must_use]#[inline]
                pub const fn attr_name(self) -> &'static str {
                    ATTR_STRS[(self as u8) as usize]
                }
                /*#[must_use]
                pub fn apply<E:crate::extraction::FtmlExtractor>(self,e:&mut E) -> Result<crate::extraction::OpenFtmlElement,crate::extraction::FtmlExtractionError> {
                    match self {$(
                        Self::$tag => crate::extraction::rules::$fun(e)
                    ),*}
                }*/
            }
        }
    };
    (@fun $self:ident todo) => { |e,a,k,n| crate::extraction::rules::todo(Self::$self,e,a,k,n) };
    (@fun $self:ident $fun:ident) => { crate::extraction::rules::$fun };
    (@u8 $slf:ident {$($p:ident)*}) => {0};
    (@u8 $slf:ident {$($p:ident)*} $n:ident $($r:ident)*) => {
        if matches!($slf,Self::$n) {return do_keys!(@count $($p)*);}
        do_keys!(@u8 $slf {$($p)* $n} $($r)*)
    };
    (@repl $p:ident) => {()};
    (@count) => {0};
    (@count $($p:ident)+ ) => { [$(do_keys!(@repl $p)),*].len() as u8 };
}

// `data-ftml-`
//pub const PREFIX: &str = ftml!();

do_keys! {
    /// Denotes a new [Section]. The given [SectionLevel] is only a sanity check;
    /// the actual level is determined by the occurrence within a [Document].
    Section                     @ SectionLevel  = "section"         + Id; -!"in [LogicalParagraph]s, [Problem]s or [Slide]s"; := section

    /// Denotes a new [LogicalParagraph] of [ParagraphKind::Definition]
    /// for the given [Symbol]s using the given styles.
    Definition                              = "definition"          + Id,Inline,Fors,Styles; &>> Definiens, Definiendum; := definition
    /// Denotes a new [LogicalParagraph] of [ParagraphKind::Assertion] (Theorems, Lemmata,
    /// Axioms, etc.) for the given [Symbol]s using the given styles.
    Assertion                               = "assertion"           + Id,Inline,Fors,Styles; := assertion
    /// Denotes a new [LogicalParagraph] of [ParagraphKind::Example] (this includes counterexamples)
    /// for the given [Symbol]s using the given styles.
    Example                                 = "example"             + Id,Inline,Fors,Styles; := example
    /// Denotes a new [LogicalParagraph] of [ParagraphKind::Paragraph]
    /// for the given [Symbol]s using the given styles.
    Paragraph                               = "paragraph"           + Id,Inline,Fors,Styles; := paragraph

    /// Denotes a new [Problem] with [`sub_problem`](Problem::sub_problem)`=false`
    Problem                                 = "problem"             + Id,Styles,Autogradable,ProblemPoints ; := todo
    /// Denotes a new [Problem] with [`sub_problem`](Problem::sub_problem)`=true`
    SubProblem                              = "subproblem"          + Id,Styles,Autogradable,ProblemPoints ; := todo

    /// Denotes a [Slide], implying that the [Document] is (or contains in some sense)
    /// a presentation.
    Slide                                   = "slide"               + Id;    -!"in [LogicalParagraph]s, [Problem]s or [Slide]s"; := todo


    // --------------------------------------------------------------------------------

    /// A (possibly empty) node that, when being rendered, should be replaced by the
    /// current slide number.
    SlideNumber                 = "slide-number"            !"in [Slide]s"; := todo

    // ------------------------------------------------------------------------------------

    /// Denotes a new [Module] (or [NestedModule]) with the given [Name] in the
    /// current [Namespace](PathURI).
    Module                      @ String        = "module"              + Metatheory, Signature; := module

    /// Denotes a new [MathStructure] or [Extension] with the given [Name].
    MathStructure               @ String        = "feature-structure"   + Macroname; !"in [Module]s"; := todo

    /// <div class="ftml-wip">TODO</div>
    Morphism                                    = "feature-morphism" := todo

    Proof                                   = "proof"               + Id,Inline,Fors,Styles,ProofHide; := todo
    SubProof                                = "subproof"            + Id,Inline,Fors,Styles,ProofHide; := todo



    Style                       = "style" := style
    CounterParent               = "counter-parent" := counter_parent
    Counter                     = "counter" := counter_parent

    DocTitle                    = "doctitle" := doctitle
    Title                       = "title" := title
    ProofTitle                  = "prooftitle" := todo
    SubproofTitle               = "subprooftitle" := todo

    Symdecl                     = "symdecl" := symdecl
    Vardef                      = "vardef" := vardef
    Varseq                      = "varseq" := varseq

    Notation                    = "notation" := notation
    NotationComp                = "notationcomp" := notation_comp
    NotationOpComp              = "notationopcomp" := notation_op_comp
    Definiendum                 = "definiendum"         <= Definition, Paragraph, Assertion; := todo

    Type                        = "type" := type_component
    Conclusion                  = "conclusion" := todo
    Definiens                   = "definiens"           <= Definition, Paragraph, Assertion; := definiens
    Rule                        = "rule" := todo

    ArgSep                      = "argsep" := argsep
    ArgMap                      = "argmap" := argmap
    ArgMapSep                   = "argmap-sep" := argmapsep

    Term                        = "term" := term
    Arg                         = "arg" := arg
    HeadTerm                    = "headterm" := todo

    ImportModule                = "import" := importmodule
    UseModule                   = "usemodule" := usemodule
    InputRef                    = "inputref" := inputref

    SetSectionLevel             = "sectionlevel" := setsectionlevel
    SkipSection                 = "skipsection" := skipsection


    ProofMethod                 = "proofmethod" := todo
    ProofSketch                 = "proofsketch" := todo
    ProofTerm                   = "proofterm" := todo
    ProofBody                   = "proofbody" := todo
    ProofAssumption             = "spfassumption" := todo
    ProofStep                   = "spfstep" := todo
    ProofStepName               = "stepname" := todo
    ProofEqStep                 = "spfeqstep" := todo
    ProofPremise                = "premise" := todo
    ProofConclusion             = "spfconclusion" := todo

    PreconditionDimension       = "preconditiondimension" := todo
    PreconditionSymbol          = "preconditionsymbol" := todo
    ObjectiveDimension          = "objectivedimension" := todo
    ObjectiveSymbol             = "objectivesymbol" := todo
    AnswerClass                 = "answerclass" := todo
    AnswerClassPts              = "answerclass-pts" := todo
    AnswerclassFeedback         = "answerclass-feedback" := todo
    ProblemMinutes              = "problemminutes" := todo
    ProblemMultipleChoiceBlock  = "multiple-choice-block" := todo
    ProblemSingleChoiceBlock    = "single-choice-block" := todo
    ProblemChoice               = "problem-choice" := todo
    ProblemChoiceVerdict        = "problem-choice-verdict" := todo
    ProblemChoiceFeedback       = "problem-choice-feedback" := todo
    ProblemFillinsol            = "fillinsol" := todo
    ProblemFillinsolWidth       = "fillinsol-width" := todo
    ProblemFillinsolCase        = "fillin-case" := todo
    ProblemFillinsolCaseValue   = "fillin-case-value" := todo
    ProblemFillinsolCaseVerdict = "fillin-case-verdict" := todo
    ProblemSolution            = "solution" := todo
    ProblemHint                = "problemhint" := todo
    ProblemNote                 = "problemnote" := todo
    ProblemGradingNote         = "problemgnote" := todo

    Comp                        = "comp" := comp
    VarComp                     = "varcomp" := comp
    MainComp                    = "maincomp" := maincomp
    DefComp                     = "defcomp" := todo

    Invisible                   = "invisible" := invisible

    IfInputref                  = "ifinputref" := todo
    ReturnType                  = "returntype" := todo
    ArgTypes                    = "argtypes" := todo

    SRef                        = "sref" := todo
    SRefIn                      = "srefin" := todo
    Slideshow                   = "slideshow" := todo
    SlideshowSlide              = "slideshow-slide"  := todo
    CurrentSectionLevel         = "currentsectionlevel" := currentsectionlevel
    Capitalize                  = "capitalize" := todo

    Assign                      = "assign" := todo
    Rename                      = "rename" := todo
    RenameTo                    = "to" := todo
    AssignMorphismFrom          = "assignmorphismfrom" := todo
    AssignMorphismTo            = "assignmorphismto" := todo

    AssocType                   = "assoctype" := todo
    ArgumentReordering          = "reorderargs" := todo
    ArgNum                      = "argnum" := argnum
    Bind                        = "bind" := todo
    MorphismDomain              = "domain" := todo
    MorphismTotal               = "total" := todo
    ArgMode                     = "argmode" := todo
    NotationId                  = "notationid" := todo
    Head                        = "head" := todo
    Language                    = "language" := todo
    /// The metatheory of a module, that provides the formal "language" the module
    /// is in
    Metatheory                              = "metatheory"      - Module; := todo
    Signature                               = "signature"       - Module; := todo
    Args                        = "args" := todo
    ProblemPoints               = "problempoints"               - Problem, SubProblem; := todo
    Autogradable                = "autogradable"                - Problem, SubProblem; := todo
    ProofHide                   = "proofhide"                   - Proof,SubProof; := todo
    Macroname                   = "macroname"                   - MathStructure; := todo
    Inline                      = "inline"                      - Definition, Paragraph, Assertion, Example, Problem, SubProblem; := todo
    Fors                        = "fors"                        - Definition, Paragraph, Assertion, Example, Proof, SubProof; := todo
    Id                          = "id"                          - Section,Definition, Paragraph, Assertion, Example, Proof, SubProof, Problem, SubProblem, Slide; := todo
    NotationFragment            = "notationfragment" := todo
    Precedence                  = "precedence" := todo
    Role                        = "role" := todo
    Styles                      = "styles" := todo
    Argprecs                    = "argprecs" := todo
}

impl std::fmt::Display for FtmlKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Debug for FtmlKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.attr_name())
    }
}

/*
#[cfg(test)]
#[test]
fn print_all_keys() {
    let _ = tracing_subscriber::fmt::try_init();
    for i in 0..NUM_KEYS {
        let k = FtmlKey::from_u8(i).expect("is a valid value");
        tracing::info!("{i}: {k} = {k:?}");
    }
}
 */
