//! Implements support for the ibc module.
use crate::{
    codec::Encoded,
    metadata::MetadataError,
    srml::{balances::Balances, system::System, ModuleCalls},
    Valid, XtBuilder,
};

use substrate_primitives::Pair;

type Identifier = u32;

///
pub trait Ibc: System + Balances {}

///
pub trait IbcXt {
    ///
    type Ibc: Ibc;
    ///
    type Pair: Pair;

    ///
    fn ibc<F>(&self, f: F) -> XtBuilder<Self::Ibc, Self::Pair, Valid>
    where
        F: FnOnce(ModuleCalls<Self::Ibc, Self::Pair>) -> Result<Encoded, MetadataError>;
}

impl<T: Ibc + 'static, P, V> IbcXt for XtBuilder<T, P, V>
where
    P: Pair,
{
    type Ibc = T;
    type Pair = P;

    fn ibc<F>(&self, f: F) -> XtBuilder<T, P, Valid>
    where
        F: FnOnce(ModuleCalls<Self::Ibc, Self::Pair>) -> Result<Encoded, MetadataError>,
    {
        self.set_call("MultiDAO", f)
    }
}

impl<T: Ibc + 'static, P> ModuleCalls<T, P>
where
    P: Pair,
{
    ///
    pub fn update_client(
        self,
        id: Identifier,
        header: Vec<u8>,
    ) -> Result<Encoded, MetadataError> {
        self.module.call("update_client", (id, header))
    }

    ///
    pub fn recv_packet(
        self,
        packet: Vec<u8>,
        proof: Vec<Vec<u8>>,
        proof_height: T::BlockNumber,
    ) -> Result<Encoded, MetadataError> {
        self.module
            .call("recv_packet", (packet, proof, proof_height))
    }
}
