pub trait IsDeclaration: crate::__private::Sealed {
    fn from_declaration(decl: &AnyDeclaration) -> Option<&Self>;
}

#[derive(Debug)]
pub enum AnyDeclaration {
    NestedModule(NestedModule),
    Import(Import),
    Symbol(Symbol),
    MathStructure(MathStructure),
    Morphism(Morphism),
    Extension(Extension),
}
crate::serde_impl! {
    enum AnyDeclaration{
        {0 = NestedModule(nm)}
        {1 = Import(ml)}
        {2 = Symbol(s)}
        {3 = MathStructure(s)}
        {4 = Morphism(m)}
        {5 = Extension(e)}
    }
}

pub enum AnyOpenDeclaration {}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NestedModule;
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Import;
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Symbol;
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MathStructure;
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Morphism;
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Extension;
