pub mod rdf {
    pub use oxrdf::vocab::rdf::*;
}
pub mod rdfs {
    pub use oxrdf::vocab::rdfs::*;
}
pub mod xsd {
    pub use oxrdf::vocab::xsd::*;
}

macro_rules! count {
    () => (0usize);
    ( $e:expr; $($n:expr;)* ) => (1usize + count!($($n;)*));
}

macro_rules! dict {
    ($(#[$meta:meta])* $name:ident = $uri:literal: $($rest:tt)*) => {
        $(#[$meta])*
        #[doc=concat!("Namespace: `",$uri,"`")]
        #[allow(non_upper_case_globals)]
        pub mod $name {

            use crate::rdf_types::*;

            #[doc=concat!("`",$uri,"`")]
            pub const NAMESPACE: NamedNodeRef = NamedNodeRef::new_unchecked($uri);

            dict!{@consts $uri {} $($rest)*}
        }
    };

    (@consts $uri:literal { $($quad:expr;)* } { $name:ident $($rst:tt)* }; $($rest:tt)*) => {
        dict!{@makequads $name $uri $($rst)* {$($quad;)*} $($rest)* }
    };

    (@consts $uri:literal { $($quad:expr;)* } $(#[$meta:meta])*
        $i:ident
        $(: $($(::$tpns:ident ::)?$tp:ident),* )?
        $( ( $(::$domns:ident ::)?$dom:ident -> $(::$codns:ident ::)?$cod:ident  ) )?
        $(! $($(::$negns:ident ::)?$neg:ident),* )?
        $(<: $($(::$subclsns:ident ::)? $subcls:ident),* )?
        $(<< $($(::$subtpns:ident ::)? $subtp:ident),* )?
        $(- $($(::$invns:ident ::)? $inv:ident),* )?
        $(@ $doc:literal)?
        ; $($rest:tt)*
    ) => {
        dict!{@consts $uri {$($quad;)*} $(#[$meta])* $i = stringify!($i),
            $(: $($(::$tpns::)?$tp),* )?
            $( ( $(::$domns::)?$dom -> $(::$codns::)?$cod ) )?
            $(! $($(::$negns::)?$neg),* )?
            $(<: $($(::$subclsns::)?$subcls),* )?
            $(<< $($(::$subtpns::)?$subtp),* )?
            $(- $($(::$invns::)?$inv),* )?
            $(@ $doc)?;
            $($rest)*
        }
    };

    (@consts $uri:literal { $($quad:expr;)* } $(#[$meta:meta])*
        $i:ident = $name:expr,
        $(: $($(::$tpns:ident ::)?$tp:ident),* )?
        $( ( $(::$domns:ident ::)?$dom:ident -> $(::$codns:ident ::)?$cod:ident  ) )?
        $(! $($(::$negns:ident ::)?$neg:ident),* )?
        $(<: $($(::$subclsns:ident ::)? $subcls:ident),* )?
        $(<< $($(::$subtpns:ident ::)? $subtp:ident),* )?
        $(- $($(::$invns:ident ::)? $inv:ident),* )?
        $(@ $doc:literal)?
        ; $($rest:tt)*
    ) => {
        $(#[doc=$doc])?
        #[doc=""] $(#[$meta])*
        #[doc=concat!("`",$uri,"#",$name,"`")]
        pub const $i : NamedNodeRef = NamedNodeRef::new_unchecked(concat!($uri,"#",$name));
        dict!{@makequads $i $uri
            $(: $($(::$tpns::)?$tp),* )?
            $( ( $(::$domns::)?$dom -> $(::$codns::)?$cod ) )?
            $(! $($(::$negns::)?$neg),* )?
            $(<: $($(::$subclsns::)?$subcls),* )?
            $(<< $($(::$subtpns::)?$subtp),* )?
            $(- $($(::$invns::)?$inv),* )?
            {$($quad;)*}
            $($rest)*
        }
    };
    (@consts $uri:literal { $($quad:expr;)* }) => {
        #[doc=concat!("All relations on the elements in `",$uri,"`")]
        pub static QUADS :&[QuadRef;count!($( $quad; )*)] = &[$( $quad ),*];
    };

    (@makequads $name:ident $uri:literal
        $(<$(::$predns:ident ::)? $pred:ident> S $val:literal)?
        $(: $($(::$tpns:ident ::)?$tp:ident),* )?
        $( ( $(::$domns:ident ::)?$dom:ident -> $(::$codns:ident ::)?$cod:ident  ) )?
        $(! $($(::$negns:ident ::)?$neg:ident),* )?
        $(<: $($(::$subclsns:ident ::)? $subcls:ident),* )?
        $(<< $($(::$subtpns:ident ::)? $subtp:ident),* )?
        $(- $($(::$invns:ident::)? $inv:ident),* )?
        {$($quad:expr;)*} $($rest:tt)*
    ) => {
        dict!{@consts $uri
            {
                $($quad;)*
                $(
                    QuadRef{
                        subject:SubjectRef::NamedNode($name),
                        predicate:$(super::$predns::)?$pred,
                        object:RDFTermRef::Literal(LiteralRef::new_simple_literal($val)),
                        graph_name:GraphNameRef::NamedNode(NAMESPACE)
                    };
                )?
                $(
                    QuadRef{
                        subject:SubjectRef::NamedNode($name),
                        predicate:super::rdfs::DOMAIN,
                        object:RDFTermRef::NamedNode($(super::$domns::)?$dom),
                        graph_name:GraphNameRef::NamedNode(NAMESPACE)
                    };
                    QuadRef{
                        subject:SubjectRef::NamedNode($name),
                        predicate:super::rdfs::RANGE,
                        object:RDFTermRef::NamedNode($(super::$codns::)?$cod),
                        graph_name:GraphNameRef::NamedNode(NAMESPACE)
                    };
                )?
                $(
                    $(QuadRef{
                        subject:SubjectRef::NamedNode($name),
                        predicate:super::owl::disjointWith,
                        object:RDFTermRef::NamedNode($(super::$negns::)?$neg),
                        graph_name:GraphNameRef::NamedNode(NAMESPACE)
                    };)*
                )?
                $(
                    $(QuadRef{
                        subject:SubjectRef::NamedNode($name),
                        predicate:super::rdf::TYPE,
                        object:RDFTermRef::NamedNode($(super::$tpns::)?$tp),
                        graph_name:GraphNameRef::NamedNode(NAMESPACE)
                    };)*
                )?
                $(
                    $(QuadRef{
                        subject:SubjectRef::NamedNode($name),
                        predicate:super::rdfs::SUB_CLASS_OF,
                        object:RDFTermRef::NamedNode($(super::$subclsns::)?$subcls),
                        graph_name:GraphNameRef::NamedNode(NAMESPACE)
                    };)*
                )?
                $(
                    $(QuadRef{
                        subject:SubjectRef::NamedNode($name),
                        predicate:super::rdfs::SUB_PROPERTY_OF,
                        object:RDFTermRef::NamedNode($(super::$subtpns::)?$subtp),
                        graph_name:GraphNameRef::NamedNode(NAMESPACE)
                    };)*
                )?
                $(
                    $(QuadRef{
                        subject:SubjectRef::NamedNode($name),
                        predicate:super::owl::inverseOf,
                        object:RDFTermRef::NamedNode($(super::$invns::)?$inv),
                        graph_name:GraphNameRef::NamedNode(NAMESPACE)
                    };)*
                )?
            }
            $($rest)*
        }
    };
}

dict! {
    /// (Parts of) the [Dublin Core Metadata Terms](https://www.dublincore.org/specifications/dublin-core/dcmi-terms) used in the ULO
    dc = "http://purl.org/dc/terms":

    /// Recommended practice is to identify the related resource by means of a URI. If this is not possible or feasible,
    /// a string conforming to a formal identification system may be provided.
    relation : ::owl::Class <: ::rdf::PROPERTY @ "A related resource.";

    /// Typically, rights information includes a statement about various property rights associated with the resource, including intellectual property rights.
    rights @ "Information about rights held in and over the resource.";

    /// Recommended practice is to use either a non-literal value representing a language from a
    /// controlled vocabulary such as ISO 639-2 or ISO 639-3, or a literal value consisting of an
    /// IETF Best Current Practice 47 [IETF-BCP47](https://tools.ietf.org/html/bcp47) language tag.
    language @ "A language of the resource.";

    /// This property is intended to be used with non-literal values.
    /// This property is an inverse property of [`isPartOf`].
    hasPart : relation -isPartOf
        @ "A related resource that is included either physically or logically in the described resource.";

    /// his property is intended to be used with non-literal values.
    /// This property is an inverse property of [`hasPart`].
    isPartOf : relation -hasPart
        @ "A related resource in which the described resource is physically or logically included.";

    /// This property is intended to be used with non-literal values.
    /// This property is an inverse property of [`isRequiredBy`].
    requires : relation -isRequiredBy
        @ "A related resource that is required by the described resource to support its function, delivery, or coherence.";

    /// This property is intended to be used with non-literal values.
    /// This property is an inverse property of [`requires`].
    isRequiredBy : relation -requires
        @ "A related resource that requires the described resource to support its function, delivery, or coherence.";
}

dict! {
    /// The [OWL Web Ontology Language](https://www.w3.org/TR/owl-ref/)
    owl = "http://www.w3.org/2002/07/owl":

    /** OWL distinguishes between two main categories of properties that an ontology builder
     * may want to define:
     * - Object properties link individuals to individuals.
     * - Datatype properties link individuals to data values.
     *
     * An object property is defined as an instance of the built-in OWL class owl:ObjectProperty.
     */
    ObjectProperty <: ::dc::relation, ::rdf::PROPERTY;

    /** OWL distinguishes between two main categories of properties that an ontology builder
     * may want to define:
     * - Object properties link individuals to individuals.
     * - Datatype properties link individuals to data values.
     *
     * A datatype property is defined as an instance of the built-in OWL class owl:DatatypeProperty.
     */
    DatatypeProperty <: ::dc::relation, ::rdf::PROPERTY;

    /// Classes provide an abstraction mechanism for grouping resources with similar characteristics.
    /// Like RDF classes, every OWL class is associated with a set of individuals, called the class
    /// extension. The individuals in the class extension are called the instances of the class.
    /// A class has an intensional meaning (the underlying concept) which is related but not equal to
    /// its class extension. Thus, two classes may have the same class extension, but still be
    /// different classes.
    Class;
    /// `owl:disjointWith` is a built-in OWL property with a class description as domain and range.
    /// Each `owl:disjointWith` statement asserts that the class extensions of the two class
    /// descriptions involved have no individuals in common. Like axioms with `rdfs:subClassOf`,
    /// declaring two classes to be disjoint is a partial definition: it imposes a necessary but not
    /// sufficient condition on the class.
    disjointWith << ObjectProperty;

    //disjointUnionOf;

    /// An `owl:complementOf` property links a class to precisely one class description. An
    /// `owl:complementOf` statement describes a class for which the class extension contains exactly
    /// those individuals that do not belong to the class extension of the class description that is
    /// the object of the statement. `owl:complementOf` is analogous to logical negation: the class
    /// extension consists of those individuals that are NOT members of the class extension of the
    /// complement class.
    complementOf << ObjectProperty;

    #[allow(clippy::doc_markdown)]
    /// Syntactically, `owl:inverseOf` is a built-in OWL property with `owl:ObjectProperty` as its
    /// domain and range. An axiom of the form $P_1$ `owl:inverseOf` $P_2$ asserts that for every
    /// pair $(x,y)$ in the property extension of $P_1$, there is a pair $(y,x)$ in the property
    /// extension of $P_2$, and vice versa. Thus, `owl:inverseOf` is a symmetric property.
    inverseOf :SymmetricProperty << ObjectProperty;

    /// A symmetric property is a property for which holds that if the pair $(x,y)$ is an instance of
    /// $P$, then the pair $(y,x)$ is also an instance of $P$. Syntactically, a property is defined
    /// as symmetric by making it an instance of the built-in OWL class `owl:SymmetricProperty`,
    /// a subclass of `owl:ObjectProperty`.
    SymmetricProperty :Class <: ObjectProperty;

    //AsymmetricProperty :Class <: ObjectProperty;

    /// When one defines a property $P$ to be a transitive property, this means that if a pair
    /// $(x,y)$ is an instance of $P$, and the pair $(y,z)$ is also instance of $P$, then we can
    /// infer the pair $(x,z)$ is also an instance of $P$.
    ///
    /// Syntactically, a property is defined as being transitive by making it an instance of the
    /// built-in OWL `class owl:TransitiveProperty`, which is defined as a subclass of
    /// `owl:ObjectProperty`.
    TransitiveProperty :Class <: ObjectProperty;

    #[allow(clippy::doc_markdown)]
    /// A functional property is a property that can have only one (unique) value $y$ for each
    /// instance $x$, i.e. there cannot be two distinct values $y_1$ and $y_2$ such that the pairs
    /// $(x,y_1)$ and $(x,y_2)$ are both instances of this property. Both object properties and
    /// datatype properties can be declared as "functional". For this purpose, OWL defines the
    /// built-in class `owl:FunctionalProperty` as a special subclass of the RDF class `rdf:Property`.
    FunctionalProperty :Class <: ::rdf::PROPERTY;
    /// Two OWL class identifiers are predefined, namely the classes `owl:Thing` and `owl:Nothing`.
    /// The class extension of `owl:Thing` is the set of all individuals.
    /// Consequently, every OWL class is a subclass of `owl:Thing`.
    Thing : Class;
}

dict! { ulo = "http://mathhub.info/ulo":
    { NAMESPACE <::dc::rights> S "This ontology is licensed under the CC-BY-SA license."};

    organizational : ::owl::DatatypeProperty;

    // ------------------------------------------------------------------------

    physical: ::owl::Class @ "An organizational unit for the physical organization of \
        mathematical knowledge into documents or document collections.";
    file <: physical @ "A document in a file system.";
    document <: physical @ "A document; typically corresponding to a file.";
    folder <: physical @ "A grouping of files and other folder, i.e. above the document level.";
    library <: physical @ "A grouping of mathematical documents. Usually in the \
        form of a repository.";
    library_group="library-group", <: physical @ "A group of libraries, usually on a \
        repository server like GitHub.";
    phrase <: physical @ "Phrasal structures in mathematical texts and formulae, \
        these include symbols, declarations, and quantifications.";
    section <: physical @ "A physical grouping inside a document. These can be nested.";

    slide <: physical @ "A slide in a presentation.";

    // ------------------------------------------------------------------------

    para <: physical @ "A document paragraph with mathematical meaning.";
    definition <: para,physical @ "A logical paragraph that defines a new concept.";
    example <: para,physical @ "A logical paragraph that introduces a mathematical example.";
    proof <: para,physical @ "A logical paragraph that serves as a justification of a proposition.";
    subproof <: para,physical @ "A logical paragraph that serves as a justification of an \
        intermediate proposition within a proof.";
    proposition <: para,physical @ "A statement of a mathematical object or some relation between some.";
    problem <: para,physical @ "A logical paragraph posing an exercise/question/problem for the reader.";
    subproblem <: para,physical @ "A logical paragraph posing a subproblem in some problem/question/problem \
        for the reader.";

    // ---------------------------------------------------------------------------

    logical: ::owl::Class !physical @ "A logical classification of mathematical \
        knowledge items.";
    primitive <: logical @ "This knowledge item does not have a definition in \
        terms of (more) primitive items." ;
    derived <: logical @ "This knowledge item has a definition in terms of (more) prmitive items";
    theory <: logical @ "A semantically meaningful block of declarations that can \
        be referred to globally. Examples include FTML modules, MMT theories, Mizar articles, \
        Isabelle locales and Coq sections.";
    structure <: logical @ "A semantically meaningful block of declarations that can \
        be instantiated by providing definientia for all (undefined) declarations.";
    morphism <: logical @ "A semantically meaningful block of declarations that map \
        map declarations in the domain to expressions over the containing module";
    variable <: logical @ "A local variable with optional type and definiens";
    notation <: logical @ "A way of representing (an application of) a symbol\
        for parsing or presentation.";
    function <: logical @ "Functions that construct objects, possibly from other \
        objects, for example in first-order logic the successor function.";
    r#type = "type", <: logical @ "Types divide their universe into named subsets.";
    universe <: logical @ "A universe, used e.g. in strong logics like Coq.";
    predicate <: function,logical @ "A predicate is a mathematical object that \
        evaluates to true/false when applied to enough arguments.";

    term <: logical;

    // ---------------------------------------------------------------------------

    declaration <: logical @ "Declarations are named objects. They can also \
        have a type and a definiens.";
    statement <: declaration @ "Statements are declarations of \
        objects that can in principle have proofs.";
    axiom <: statement,declaration @ "Logically (using the Curry-Howard isomorphism), an axiom \
        is a primitive statement, i.e. a declaration without a definiens.";
    theorem <: statement,declaration @ "Logically (using the Curry-Howard isomorphism), a \
        theorem is a derived statement, i.e. a declaration with a definiens (this is the proof of \
        the theorem given in the type)";
    rule <: statement @  "Rules are statements that can be used for computation, \
        e.g. theorems that can be used for simplification.";

    function_declaration = "function-declaration", <: declaration,function ;
    type_declaration = "type-declaration", <: declaration,r#type ;
    universe_declaration = "universe-declaration", <: declaration,universe ;

    // -----------------------------------------------------------------------------

    contains: ::owl::ObjectProperty (physical -> physical);
    declares: ::owl::ObjectProperty (logical -> logical);
    has_type  = "has-type",: ::owl::ObjectProperty (logical -> logical) ;
    specifies: ::owl::ObjectProperty (physical -> logical) -specified_in @ "The physical \
        organizational item S specifies a knowledge item O, i.e. S is represented in O.";
    specified_in = "specified-in", : ::owl::ObjectProperty (logical -> physical) -specifies;

    has_meta_theory = "has-meta-theory", : ::owl::ObjectProperty (theory -> theory);
    has_signature = "has-signature", : ::owl::DatatypeProperty;

    crossrefs: ::owl::ObjectProperty;
    aligned_with = "aligned-with", : ::owl::ObjectProperty,::owl::SymmetricProperty << crossrefs;
    same_as = "same-as", : ::owl::ObjectProperty,::owl::SymmetricProperty << crossrefs,aligned_with;
    similar_to = "similar-to", : ::owl::ObjectProperty,::owl::SymmetricProperty << crossrefs;
    alternative_for = "alternative-for", : ::owl::ObjectProperty << crossrefs;
    inspired_by = "inspired-by", : ::owl::ObjectProperty << crossrefs;
    see_also = "see-also", : ::owl::ObjectProperty << crossrefs;

    imports : ::owl::ObjectProperty,::owl::TransitiveProperty (logical -> logical) << crossrefs
        @ "This theory is an extension of this other theory, in that every expression \
        over the latter is a valid expression over this";

    // -----------------------------------------------------------------------------

    inter_statement = "inter-statement", : ::owl::ObjectProperty;
    constructs : ::owl::ObjectProperty << inter_statement @ "S is a constructor for an inductive type or predicate O";
    extends : ::owl::ObjectProperty << inter_statement @ "S is a conservative extension of O";
    example_for = "example-for", : ::owl::ObjectProperty !counter_example_for << inter_statement;
    counter_example_for = "counter-example-for", : ::owl::ObjectProperty !example_for << inter_statement;
    defines : ::owl::ObjectProperty  (definition -> function) << inter_statement @ "A definition defines various objects.";
    generated_by = "generated-by", : ::owl::ObjectProperty (function -> function) << inter_statement;
    inductive_on = "inductive-on", : ::owl::ObjectProperty << inter_statement;
    justifies : ::owl::ObjectProperty (proof -> statement) << inter_statement;
    notation_for = "notation-for", : ::owl::ObjectProperty (notation -> function) << inter_statement;

    precondition = "precondition-pair", : ::owl::Class;
    objective = "objective-pair", : ::owl::Class;

    cognitive_dimension = "bloom-dimension", : ::owl::Class;

    has_precondition = "precondition",;
    has_objective = "objective",;

    has_cognitive_dimension = "cognitive-dimension", : ::owl::ObjectProperty;
    po_has_symbol = "po-symbol", : ::owl::ObjectProperty;

    remember = "cd-remember", : cognitive_dimension;
    understand = "cd-understand", : cognitive_dimension;
    apply = "cd-apply", : cognitive_dimension;
    analyze = "cd-analyze", : cognitive_dimension;
    evaluate = "cd-evaluate", : cognitive_dimension;
    create = "cd-create", : cognitive_dimension;
    /*


    // -----------------------------------------------------------------------------

    OBJPROP NYMS = "nyms";
    OBJPROP ANTONYM = "antonym" <: NYMS;
    OBJPROP HYPONYM = "hyponym" <: NYMS;
    OBJPROP HYPERNYM = "hypernym" <: NYMS -HYPONYM;

    // -----------------------------------------------------------------------------

    OBJPROP FORMALIZES = "formalizes";
    OBJPROP USES = "uses" (STATEMENT => FUNCTION);
    { USES <super::rdfs::RANGE> <TYPE>};
    { USES <super::rdf::TYPE> <super::owl::TRANSITIVE_PROPERTY>};

    OBJPROP INSTANCE_OF = "instance-of" @ "S is an instance of O iff it is a model of O, iniherits \
        from O, interprets O, etc.";

    OBJPROP SUPERSEDED_BY = "superseded-by" @ "S (a deprecated knowledge item) is superseded by another.";
    { SUPERSEDED_BY <super::rdf::TYPE> <super::owl::TRANSITIVE_PROPERTY>};

    // -----------------------------------------------------------------------------

    DATAPROP SIZE_PROPERTIES = "size-properties";
    { SIZE_PROPERTIES <super::rdfs::DOMAIN> <super::owl::THING>};
    { SIZE_PROPERTIES <super::rdf::TYPE> <super::owl::FUNCTIONAL_PROPERTY>};

    DATAPROP AUTOMATICALLY_PROVED = "automatically-proved" <: ORGANIZATIONAL : super::xsd::STRING
        @ "S is automatically proven by a theorem prover, O is an explanatory string.";
    DATAPROP CHECK_TIME = "check-time" <: SIZE_PROPERTIES : super::xsd::DAY_TIME_DURATION
        @ "time it took to check the declaration that introduced the subject.";
    { CHECK_TIME <super::rdfs::DOMAIN> <FUNCTION>};
    { CHECK_TIME <super::rdfs::DOMAIN> <TYPE>};
    DATAPROP DEPRECATED = "deprecated" <: ORGANIZATIONAL : super::xsd::STRING
        @ "S is deprecated (do not use any longer), O is an explanatory string.";
    DATAPROP LAST_CHECKED_AT = "last-checked-at" <: SIZE_PROPERTIES : super::xsd::DATE_TIME_STAMP
        @ "The time stamp of when the subject was last checked.";
    { LAST_CHECKED_AT <super::rdfs::DOMAIN> <FUNCTION>};
    { LAST_CHECKED_AT <super::rdfs::DOMAIN> <TYPE>};
    DATAPROP SOURCEREF = "sourceref" : super::xsd::ANY_URI @ "The URI of the physical \
        location (e.g. file/URI, line, column) of the source code that introduced the subject.";


 */
}
