#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum Opaque {
    Term(u32),
    Node {
        tag: ftml_uris::Id,
        #[cfg_attr(feature = "serde", serde(default))]
        attributes: Box<[(ftml_uris::Id, Box<str>)]>,
        #[cfg_attr(feature = "serde", serde(default))]
        children: Box<[Opaque]>,
    },
    Text(Box<str>),
}

impl Opaque {
    #[must_use]
    pub fn iter_opt(&self) -> Option<impl Iterator<Item = &Self>> {
        match self {
            Self::Term(_) | Self::Text(_) => None,
            Self::Node { children, .. } => Some(OpaqueIter {
                curr: children.iter(),
                stack: smallvec::SmallVec::new(),
            }),
        }
    }
    #[must_use]
    pub fn iter_mut_opt(&mut self) -> Option<impl Iterator<Item = &mut Self>> {
        match self {
            Self::Term(_) | Self::Text(_) => None,
            Self::Node { children, .. } => Some(OpaqueIterMut {
                curr: children.iter_mut(),
                stack: smallvec::SmallVec::new(),
            }),
        }
    }
}

struct OpaqueIter<'a> {
    curr: std::slice::Iter<'a, Opaque>,
    stack: smallvec::SmallVec<std::slice::Iter<'a, Opaque>, 4>,
}
impl<'a> Iterator for OpaqueIter<'a> {
    type Item = &'a Opaque;
    fn next(&mut self) -> Option<Self::Item> {
        let r = self.curr.next().or_else(|| {
            self.curr = self.stack.pop()?;
            self.curr.next()
        });
        if let Some(Opaque::Node { children, .. }) = r {
            self.stack
                .push(std::mem::replace(&mut self.curr, children.iter()));
        }
        r
    }
}
struct OpaqueIterMut<'a> {
    curr: std::slice::IterMut<'a, Opaque>,
    stack: smallvec::SmallVec<std::slice::IterMut<'a, Opaque>, 4>,
}
impl<'a> Iterator for OpaqueIterMut<'a> {
    type Item = &'a mut Opaque;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let r = self.curr.next().or_else(|| {
                self.curr = self.stack.pop()?;
                self.curr.next()
            });
            if let Some(Opaque::Node { children, .. }) = r {
                self.stack
                    .push(std::mem::replace(&mut self.curr, children.iter_mut()));
            } else {
                return r;
            }
        }
    }
}

impl std::fmt::Display for Opaque {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(t) => write!(f, "\"{t}\""),
            Self::Term(t) => write!(f, "<term {t}/>"),
            Self::Node {
                tag,
                attributes,
                children,
            } => {
                write!(f, "<{tag}")?;
                for (k, v) in attributes {
                    write!(f, " {k}=\"{v}\"")?;
                }
                f.write_str(">\n")?;
                for t in children {
                    writeln!(f, "{t}")?;
                }
                write!(f, "</{tag}>")
            }
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for Opaque {
    fn deep_size_of_children(&self, _: &mut deepsize::Context) -> usize {
        match self {
            Self::Node {
                attributes,
                children,
                ..
            } => {
                attributes
                    .iter()
                    .map(|p| std::mem::size_of_val(p) + p.1.len())
                    .sum::<usize>()
                    + children.iter().map(Self::deep_size_of).sum::<usize>()
            }
            Self::Text(t) => t.len(),
            Self::Term(_) => 0,
        }
    }
}
