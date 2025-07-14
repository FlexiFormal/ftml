# ULO - Upper Library Ontology

The ULO implemented as [oxrdf](https://docs.rs/oxrdf) constants; see [this paper](https://kwarc.info/people/mkohlhase/papers/cicm19-ulo.pdf).

## RDF Ontology Summary

#### [`Document`](crate::narration::documents::Document) `D`
| struct | field | triple |
| -----  | ----- | ------ |
|   |    | `D` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#document>`](ulo::document) |
|   | language `l` | `D` [`<dc:#language>`](dc::language) `l` |
|   | in archive `A`  | `A` [`<ulo:#contains>`](ulo::contains) `D` |
| [`DocumentReference`](crate::narration::DocumentElement::DocumentReference) | [`.target`](crate::narration::DocumentElement::DocumentReference::target)`=D2` | `D` [`<dc:#hasPart>`](dc::hasPart) `D2` |
| [`UseModule`](crate::narration::DocumentElement::UseModule) | `(M)` | `D` [`<dc:#requires>`](dc::requires) `M` |
| [`Paragraph`](crate::narration::paragraphs::LogicalParagraph) |   | `D` [`<ulo:#contains>`](ulo::contains) `P` |
|   | `P`[`.kind`](crate::narration::paragraphs::LogicalParagraph::kind)`=`[`Definition`](crate::narration::paragraphs::ParagraphKind::Definition) | `P` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#definition>`](ulo::definition) |
|   | `P`[`.kind`](crate::narration::paragraphs::LogicalParagraph::kind)`=`[`Assertion`](crate::narration::paragraphs::ParagraphKind::Assertion) | `P` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#proposition>`](ulo::proposition) |
|   | `P`[`.kind`](crate::narration::paragraphs::LogicalParagraph::kind)`=`[`Paragraph`](crate::narration::paragraphs::ParagraphKind::Paragraph) | `P` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#para>`](ulo::para) |
|   | `P`[`.kind`](crate::narration::paragraphs::LogicalParagraph::kind)`=`[`Example`](crate::narration::paragraphs::ParagraphKind::Example) | `P` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#example>`](ulo::example) |
|   | is [`Example`](crate::narration::paragraphs::ParagraphKind::Example) and `_`[`.fors`](crate::narration::paragraphs::LogicalParagraph::fors)`.contains(S)`  | `P` [`<ulo:#example-for>`](ulo::example_for) `S` |
|   | [`is_definition_like`](crate::narration::paragraphs::ParagraphKind::is_definition_like) and  `_`[`.fors`](crate::narration::paragraphs::LogicalParagraph::fors)`.contains(S)`  | `P` [`<ulo:#defines>`](ulo::defines) `S` |
| [`Problem`](crate::narration::problems::Problem) `E` |   | `D` [`<ulo:#contains>`](ulo::contains) `E` |
|   | [`.sub_problem`](crate::narration::problems::Problem::sub_problem)`==false`   | `E` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#problem>`](ulo::problem) |
|   | [`.sub_problem`](crate::narration::problems::Problem::sub_problem)`==true`   | `E` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#subproblem>`](ulo::subproblem) |
|   | `_`[`.preconditions`](crate::narration::problems::Problem::preconditions)`.contains(d,S)`  | `E` [`<ulo:#precondition>`](ulo::precondition) `<BLANK>` |
|   |    | `<BLANK>` [`<ulo:#cognitive-dimension>`](ulo::cognitive_dimension) `d`, where `d=`[`<ulo:#cs-remember>`](ulo::remember)⏐[`<ulo:#cs-understand>`](ulo::understand)⏐[`<ulo:#cs-apply>`](ulo::apply)⏐[`<ulo:#cs-analyze>`](ulo::analyze)⏐[`<ulo:#cs-evaluate>`](ulo::evaluate)⏐[`<ulo:#cs-create>`](ulo::create) |
|   |    | `<BLANK>` [`<ulo:#po-symbol>`](ulo::po_has_symbol) `S` |
|   | `_`[`.objectives`](crate::narration::problems::Problem::objectives)`.contains(d,S)`  | `E` [`<ulo:#objective>`](ulo::objective) `<BLANK>` |
|   |    | `<BLANK>` [`<ulo:#cognitive-dimension>`](ulo::cognitive_dimension) `d`, where `d=`[`<ulo:#cs-remember>`](ulo::remember)⏐[`<ulo:#cs-understand>`](ulo::understand)⏐[`<ulo:#cs-apply>`](ulo::apply)⏐[`<ulo:#cs-analyze>`](ulo::analyze)⏐[`<ulo:#cs-evaluate>`](ulo::evaluate)⏐[`<ulo:#cs-create>`](ulo::create) |
|   |    | `<BLANK>` [`<ulo:#po-symbol>`](ulo::po_has_symbol) `S` |

#### [`Module`](crate::content::modules::Module) `M`
| struct | field | triple |
| -----  | ----- | ------ |
|   |    | `D` [`<ulo:#contains>`](ulo::contains) `M` |
|   |    | `M` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#theory>`](ulo::theory) |
| [`Import`](crate::content::declarations::OpenDeclaration::Import) | `(M2)` | `M` [`<ulo:#imports>`](ulo::imports) `M2` |
| [`NestedModule`](crate::content::declarations::OpenDeclaration::NestedModule) | `(M2)` | `D` [`<ulo:#contains>`](ulo::contains) `M2` |
|   |    | `M` [`<ulo:#contains>`](ulo::contains) `M2` |
|   |    | `M2` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#theory>`](ulo::theory) |
| [`MathStructure`](crate::content::declarations::OpenDeclaration::MathStructure) | `(S)` | `M` [`<ulo:#contains>`](ulo::contains) `S` |
|   |    | `S` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#structure>`](ulo::structure) |
|   | [`Import`](crate::content::declarations::OpenDeclaration::Import)(`S2`)   | `S` [`<ulo:#extends>`](ulo::extends) `S2` |
| [`Morphism`](crate::content::declarations::OpenDeclaration::Morphism) | `(F)` | `M` [`<ulo:#contains>`](ulo::contains) `F` |
|   |    | `F` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#morphism>`](ulo::morphism) |
|   | [`.domain`](crate::content::declarations::morphisms::Morphism)`=M2`   | `F` [`<rdfs:#domain>`](rdfs::DOMAIN) `M2` |



# Some Example Queries

#### Unused files in `ARCHIVE`:
All elements contained in the archive that are neither inputrefed elsewhere
nor (transitively) contain an element that is required or imported (=> is a module)
by another document:
```sql
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
