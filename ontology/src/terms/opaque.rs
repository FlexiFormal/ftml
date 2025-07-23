#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum Opaque {
    Expr(u8),
    Node {
        tag: ftml_uris::Id,
        attributes: Box<[(ftml_uris::Id, Box<str>)]>,
        children: Box<[Opaque]>,
    },
    Text(Box<str>),
}

impl Opaque {
    #[must_use]
    pub fn iter_opt(&self) -> Option<impl Iterator<Item = &Self>> {
        match self {
            Self::Expr(_) | Self::Text(_) => None,
            Self::Node { children, .. } => Some(OpaqueIter {
                curr: children.iter(),
                stack: smallvec::SmallVec::new(),
            }),
        }
    }
    #[must_use]
    pub fn iter_mut_opt(&mut self) -> Option<impl Iterator<Item = &mut Self>> {
        match self {
            Self::Expr(_) | Self::Text(_) => None,
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
