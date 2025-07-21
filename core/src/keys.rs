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

        #[derive(Copy,Clone,PartialEq, Eq,Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
        #[repr(u8)]
        pub enum FtmlKey {
            $(
                #[doc = do_keys!(@DOC "FTMLKey::" $tag
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
                        crate::extraction::rules::$fun
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
    Definition                              = "definition"          + Id,Inline,Fors,Styles; &>> Definiens, Definiendum; := no_op
    /// Denotes a new [LogicalParagraph] of [ParagraphKind::Assertion] (Theorems, Lemmata,
    /// Axioms, etc.) for the given [Symbol]s using the given styles.
    Assertion                               = "assertion"           + Id,Inline,Fors,Styles; := no_op
    /// Denotes a new [LogicalParagraph] of [ParagraphKind::Example] (this includes counterexamples)
    /// for the given [Symbol]s using the given styles.
    Example                                 = "example"             + Id,Inline,Fors,Styles; := no_op
    /// Denotes a new [LogicalParagraph] of [ParagraphKind::Paragraph]
    /// for the given [Symbol]s using the given styles.
    Paragraph                               = "paragraph"           + Id,Inline,Fors,Styles; := no_op

    /// Denotes a new [Problem] with [`sub_problem`](Problem::sub_problem)`=false`
    Problem                                 = "problem"             + Id,Styles,Autogradable,ProblemPoints ; := no_op
    /// Denotes a new [Problem] with [`sub_problem`](Problem::sub_problem)`=true`
    SubProblem                              = "subproblem"          + Id,Styles,Autogradable,ProblemPoints ; := no_op

    /// Denotes a [Slide], implying that the [Document] is (or contains in some sense)
    /// a presentation.
    Slide                                   = "slide"               + Id;    -!"in [LogicalParagraph]s, [Problem]s or [Slide]s"; := no_op


    // --------------------------------------------------------------------------------

    /// A (possibly empty) node that, when being rendered, should be replaced by the
    /// current slide number.
    SlideNumber                 = "slide-number"            !"in [Slide]s"; := no_op

    // ------------------------------------------------------------------------------------

    /// Denotes a new [Module] (or [NestedModule]) with the given [Name] in the
    /// current [Namespace](PathURI).
    Module                      @ String        = "module"              + Metatheory, Signature; := module

    /// Denotes a new [MathStructure] or [Extension] with the given [Name].
    MathStructure               @ String        = "feature-structure"   + Macroname; !"in [Module]s"; := no_op

    /// <div class="ftml-wip">TODO</div>
    Morphism                                    = "feature-morphism" := no_op

    Proof                                   = "proof"               + Id,Inline,Fors,Styles,ProofHide; := no_op
    SubProof                                = "subproof"            + Id,Inline,Fors,Styles,ProofHide; := no_op



    Style                       = "style" := style
    CounterParent               = "counter-parent" := counter_parent
    Counter                     = "counter" := counter_parent

    DocTitle                    = "doctitle" := no_op
    Title                       = "title" := no_op
    ProofTitle                  = "prooftitle" := no_op
    SubproofTitle               = "subprooftitle" := no_op

    Symdecl                     = "symdecl" := symbol
    Vardef                      = "vardef" := no_op
    Varseq                      = "varseq" := no_op

    Notation                    = "notation" := no_op
    NotationComp                = "notationcomp" := no_op
    NotationOpComp              = "notationopcomp" := no_op
    Definiendum                 = "definiendum"         <= Definition, Paragraph, Assertion; := no_op

    Type                        = "type" := no_op
    Conclusion                  = "conclusion" := no_op
    Definiens                   = "definiens"           <= Definition, Paragraph, Assertion; := no_op
    Rule                        = "rule" := no_op

    ArgSep                      = "argsep" := no_op
    ArgMap                      = "argmap" := no_op
    ArgMapSep                   = "argmap-sep" := no_op

    Term                        = "term" := no_op
    Arg                         = "arg" := no_op
    HeadTerm                    = "headterm" := no_op

    ImportModule                = "import" := no_op
    UseModule                   = "usemodule" := no_op
    InputRef                    = "inputref" := no_op

    SetSectionLevel             = "sectionlevel" := setsectionlevel
    SkipSection                 = "skipsection" := no_op


    ProofMethod                 = "proofmethod" := no_op
    ProofSketch                 = "proofsketch" := no_op
    ProofTerm                   = "proofterm" := no_op
    ProofBody                   = "proofbody" := no_op
    ProofAssumption             = "spfassumption" := no_op
    ProofStep                   = "spfstep" := no_op
    ProofStepName               = "stepname" := no_op
    ProofEqStep                 = "spfeqstep" := no_op
    ProofPremise                = "premise" := no_op
    ProofConclusion             = "spfconclusion" := no_op

    PreconditionDimension       = "preconditiondimension" := no_op
    PreconditionSymbol          = "preconditionsymbol" := no_op
    ObjectiveDimension          = "objectivedimension" := no_op
    ObjectiveSymbol             = "objectivesymbol" := no_op
    AnswerClass                 = "answerclass" := no_op
    AnswerClassPts              = "answerclass-pts" := no_op
    AnswerclassFeedback         = "answerclass-feedback" := no_op
    ProblemMinutes              = "problemminutes" := no_op
    ProblemMultipleChoiceBlock  = "multiple-choice-block" := no_op
    ProblemSingleChoiceBlock    = "single-choice-block" := no_op
    ProblemChoice               = "problem-choice" := no_op
    ProblemChoiceVerdict        = "problem-choice-verdict" := no_op
    ProblemChoiceFeedback       = "problem-choice-feedback" := no_op
    ProblemFillinsol            = "fillinsol" := no_op
    ProblemFillinsolWidth       = "fillinsol-width" := no_op
    ProblemFillinsolCase        = "fillin-case" := no_op
    ProblemFillinsolCaseValue   = "fillin-case-value" := no_op
    ProblemFillinsolCaseVerdict = "fillin-case-verdict" := no_op
    ProblemSolution            = "solution" := no_op
    ProblemHint                = "problemhint" := no_op
    ProblemNote                 = "problemnote" := no_op
    ProblemGradingNote         = "problemgnote" := no_op

    Comp                        = "comp" := no_op
    VarComp                     = "varcomp" := no_op
    MainComp                    = "maincomp" := no_op
    DefComp                     = "defcomp" := no_op

    Invisible                   = "invisible" := invisible

    IfInputref                  = "ifinputref" := no_op
    ReturnType                  = "returntype" := no_op
    ArgTypes                    = "argtypes" := no_op

    SRef                        = "sref" := no_op
    SRefIn                      = "srefin" := no_op
    Slideshow                   = "slideshow" := no_op
    SlideshowSlide              = "slideshow-slide"  := no_op
    CurrentSectionLevel         = "currentsectionlevel" := no_op
    Capitalize                  = "capitalize" := no_op

    Assign                      = "assign" := no_op
    Rename                      = "rename" := no_op
    RenameTo                    = "to" := no_op
    AssignMorphismFrom          = "assignmorphismfrom" := no_op
    AssignMorphismTo            = "assignmorphismto" := no_op

    AssocType                   = "assoctype" := no_op
    ArgumentReordering          = "reorderargs" := no_op
    ArgNum                      = "argnum" := no_op
    Bind                        = "bind" := no_op
    MorphismDomain              = "domain" := no_op
    MorphismTotal               = "total" := no_op
    ArgMode                     = "argmode" := no_op
    NotationId                  = "notationid" := no_op
    Head                        = "head" := no_op
    Language                    = "language" := no_op
    /// The metatheory of a module, that provides the formal "language" the module
    /// is in
    Metatheory                              = "metatheory"      - Module; := no_op
    Signature                               = "signature"       - Module; := no_op
    Args                        = "args" := no_op
    ProblemPoints               = "problempoints"               - Problem, SubProblem; := no_op
    Autogradable                = "autogradable"                - Problem, SubProblem; := no_op
    ProofHide                   = "proofhide"                   - Proof,SubProof; := no_op
    Macroname                   = "macroname"                   - MathStructure; := no_op
    Inline                      = "inline"                      - Definition, Paragraph, Assertion, Example, Problem, SubProblem; := no_op
    Fors                        = "fors"                        - Definition, Paragraph, Assertion, Example, Proof, SubProof; := no_op
    Id                          = "id"                          - Section,Definition, Paragraph, Assertion, Example, Proof, SubProof, Problem, SubProblem, Slide; := no_op
    NotationFragment            = "notationfragment" := no_op
    Precedence                  = "precedence" := no_op
    Role                        = "role" := no_op
    Styles                      = "styles" := no_op
    Argprecs                    = "argprecs" := no_op
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
