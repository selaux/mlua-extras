use std::ops::Deref;

#[derive(strum::EnumIs)]
pub enum Kind {
    Regular,
    Typed,
}

pub struct Builder {
    kind: Kind
}

impl Deref for Builder {
    type Target = Kind;
    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl Builder {
    #[inline(always)]
    pub fn regular() -> Self { Self { kind: Kind::Regular } }

    #[inline(always)]
    pub fn typed() -> Self { Self { kind: Kind::Typed } }
}