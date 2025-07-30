use std::{borrow::Cow, hint::unreachable_unchecked};

use crate::{FtmlKey, extraction::FtmlExtractionError};
use either::Either::{Left, Right};
use ftml_ontology::{
    narrative::{
        DocumentRange,
        elements::notations::{NodeOrText, NotationComponent, NotationNode},
    },
    terms::{Term, opaque::Opaque},
};
use ftml_uris::Id;

pub trait FtmlNode: Clone + std::fmt::Debug {
    //type Ancestors<'a>: Iterator<Item = Self> where Self: 'a;
    //fn ancestors(&self) -> Self::Ancestors<'_>;
    //fn with_elements<R>(&mut self, f: impl FnMut(Option<&mut FTMLElements>) -> R) -> R;
    fn string(&self) -> Cow<'_, str>;
    fn inner_string(&self) -> Cow<'_, str>;
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
    fn tag_id(&self) -> Result<Id, String> {
        self.tag_name()?
            .parse()
            .map_err(|e| format!("invalid attribute: {e}"))
    }

    /// ### Errors
    fn collect_attributes(&self) -> Result<Vec<(Id, Box<str>)>, String> {
        self.iter_attributes()
            .map(|e| {
                e.map_or_else::<Result<_, String>, _, _>(Err, |(k, v)| {
                    let k = k
                        .parse::<Id>()
                        .map_err(|e| format!("invalid attribute: {e}"))?;
                    let v = v.into_boxed_str();
                    Ok((k, v))
                })
            })
            .collect::<Result<Vec<_>, String>>()
    }

    /// ### Errors
    fn as_notation(
        &self,
        comppairs: Vec<(NotationComponent, crate::NodePath)>,
    ) -> Result<NotationComponent, FtmlExtractionError> {
        fn rec<N: FtmlNode>(
            path: &mut crate::NodePath,
            paths: &[crate::NodePath],
            comps: &mut Vec<Option<NotationComponent>>,
            node: &N,
        ) -> Result<NotationComponent, FtmlExtractionError> {
            let tag = node
                .tag_id()
                .map_err(FtmlExtractionError::InvalidInformal)?;
            let attributes = node
                .collect_attributes()
                .map_err(FtmlExtractionError::InvalidInformal)?;
            let mut children = Vec::new();
            for (i, c) in node.children().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                if let Some((j, _)) = paths.iter().enumerate().find(|(_, p)| {
                    p.len() == path.len() + 1
                        && p.starts_with(path)
                        && p.last().is_some_and(|j| *j == i as u32)
                }) {
                    children.push(
                        comps
                            .get_mut(j)
                            .ok_or(FtmlExtractionError::InvalidValue(FtmlKey::NotationComp))?
                            .take()
                            .ok_or(FtmlExtractionError::InvalidValue(FtmlKey::NotationComp))?,
                    );
                    continue;
                }
                match c {
                    Some(Right(s)) if !s.as_bytes().iter().all(u8::is_ascii_whitespace) => {
                        children.push(NotationComponent::Text(s.into_boxed_str()));
                    }
                    Some(Left(n)) => {
                        #[allow(clippy::cast_possible_truncation)]
                        path.push(i as u32);
                        children.push(rec(path, paths, comps, &n)?);
                        path.pop();
                    }
                    None | Some(Right(_)) => (),
                }
            }
            if attributes.is_empty() && children.len() == 1 && TRANSPARENT.contains(&tag.as_ref()) {
                // SAFETY len == 1
                return Ok(unsafe { children.pop().unwrap_unchecked() });
            }
            Ok(NotationComponent::Node {
                tag,
                attributes: attributes.into_boxed_slice(),
                children: children.into_boxed_slice(),
            })
        }

        // ------------------------------
        let mut paths = Vec::with_capacity(comppairs.len());
        let mut comps = comppairs
            .into_iter()
            .map(|(t, p)| {
                paths.push(p);
                Some(t)
            })
            .collect();
        let mut path = crate::NodePath::new();
        rec(&mut path, &paths, &mut comps, self)
        /*
        let (tag, attributes, mut children) = rec(&mut path, &paths, &mut comps, self)?;
        if attributes.is_empty() && children.len() == 1 && TRANSPARENT.contains(&tag.as_ref()) {
            // SAFETY len == 1
            return Ok(unsafe { children.pop().unwrap_unchecked() });
        }
        Ok(NotationComponent::Node {
            tag,
            attributes,
            children: children.into_boxed_slice(),
        })
         */
    }

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
                .tag_id()
                .map_err(FtmlExtractionError::InvalidInformal)?;
            let attributes = node
                .collect_attributes()
                .map_err(FtmlExtractionError::InvalidInformal)?;
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

    /// ### Errors
    fn as_component(&self) -> Result<NotationNode, FtmlExtractionError> {
        let tag = self
            .tag_id()
            .map_err(FtmlExtractionError::InvalidNotationComponent)?;
        let attributes = self
            .collect_attributes()
            .map_err(FtmlExtractionError::InvalidNotationComponent)?;
        let mut children = Vec::new();
        for c in self.children() {
            match c {
                Some(Left(n)) => children.push(NodeOrText::Node(n.as_component()?)),
                Some(Right(t)) if !t.as_bytes().iter().all(u8::is_ascii_whitespace) => {
                    children.push(NodeOrText::Text(t.into_boxed_str()));
                }
                None | Some(Right(_)) => (),
            }
        }
        if attributes.is_empty()
            && children.len() == 1
            && TRANSPARENT.contains(&tag.as_ref())
            && matches!(children.first(), Some(NodeOrText::Node(_)))
        {
            // SAFETY len == 1 && matches!(...)
            unsafe {
                let Some(NodeOrText::Node(n)) = children.pop() else {
                    unreachable_unchecked()
                };
                return Ok(n);
            }
        }
        Ok(NotationNode {
            tag,
            attributes: attributes.into_boxed_slice(),
            children: children.into_boxed_slice(),
        })
    }
}

const TRANSPARENT: [&str; 1] = ["mrow"];
