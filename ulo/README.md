# ULO - Upper Library Ontology

## RDF Ontology Summary

#### [`Document`](crate::narration::documents::Document) `D`
| struct | field | triple |
| -----  | ----- | ------ |
|   |    | `D` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#document>`](ulo2::DOCUMENT) |
|   | language `l` | `D` [`<dc:#language>`](dc::LANGUAGE) `l` |
|   | in archive `A`  | `A` [`<ulo:#contains>`](ulo2::CONTAINS) `D` |
| [`DocumentReference`](crate::narration::DocumentElement::DocumentReference) | [`.target`](crate::narration::DocumentElement::DocumentReference::target)`=D2` | `D` [`<dc:#hasPart>`](dc::HAS_PART) `D2` |
| [`UseModule`](crate::narration::DocumentElement::UseModule) | `(M)` | `D` [`<dc:#requires>`](dc::REQUIRES) `M` |
| [`Paragraph`](crate::narration::paragraphs::LogicalParagraph) |   | `D` [`<ulo:#contains>`](ulo2::CONTAINS) `P` |
|   | `P`[`.kind`](crate::narration::paragraphs::LogicalParagraph::kind)`=`[`Definition`](crate::narration::paragraphs::ParagraphKind::Definition) | `P` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#definition>`](ulo2::DEFINITION) |
|   | `P`[`.kind`](crate::narration::paragraphs::LogicalParagraph::kind)`=`[`Assertion`](crate::narration::paragraphs::ParagraphKind::Assertion) | `P` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#proposition>`](ulo2::PROPOSITION) |
|   | `P`[`.kind`](crate::narration::paragraphs::LogicalParagraph::kind)`=`[`Paragraph`](crate::narration::paragraphs::ParagraphKind::Paragraph) | `P` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#para>`](ulo2::PARA) |
|   | `P`[`.kind`](crate::narration::paragraphs::LogicalParagraph::kind)`=`[`Example`](crate::narration::paragraphs::ParagraphKind::Example) | `P` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#example>`](ulo2::EXAMPLE) |
|   | is [`Example`](crate::narration::paragraphs::ParagraphKind::Example) and `_`[`.fors`](crate::narration::paragraphs::LogicalParagraph::fors)`.contains(S)`  | `P` [`<ulo:#example-for>`](ulo2::EXAMPLE_FOR) `S` |
|   | [`is_definition_like`](crate::narration::paragraphs::ParagraphKind::is_definition_like) and  `_`[`.fors`](crate::narration::paragraphs::LogicalParagraph::fors)`.contains(S)`  | `P` [`<ulo:#defines>`](ulo2::DEFINES) `S` |
| [`Problem`](crate::narration::problems::Problem) `E` |   | `D` [`<ulo:#contains>`](ulo2::CONTAINS) `E` |
|   | [`.sub_problem`](crate::narration::problems::Problem::sub_problem)`==false`   | `E` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#problem>`](ulo2::PROBLEM) |
|   | [`.sub_problem`](crate::narration::problems::Problem::sub_problem)`==true`   | `E` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#subproblem>`](ulo2::SUBPROBLEM) |
|   | `_`[`.preconditions`](crate::narration::problems::Problem::preconditions)`.contains(d,S)`  | `E` [`<ulo:#precondition>`](ulo2::PRECONDITION) `<BLANK>` |
|   |    | `<BLANK>` [`<ulo:#cognitive-dimension>`](ulo2::COGDIM) `d`, where `d=`[`<ulo:#cs-remember>`](ulo2::REMEMBER)⏐[`<ulo:#cs-understand>`](ulo2::UNDERSTAND)⏐[`<ulo:#cs-apply>`](ulo2::APPLY)⏐[`<ulo:#cs-analyze>`](ulo2::ANALYZE)⏐[`<ulo:#cs-evaluate>`](ulo2::EVALUATE)⏐[`<ulo:#cs-create>`](ulo2::CREATE) |
|   |    | `<BLANK>` [`<ulo:#po-symbol>`](ulo2::POSYMBOL) `S` |
|   | `_`[`.objectives`](crate::narration::problems::Problem::objectives)`.contains(d,S)`  | `E` [`<ulo:#objective>`](ulo2::OBJECTIVE) `<BLANK>` |
|   |    | `<BLANK>` [`<ulo:#cognitive-dimension>`](ulo2::COGDIM) `d`, where `d=`[`<ulo:#cs-remember>`](ulo2::REMEMBER)⏐[`<ulo:#cs-understand>`](ulo2::UNDERSTAND)⏐[`<ulo:#cs-apply>`](ulo2::APPLY)⏐[`<ulo:#cs-analyze>`](ulo2::ANALYZE)⏐[`<ulo:#cs-evaluate>`](ulo2::EVALUATE)⏐[`<ulo:#cs-create>`](ulo2::CREATE) |
|   |    | `<BLANK>` [`<ulo:#po-symbol>`](ulo2::POSYMBOL) `S` |

#### [`Module`](crate::content::modules::Module) `M`
| struct | field | triple |
| -----  | ----- | ------ |
|   |    | `D` [`<ulo:#contains>`](ulo2::CONTAINS) `M` |
|   |    | `M` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#theory>`](ulo2::THEORY) |
| [`Import`](crate::content::declarations::OpenDeclaration::Import) | `(M2)` | `M` [`<ulo:#imports>`](ulo2::IMPORTS) `M2` |
| [`NestedModule`](crate::content::declarations::OpenDeclaration::NestedModule) | `(M2)` | `D` [`<ulo:#contains>`](ulo2::CONTAINS) `M2` |
|   |    | `M` [`<ulo:#contains>`](ulo2::CONTAINS) `M2` |
|   |    | `M2` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#theory>`](ulo2::THEORY) |
| [`MathStructure`](crate::content::declarations::OpenDeclaration::MathStructure) | `(S)` | `M` [`<ulo:#contains>`](ulo2::CONTAINS) `S` |
|   |    | `S` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#structure>`](ulo2::STRUCTURE) |
|   | [`Import`](crate::content::declarations::OpenDeclaration::Import)(`S2`)   | `S` [`<ulo:#extends>`](ulo2::EXTENDS) `S2` |
| [`Morphism`](crate::content::declarations::OpenDeclaration::Morphism) | `(F)` | `M` [`<ulo:#contains>`](ulo2::CONTAINS) `F` |
|   |    | `F` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#morphism>`](ulo2::MORPHISM) |
|   | [`.domain`](crate::content::declarations::morphisms::Morphism)`=M2`   | `F` [`<rdfs:#domain>`](rdfs::DOMAIN) `M2` |



# Some Example Queries

#### Unused files in `ARCHIVE`:
All elements contained in the archive that are neither inputrefed elsewhere
nor (transitively) contain an element that is required or imported (=> is a module)
by another document:
```sparql
SELECT DISTINCT ?f WHERE {
  <ARCHIVE> ulo:contains ?f .
  MINUS { ?d dc:hasPart ?f }
  MINUS {
    ?f ulo:contains+ ?m.
    ?d (dc:requires|ulo:imports) ?m.
  }
}
```

#### All referenced symbols in `DOCUMENT`:
All symbols referenced in an element that is transitively contained or inputrefed in
the document:
```sparql
SELECT DISTINCT ?s WHERE {
  <DOCUMENT> (ulo:contains|dc:hasPart)* ?p.
  ?p ulo:crossrefs ?s.
}
```

#### All symbols defined in a `DOCUMENT`:
All symbols defined by a paragraph `p` that is transitively contained or inputrefed in
the document:
```sparql
SELECT DISTINCT ?s WHERE {
  <DOCUMENT> (ulo:contains|dc:hasPart)* ?p.
  ?p ulo:defines ?s.
}
```

#### All "prerequisite" concepts in a `DOCUMENT`:
All symbols references in the document that are not also defined in it:
```sparql
SELECT DISTINCT ?s WHERE {
  <DOCUMENT> (ulo:contains|dc:hasPart)* ?p.
  ?p ulo:crossrefs ?s.
  MINUS {
    <DOCUMENT> (ulo:contains|dc:hasPart)* ?p.
    ?p ulo:defines ?s.
  }
}
```
