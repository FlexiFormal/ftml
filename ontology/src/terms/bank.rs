use ftml_uris::UriName;

use crate::terms::{
    Argument, BoundArgument, Term, VarOrSym,
    opaque::OpaqueNode,
    term::{
        Application, ApplicationTerm, Binding, BindingTerm, Opaque, OpaqueTerm, RecordField,
        RecordFieldTerm,
    },
};
use std::hash::Hash;
use std::hash::Hasher;

struct TermBank {
    applications: dashmap::DashSet<ApplicationTerm, rustc_hash::FxBuildHasher>,
    bindings: dashmap::DashSet<BindingTerm, rustc_hash::FxBuildHasher>,
    records: dashmap::DashSet<RecordFieldTerm, rustc_hash::FxBuildHasher>,
    opaques: dashmap::DashSet<OpaqueTerm, rustc_hash::FxBuildHasher>,
}
static TERM_BANK: std::sync::LazyLock<TermBank> = std::sync::LazyLock::new(|| TermBank {
    applications: dashmap::DashSet::default(),
    bindings: dashmap::DashSet::default(),
    records: dashmap::DashSet::default(),
    opaques: dashmap::DashSet::default(),
});

macro_rules! imp {
    ($outer:ident($inner:ident{$($name:ident:$tp:ty),*$(,)?}) = $set:ident @ $num:literal ) => {
        impl std::ops::Deref for $outer {
            type Target = $inner;
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::borrow::Borrow<$inner> for $outer {
            #[inline]
            fn borrow(&self) -> &$inner {
                &self.0
            }
        }

        impl $outer {
            #[must_use]
            pub fn new( $($name:$tp),* ) -> Self {
                Self::from_inner($inner {
                    $($name,)*
                    hash: 0,
                })
            }
            fn from_inner(mut inner: $inner) -> Self {
                inner.set_hash();
                if let Some(v) = TERM_BANK.$set.get(&inner) {
                    return v.clone();
                }
                let t = Self(triomphe::Arc::new(inner));
                TERM_BANK.$set.insert(t.clone());
                if TERM_BANK.$set.len() > $num {
                    TERM_BANK.$set.retain(|e| !e.0.is_unique());
                }
                t
            }
        }

        impl $inner {
            fn set_hash(&mut self) {
                let mut hash = rustc_hash::FxHasher::default();
                $(
                    self.$name.hash(&mut hash);
                )*
                self.hash = hash.finish();
            }
        }

        impl std::hash::Hash for $inner {
            #[inline]
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.hash.hash(state);
            }
        }
        impl std::hash::Hash for $outer {
            #[inline]
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.hash.hash(state);
            }
        }
        impl PartialEq for $outer {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                std::ptr::eq(&*self.0, &*other.0)
            }
        }
        impl Eq for $outer {}

        #[cfg(feature = "serde-lite")]
        impl serde_lite::Serialize for $outer {
            #[inline]
            fn serialize(&self) -> Result<serde_lite::Intermediate, serde_lite::Error> {
                self.0.serialize()
            }
        }

        #[cfg(feature = "serde-lite")]
        impl serde_lite::Deserialize for $outer {
            #[inline]
            fn deserialize(val:&serde_lite::Intermediate) -> Result<Self,serde_lite::Error> {
                $inner::deserialize(val).map(Self::from_inner)
            }

        }

        #[cfg(feature = "serde")]
        impl serde::Serialize for $outer {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $outer {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $inner::deserialize(deserializer).map(Self::from_inner)
            }
        }

        #[cfg(feature = "serde")]
        impl<Context> bincode::Decode<Context> for $outer {
            fn decode<D: bincode::de::Decoder<Context = Context>>(
                decoder: &mut D,
            ) -> Result<Self, bincode::error::DecodeError> {
                $inner::decode(decoder).map(Self::from_inner)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de, Context> bincode::BorrowDecode<'de, Context> for $outer {
            fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
                decoder: &mut D,
            ) -> Result<Self, bincode::error::DecodeError> {
                $inner::borrow_decode(decoder).map(Self::from_inner)
            }
        }

        #[cfg(feature = "serde")]
        impl bincode::Encode for $outer {
            fn encode<E: bincode::enc::Encoder>(
                &self,
                encoder: &mut E,
            ) -> Result<(), bincode::error::EncodeError> {
                self.0.encode(encoder)
            }
        }
    };
}

imp!(ApplicationTerm(Application{
    head: Term,
    arguments: Box<[Argument]>,
    presentation: Option<VarOrSym>,
}) = applications @ 2048);

imp!(BindingTerm(Binding{
    head: Term,
    arguments: Box<[BoundArgument]>,
    //body: Term,
    presentation: Option<VarOrSym>,
}) = bindings @ 512);

imp!(RecordFieldTerm(RecordField{
    record: Term,
    key: UriName,
    record_type: Option<Term>,
    presentation: Option<VarOrSym>,
}) = records @ 256);

imp!(OpaqueTerm(Opaque {
    node:OpaqueNode,
    terms: Box<[Term]>,
}) = opaques @ 2048);

#[cfg(feature = "deepsize")]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[derive(Debug, Clone, Copy)]
pub struct TermCacheSize {
    pub num_applications: usize,
    pub applications_bytes: usize,
    pub num_bindings: usize,
    pub bindings_bytes: usize,
    pub num_records: usize,
    pub records_bytes: usize,
    pub num_opaques: usize,
    pub opaques_bytes: usize,
}

#[cfg(feature = "deepsize")]
impl TermCacheSize {
    #[must_use]
    pub const fn total_bytes(&self) -> usize {
        self.applications_bytes + self.bindings_bytes + self.records_bytes + self.opaques_bytes
    }
}

#[cfg(feature = "deepsize")]
impl std::fmt::Display for TermCacheSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total = self.total_bytes();
        let Self {
            num_applications,
            applications_bytes,
            num_bindings,
            bindings_bytes,
            num_records,
            records_bytes,
            num_opaques,
            opaques_bytes,
        } = self;
        write!(
            f,
            "\n\
             applications: {num_applications} ({})\n\
             bindings:     {num_bindings} ({})\n\
             records:      {num_records} ({})\n\
             opaques:      {num_opaques} ({})\n\
             ----------------------------------\n\
             total: {}
            ",
            bytesize::ByteSize::b(*applications_bytes as u64)
                .display()
                .iec_short(),
            bytesize::ByteSize::b(*bindings_bytes as u64)
                .display()
                .iec_short(),
            bytesize::ByteSize::b(*records_bytes as u64)
                .display()
                .iec_short(),
            bytesize::ByteSize::b(*opaques_bytes as u64)
                .display()
                .iec_short(),
            bytesize::ByteSize::b(total as u64).display().iec_short(),
        )
    }
}

pub fn clear_term_cache() {
    TERM_BANK.applications.retain(|e| !e.0.is_unique());
    TERM_BANK.bindings.retain(|e| !e.0.is_unique());
    TERM_BANK.records.retain(|e| !e.0.is_unique());
    TERM_BANK.opaques.retain(|e| !e.0.is_unique());
}

#[cfg(feature = "deepsize")]
pub fn get_cache_size() -> TermCacheSize {
    use deepsize::DeepSizeOf;
    let mut num_applications = 0;
    let mut applications_bytes = 0;
    let mut num_bindings = 0;
    let mut bindings_bytes = 0;
    let mut num_records = 0;
    let mut records_bytes = 0;
    let mut num_opaques = 0;
    let mut opaques_bytes = 0;
    for a in TERM_BANK.applications.iter() {
        num_applications += 1;
        applications_bytes += std::mem::size_of::<ApplicationTerm>()
            + std::mem::size_of::<Option<VarOrSym>>() // presentation
            + a.head.deep_size_of()
            + a.arguments
                .iter()
                .map(Argument::deep_size_of)
                .sum::<usize>()
            + 8  // hash
            + 8; // dashmap overhead?
    }
    for a in TERM_BANK.bindings.iter() {
        num_bindings += 1;
        bindings_bytes += std::mem::size_of::<BindingTerm>()
            + std::mem::size_of::<Option<VarOrSym>>() // presentation
            + a.head.deep_size_of()
            //+ a.body.deep_size_of()
            + a.arguments
                .iter()
                .map(BoundArgument::deep_size_of)
                .sum::<usize>()
            + 8  // hash
            + 8; // dashmap overhead?
    }
    for a in TERM_BANK.records.iter() {
        num_records += 1;
        records_bytes += std::mem::size_of::<RecordFieldTerm>()
            + std::mem::size_of::<Option<VarOrSym>>() // presentation
            + a.record.deep_size_of()
            + a.record_type.deep_size_of()
            + 8  // hash
            + 8; // dashmap overhead?
    }
    for a in TERM_BANK.opaques.iter() {
        num_opaques += 1;
        opaques_bytes += std::mem::size_of::<OpaqueTerm>()
            + a.node.deep_size_of()
            + a.terms
                .iter()
                .map(Term::deep_size_of)
                .sum::<usize>()
            + 8  // hash
            + 8; // dashmap overhead?
    }

    TermCacheSize {
        num_applications,
        applications_bytes,
        num_bindings,
        bindings_bytes,
        num_records,
        records_bytes,
        num_opaques,
        opaques_bytes,
    }
}
