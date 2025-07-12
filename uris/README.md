# FTML URIs

[FTML](https://mathhub.info/?a=Papers%2F25-CICM-FLAMS&d=paper&l=en) URIs are intended to serve as hierarchical,
globally unique identifiers, are used as keys for retrieving identified content elements, and occur in large
numbers in FTML documents. As such, it is important that they are fast to parse, clone, equality-check,
(de)serialize, and ideally are parsimonious with respect to memory usage.

Naturally, these desiderata are contradictory. Hence, as a tradeoff, we
- intern [Uri]s and Uri *components* for deduplication,
- use [strumbra](strumbra::SharedString) strings to keep allocations infrequent,
- use [Arc](triomphe::Arc)s where heap is unavoidable
- use pointer-equality (thanks to interning) for fast equality checks

## Grammar

| Type  |     | Cases/Def | Trait |
|----------- |---- | -----|-------|
| [`Uri`]      | ::= | [`BaseUri`]⏐[`ArchiveUri`]⏐[`PathUri`]⏐[`ModuleUri`]⏐[`SymbolUri`]⏐[`DocumentUri`]⏐[`DocumentElementUri`] | [`IsFtmlUri`] |
| [`BaseUri`]  | ::= | (URL with no query/fragment) | - |
| [`ArchiveUri`] | ::= | <code>[BaseUri]?a=[ArchiveId]</code> | [`UriWithArchive`] |
| [`PathUri`]  | ::= | <code>[ArchiveUri][&p=[UriPath]]</code> | [`UriWithPath`] |
| [`DomainUri`] | ::= | [`ModuleUri`]⏐[`SymbolUri`]   | [`IsDomainUri`] |
| [`ModuleUri`] | ::= | <code>[PathUri]&m=[UriName]&l=[Language]</code> | - |
| [`SymbolUri`] | ::= | <code>[ModuleUri]&s=[UriName]</code> | - |
| [`NarrativeUri`] | ::= | [`DocumentUri`]⏐[`DocumentElementUri`] | [`IsNarrativeUri`] |
| [`DocumentUri`] | ::= | <code>[PathUri]&d=[SimpleUriName]&l=[Language]</code> | - |
| [`DocumentElementUri`] | ::= | <code>[DocumentUri]&e=[UriName]</code> | - |
