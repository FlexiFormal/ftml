use std::{borrow::Cow, str::FromStr};

use ftml_ontology::terms::{VarOrSym, Variable};
use ftml_uris::{
    DocumentElementUri, DocumentUri, DomainUri, Id, Language, ModuleUri, SymbolUri, Uri,
};

use super::Result;
use crate::{
    FtmlKey,
    extraction::{FtmlExtractionError, FtmlExtractor},
};

pub trait Attributes {
    type Ext: FtmlExtractor;
    /*type KeyIter<'a>: Iterator<Item = FtmlKey>
    where
        Self: 'a;*/
    type Value<'a>: AsRef<str> + Into<Cow<'a, str>> + Into<String>
    where
        Self: 'a;
    //fn keys(&self) -> Self::KeyIter<'_>;
    fn value(&self, key: &str) -> Option<Self::Value<'_>>;
    fn set(&mut self, key: &str, value: impl std::fmt::Display);
    fn take(&mut self, key: &str) -> Option<String>;

    #[inline]
    fn get(&self, tag: FtmlKey) -> Option<Self::Value<'_>> {
        self.value(tag.attr_name())
    }
    #[inline]
    fn remove(&mut self, tag: FtmlKey) -> Option<String> {
        self.take(tag.attr_name())
    }

    /// #### Errors
    fn get_typed<T, E>(
        &self,
        key: FtmlKey,
        f: impl FnOnce(&str) -> std::result::Result<T, E>,
    ) -> Result<T>
    where
        (FtmlKey, E): Into<FtmlExtractionError>,
    {
        let Some(v) = self.get(key) else {
            return Err(FtmlExtractionError::MissingKey(key));
        };
        if v.as_ref().is_empty() {
            return Err(FtmlExtractionError::MissingKey(key));
        }
        f(v.as_ref().trim()).map_err(|e| (key, e).into())
    }

    /// #### Errors
    fn get_typed_vec<E, T>(
        &self,
        key: FtmlKey,
        mut f: impl FnMut(&str) -> Result<T>,
    ) -> Result<Vec<T>> {
        let Some(v) = self.get(key) else {
            return Err(FtmlExtractionError::MissingKey(key));
        };
        let mut vec = Vec::new();
        for e in v.as_ref().split(',') {
            vec.push(f(e.trim())?);
        }
        Ok(vec)
    }

    /// #### Errors
    fn take_typed<T, E>(
        &mut self,
        key: FtmlKey,
        f: impl FnOnce(&str) -> std::result::Result<T, E>,
    ) -> Result<T>
    where
        (FtmlKey, E): Into<FtmlExtractionError>,
    {
        let Some(v) = self.remove(key) else {
            return Err(FtmlExtractionError::MissingKey(key));
        };
        f(v.as_str().trim()).map_err(|e| (key, e).into())
    }

    // --------------------------------------------------------------------

    /// #### Errors
    #[inline]
    fn get_language(&self, key: FtmlKey) -> Result<Language> {
        self.get_typed(key, |l| {
            Language::from_str(l)
                .map_err(|_| FtmlExtractionError::InvalidLanguage(key, l.to_string()))
        })
    }

    /// #### Errors
    #[inline]
    fn take_language(&mut self, key: FtmlKey) -> Result<Language> {
        self.take_typed(key, |l| {
            Language::from_str(l)
                .map_err(|_| FtmlExtractionError::InvalidLanguage(key, l.to_string()))
        })
    }

    /// #### Errors
    #[inline]
    fn get_module_uri(&self, key: FtmlKey) -> Result<ModuleUri> {
        self.get_typed(key, ModuleUri::from_str)
    }

    /// #### Errors
    #[inline]
    fn take_module_uri(&mut self, key: FtmlKey) -> Result<ModuleUri> {
        self.take_typed(key, ModuleUri::from_str)
    }

    /// #### Errors
    #[inline]
    fn get_symbol_uri(&self, key: FtmlKey) -> Result<SymbolUri> {
        self.get_typed(key, SymbolUri::from_str)
    }

    /// #### Errors
    #[inline]
    fn take_symbol_uri(&mut self, key: FtmlKey) -> Result<SymbolUri> {
        self.take_typed(key, SymbolUri::from_str)
    }

    /// #### Errors
    #[inline]
    fn take_symbol_or_module_uri(&mut self, key: FtmlKey) -> Result<ModuleUri> {
        match self.take_typed(key, DomainUri::from_str)? {
            DomainUri::Module(m) => Ok(m),
            DomainUri::Symbol(s) => Ok(s.into_module()),
        }
    }

    /// ### Errors
    fn get_symbol_or_var(
        &mut self,
        key: FtmlKey,
        ext: &mut impl FtmlExtractor,
    ) -> Result<VarOrSym> {
        let Some(headv) = self.get(key) else {
            return Err(FtmlExtractionError::MissingKey(key));
        };
        let head = headv.as_ref().trim();
        if head.contains('?') {
            let uri = head.parse::<Uri>().map_err(|e| (key, e))?;
            match uri {
                Uri::Symbol(s) => Ok(VarOrSym::Sym(s)),
                Uri::Module(m) => {
                    let Some(s) = m.into_symbol() else {
                        return Err(FtmlExtractionError::InvalidValue(key));
                    };
                    Ok(VarOrSym::Sym(s))
                }
                //Uri::Module(_) => VarOrSym::S(m.into()) ???
                Uri::DocumentElement(e) => Ok(VarOrSym::Var(Variable::Ref {
                    declaration: e,
                    is_sequence: None,
                })),
                _ => Err(FtmlExtractionError::InvalidValue(key)),
            }
        } else {
            Ok(VarOrSym::Var(ext.resolve_variable_name(
                head.parse().map_err(|e| (key, e))?,
            )))
        }
    }

    /// #### Errors
    #[inline]
    fn get_document_uri(&self, key: FtmlKey) -> Result<DocumentUri> {
        self.get_typed(key, DocumentUri::from_str)
    }

    /// #### Errors
    #[inline]
    fn take_document_uri(&mut self, key: FtmlKey) -> Result<DocumentUri> {
        self.take_typed(key, DocumentUri::from_str)
    }

    fn get_bool(&self, key: FtmlKey) -> bool {
        self.get(key)
            .and_then(|s| s.as_ref().parse().ok())
            .unwrap_or_default()
    }

    fn take_bool(&mut self, key: FtmlKey) -> bool {
        self.remove(key)
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }

    /// #### Errors
    fn get_new_module_uri(
        &self,
        key: FtmlKey,
        in_elem: FtmlKey,
        extractor: &mut Self::Ext,
    ) -> Result<ModuleUri> {
        let Some(v) = self.get(key) else {
            return Err(FtmlExtractionError::MissingKey(key));
        };
        extractor.get_domain_uri(in_elem).map_or_else(
            |_| {
                extractor
                    .in_document()
                    .module_uri_from(v.as_ref())
                    .map_err(|e| (key, e).into())
            },
            |m| {
                v.as_ref()
                    .parse()
                    .map(|v| m.into_owned() / &v)
                    .map_err(|e| (key, e).into())
            },
        )
    }
    /// #### Errors
    fn take_new_module_uri(
        &mut self,
        key: FtmlKey,
        in_elem: FtmlKey,
        extractor: &mut Self::Ext,
    ) -> Result<ModuleUri> {
        let Some(v) = self.remove(key) else {
            return Err(FtmlExtractionError::MissingKey(key));
        };
        extractor.get_domain_uri(in_elem).map_or_else(
            |_| {
                extractor
                    .in_document()
                    .module_uri_from(v.as_ref())
                    .map_err(|e| (key, e).into())
            },
            |m| {
                v.parse()
                    .map(|v| m.into_owned() / &v)
                    .map_err(|e| (key, e).into())
            },
        )
    }

    /// #### Errors
    fn get_new_symbol_uri(
        &self,
        key: FtmlKey,
        in_elem: FtmlKey,
        extractor: &mut Self::Ext,
    ) -> Result<SymbolUri> {
        let Some(v) = self.get(key) else {
            return Err(FtmlExtractionError::MissingKey(key));
        };
        let module = extractor.get_domain_uri(in_elem)?;
        v.as_ref()
            .parse()
            .map(|v| module.into_owned() | v)
            .map_err(|e| (key, e).into())
    }

    /// #### Errors
    fn take_new_symbol_uri(
        &mut self,
        key: FtmlKey,
        in_elem: FtmlKey,
        extractor: &mut Self::Ext,
    ) -> Result<SymbolUri> {
        let Some(v) = self.remove(key) else {
            return Err(FtmlExtractionError::MissingKey(key));
        };
        let module = extractor.get_domain_uri(in_elem)?;
        v.parse()
            .map(|v| module.into_owned() | v)
            .map_err(|e| (key, e).into())
    }

    /// #### Errors
    fn get_elem_uri_from_id(
        &mut self,
        extractor: &mut Self::Ext,
        prefix: impl Into<Cow<'static, str>>,
    ) -> Result<DocumentElementUri> {
        let id: Id = if let Some(id) = self.get(FtmlKey::Id) {
            id.as_ref()
                .parse()
                .map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::Id))?
        } else {
            extractor.new_id(FtmlKey::Id, prefix)?
        };
        let curr_uri = extractor.get_narrative_uri();

        Ok(curr_uri & &id)
    }
}
