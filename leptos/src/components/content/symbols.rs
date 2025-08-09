use crate::{
    components::content::FtmlViewable,
    config::FtmlConfigState,
    utils::{
        Header, LocalCacheExt,
        block::{Block, HeaderLeft, HeaderRight},
    },
};
use ftml_backend::ParagraphOrProblemKind;
use ftml_core::extraction::VarOrSym;
use ftml_dom::{
    notations::{NotationExt, TermExt},
    utils::{
        css::inject_css,
        local_cache::{GlobalLocal, LocalCache, SendBackend},
    },
};
use ftml_ontology::{
    domain::declarations::symbols::{ArgumentSpec, Symbol, SymbolData},
    narrative::elements::Notation,
    terms::Variable,
};
use ftml_uris::{DocumentElementUri, IsNarrativeUri, LeafUri, SymbolUri};
use leptos::{html::span, prelude::*};
use thaw::{Text, TextTag};

impl super::FtmlViewable for Symbol {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Be> {
        let Self { uri, data } = self;
        let SymbolData {
            arity,
            macroname,
            role,
            tp,
            df,
            return_type,
            argument_types,
            assoctype,
            reordering,
        } = &**data;
        let symbol_str = if role.iter().any(|s| s.as_ref() == "textsymdecl") {
            "Text Symbol "
        } else {
            "Symbol "
        };
        let name = span()
            .child(uri.name().last().to_string())
            .title(uri.to_string());
        let macroname = macroname.as_ref().map(|name| {
            let name = name.to_string();
            view!(<span>" ("<Text tag=TextTag::Code>"\\"{name}</Text>")"</span>)
        });
        let tp = tp.as_ref().map(|t| {
            let t = t.clone().into_view::<crate::Views<Be>, Be>(false);
            view! {" of type "<math>{t}</math>}
        });
        let df = df.as_ref().map(|t| {
            let t = t.clone().into_view::<crate::Views<Be>, Be>(false);
            view! {"Definiens: "<math style="white-space:nowrap;">{t}</math>}
        });
        let header = view! {
            <b>{symbol_str}{name}</b>
            {macroname}
            {tp}
        };
        let notations = do_notations::<Be>(LeafUri::Symbol(uri.clone()), arity.clone());
        let paragraphs = do_paragraphs::<Be>(uri.clone());
        view! {
            <Block show_separator = true>
                <Header slot>{header}</Header>
                <HeaderLeft slot>{notations}</HeaderLeft>
                <HeaderRight slot>{df}</HeaderRight>
                {paragraphs}
                //<Footer slot>"moar"</Footer>
            </Block>
        }
        /*
        let uriclone = uri.clone();
        let uriclone_b = uri.clone();
        view! {
            <Block show_separator>
                <Header slot><span>
                    <b>{symbol_str}{super::symbol_name(&uri, uri.name().last_name().as_ref())}</b>
                    {macro_name.map(|name| view!(<span>" ("<Text tag=TextTag::Code>"\\"{name}</Text>")"</span>))}
                    {tp.map(|t| view! {
                        " of type "{
                            crate::remote::get!(present(t.clone()) = html => {
                                view!(<FTMLStringMath html/>)
                            })
                        }
                    })}
                </span></Header>
                <HeaderLeft slot>{do_notations(URI::Content(uriclone_b.into()),arity)}</HeaderLeft>
                <HeaderRight slot><span style="white-space:nowrap;">{df.map(|t| view! {
                    "Definiens: "{
                        crate::remote::get!(present(t.clone()) = html => {
                            view!(<FTMLStringMath html/>)
                        })
                    }
                })}</span></HeaderRight>
                {do_los(uriclone)}
            </Block>
        }
         */
    }
}

pub(super) fn do_paragraphs<Be: SendBackend>(uri: SymbolUri) -> impl IntoView {
    use crate::utils::collapsible::LazyCollapsible;

    let cached = move || {
        let uri = uri.clone();
        LocalCache::with_or_toast::<Be, _, _, _, _>(
            move |be| async move { Ok(be.get_paragraphs(uri, false).await) },
            |ps| {
                let mut definitions = Vec::new();
                let mut examples = Vec::new();
                for (uri, kind) in ps.into_iter() {
                    match kind {
                        ParagraphOrProblemKind::Definition => definitions.push(uri),
                        ParagraphOrProblemKind::Example => examples.push(uri),
                        _ => (),
                    }
                }
                view! {
                    {super::CommaSep("Definitions",definitions).into_view::<Be>()}
                    <br/>
                    {super::CommaSep("Examples",examples).into_view::<Be>()}
                }
            },
            || "(errored)",
        )
    };
    view! {
            <LazyCollapsible>
                <Header slot><span>"Associated Paragraphs"</span></Header>
                <div style="padding-left:15px">
                { cached() }
    /*
                {
                    let uri = uri.clone();
                    crate::remote::get!(get_los(uri.clone(),true) = v => {
                        let LOs {definitions,examples,problems} = v.lo_sort();
                        view!{
                            <div>{if definitions.is_empty() { None } else {Some(
                                super::comma_sep("Definitions", definitions.into_iter().map(|uri| {
                                    let title = uri.name().last().to_string();
                                    super::doc_elem_name(uri,None,title)
                                }))
                            )}}</div>
                            <div>{if examples.is_empty() { None } else {Some(
                                super::comma_sep("Examples", examples.into_iter().map(|uri| {
                                    let title = uri.name().last().to_string();
                                    super::doc_elem_name(uri,None,title)
                                }))
                            )}}</div>
                            <div>{if problems.is_empty() { None } else {Some(
                                super::comma_sep("Problems", problems.into_iter().map(|(_,uri,cd)| {
                                    let title = uri.name().last().to_string();
                                    view!{
                                        {super::doc_elem_name(uri,None,title)}
                                        " ("{cd.to_string()}")"
                                    }
                                }))
                            )}}</div>
                        }
                    })
                } */
                </div>
            </LazyCollapsible>
        }
}

pub(super) fn do_notations<Be: SendBackend>(
    uri: LeafUri,
    arity: ArgumentSpec,
) -> impl IntoView + use<Be> {
    //let functional = arity.num() > 0;
    //let as_variable = matches!(uri, LeafUri::Element(_));
    let var_or_sym = match &uri {
        LeafUri::Element(e) => VarOrSym::V(Variable::Ref {
            declaration: e.clone(),
            is_sequence: None,
        }),
        LeafUri::Symbol(s) => VarOrSym::S(s.clone()),
    };
    inject_css("ftml-notation-table", include_str!("notations.css"));
    LocalCache::with_or_toast::<Be, _, _, _, _>(
        move |b| async move { Ok(b.get_notations(uri).await) },
        move |nots| do_table::<Be, _>(var_or_sym, arity, nots),
        || "(errored)",
    )
}

fn do_table<Be: SendBackend, E>(
    head: VarOrSym,
    arity: ArgumentSpec,
    nots: GlobalLocal<Vec<(DocumentElementUri, Notation)>, E>,
) -> impl IntoView + use<Be, E> {
    use thaw::{Popover, PopoverTrigger, Table, TableCell, TableHeader, TableHeaderCell, TableRow};
    fn render_not<Be: SendBackend>(
        head: &VarOrSym,
        arity: &ArgumentSpec,
        not_uri: DocumentElementUri,
        not: &Notation,
    ) -> impl IntoView + use<Be> {
        let functional = arity.num() > 0;
        let notation = not.as_view_safe::<crate::Views<Be>>(head);
        let op = if functional {
            let op = not.as_op_safe::<crate::Views<Be>>(head);
            Some(view! {<TableCell class="ftml-notation-cell">{op}</TableCell>})
        } else {
            None
        };
        let notation2 = not.as_view_safe::<crate::Views<Be>>(head);
        view! {<TableCell class="ftml-notation-cell">
            <Popover>
                <PopoverTrigger slot><span>{notation}</span></PopoverTrigger>
                <Table class="ftml-notation-table">
                    <TableHeader>
                        <TableRow>
                            <TableHeaderCell class="ftml-notation-header">"source document"</TableHeaderCell>
                            {if functional {Some(view!{<TableHeaderCell class="ftml-notation-header">"operation"</TableHeaderCell>})} else {None}}
                            <TableHeaderCell class="fftml-notation-header">"notation"</TableHeaderCell>
                        </TableRow>
                    </TableHeader>
                    <TableRow>
                        <TableCell class="ftml-notation-cell">{not_uri.document_uri().as_view::<Be>()}</TableCell>
                        {op}
                        <TableCell class="ftml-notation-cell">
                        {notation2}
                        </TableCell>
                    </TableRow>
                </Table>
            </Popover>
        </TableCell>}
    }
    FtmlConfigState::disable_hovers(move || {
        let notations = nots
            .into_iter()
            .map(|(k, v)| render_not::<Be>(&head, &arity, k, &v))
            .collect::<Vec<_>>();
        if notations.is_empty() {
            return None;
        }
        Some(view! {
            <div>
                <Table class="ftml-notation-table"><TableRow>
                    <TableCell class="ftml-notation-header"><span>"Notations: "</span></TableCell>
                {notations.collect_view()}
                </TableRow></Table>
            </div>
        })
    })
}
