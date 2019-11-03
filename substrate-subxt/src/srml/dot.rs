//! Implements support for the ibc module.
use crate::{
    codec::Encoded,
    metadata::MetadataError,
    srml::{balances::Balances, system::System, ModuleCalls},
    Valid, XtBuilder,
};

use substrate_primitives::Pair;

///
pub trait Dot: System + Balances {}

///
pub trait DotXt {
    ///
    type Dot: Dot;
    ///
    type Pair: Pair;

    ///
    fn ibc<F>(&self, f: F) -> XtBuilder<Self::Dot, Self::Pair, Valid>
    where
        F: FnOnce(ModuleCalls<Self::Dot, Self::Pair>) -> Result<Encoded, MetadataError>;
}

impl<T: Dot + 'static, P, V> DotXt for XtBuilder<T, P, V>
where
    P: Pair,
{
    type Dot = T;
    type Pair = P;

    fn ibc<F>(&self, f: F) -> XtBuilder<T, P, Valid>
    where
        F: FnOnce(ModuleCalls<Self::Dot, Self::Pair>) -> Result<Encoded, MetadataError>,
    {
        self.set_call("TemplateModule", f)
    }
}

impl<T: Dot + 'static, P> ModuleCalls<T, P>
where
    P: Pair,
{
    ///
    pub fn transfer_dot(self, amount: u128) -> Result<Encoded, MetadataError> {
        println!("in transfer_dot");
        self.module.call("transfer_dot", amount)
    }
}
