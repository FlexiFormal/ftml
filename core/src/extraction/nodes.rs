use std::borrow::Cow;

use crate::extraction::FtmlExtractionError;
use either::Either::{Left, Right};
use ftml_ontology::{
    narrative::DocumentRange,
    terms::{Term, opaque::Opaque},
};
use ftml_uris::Id;

pub trait FtmlNode: Clone {
    //type Ancestors<'a>: Iterator<Item = Self> where Self: 'a;
    //fn ancestors(&self) -> Self::Ancestors<'_>;
    //fn with_elements<R>(&mut self, f: impl FnMut(Option<&mut FTMLElements>) -> R) -> R;
    //fn string(&self) -> Cow<'_, str>;
    //fn inner_string(&self) -> Cow<'_, str>;
    //fn as_notation(&self) -> Option<NotationSpec>;
    //fn as_op_notation(&self) -> Option<OpNotation>;
    //fn as_term(&self) -> Term;
    //fn delete_children(&self);
    fn delete(&self);
    fn range(&self) -> DocumentRange;
    fn inner_range(&self) -> DocumentRange;
    fn path_from(&self, ancestor: &Self) -> crate::NodePath;
    fn children(&self) -> impl Iterator<Item = Option<either::Either<Self, String>>>;

    /// ### Errors
    fn tag_name(&self) -> Result<Cow<'_, str>, String>;

    fn iter_attributes(&self) -> impl Iterator<Item = Result<(Cow<'_, str>, String), String>>;

    /// ### Errors
    #[allow(clippy::type_complexity)]
    fn as_term(
        &self,
        termpairs: Vec<(Term, crate::NodePath)>,
    ) -> Result<Term, FtmlExtractionError> {
        fn rec<N: FtmlNode>(
            path: &mut crate::NodePath,
            paths: &[crate::NodePath],
            node: &N,
        ) -> Result<(Id, Box<[(Id, Box<str>)]>, Box<[Opaque]>), FtmlExtractionError> {
            let tag = node
                .tag_name()
                .map_err(FtmlExtractionError::InvalidInformal)?
                .parse()
                .map_err(|e| {
                    FtmlExtractionError::InvalidInformal(format!("invalid attribute: {e}"))
                })?;
            let attributes = node
                .iter_attributes()
                .map(|e| {
                    e.map_or_else::<Result<_, FtmlExtractionError>, _, _>(
                        |e| Err(FtmlExtractionError::InvalidInformal(e)),
                        |(k, v)| {
                            let k = k.parse::<Id>().map_err(|e| {
                                FtmlExtractionError::InvalidInformal(format!(
                                    "invalid attribute: {e}"
                                ))
                            })?;
                            let v = v.into_boxed_str();
                            Ok((k, v))
                        },
                    )
                })
                .collect::<Result<Vec<_>, FtmlExtractionError>>()?;
            let mut children = Vec::new();
            for (i, c) in node.children().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                if let Some((j, _)) = paths.iter().enumerate().find(|(_, p)| {
                    p.len() == path.len() + 1
                        && p.starts_with(path)
                        && p.last().is_some_and(|j| *j == i as u32)
                }) {
                    #[allow(clippy::cast_possible_truncation)]
                    children.push(Opaque::Term(j as u32));
                    continue;
                }
                match c {
                    Some(Right(s)) => children.push(Opaque::Text(s.into_boxed_str())),
                    Some(Left(n)) => {
                        #[allow(clippy::cast_possible_truncation)]
                        path.push(i as u32);
                        let (id, attrs, ch) = rec(path, paths, &n)?;
                        children.push(Opaque::Node {
                            tag: id,
                            attributes: attrs,
                            children: ch,
                        });
                        path.pop();
                    }
                    None => (),
                }
            }
            Ok((
                tag,
                attributes.into_boxed_slice(),
                children.into_boxed_slice(),
            ))
        }
        let mut paths = Vec::with_capacity(termpairs.len());
        let terms = termpairs
            .into_iter()
            .map(|(t, p)| {
                paths.push(p);
                t
            })
            .collect();
        let mut path = crate::NodePath::new();
        let (tag, attributes, children) = rec(&mut path, &paths, self)?;
        Ok(Term::Opaque {
            tag,
            attributes,
            terms,
            children,
        })
    }
}
