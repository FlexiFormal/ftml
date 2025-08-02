use crate::utils::{
    Header,
    block::{Block, HeaderLeft, HeaderRight},
};
use ftml_dom::{notations::TermExt, utils::local_cache::SendBackend};
use ftml_ontology::domain::declarations::symbols::{Symbol, SymbolData};
use leptos::{html::span, prelude::*};
use thaw::{Text, TextTag};

pub trait FtmlViewable {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Self, Be>;
}

impl FtmlViewable for Symbol {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Be> {
        let Self { uri, data } = self;
        let SymbolData {
            arity,
            macroname,
            role,
            tp,
            df,
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
        view! {
            <Block show_separator = true>
                <Header slot>{header}</Header>
                <HeaderLeft slot>"(Notations)"</HeaderLeft>
                <HeaderRight slot>{df}</HeaderRight>
                "(paragraphs)"
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
