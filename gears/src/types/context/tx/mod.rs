mod unconsumed_impl;

use store_crate::{
    database::{Database, PrefixDB},
    types::{kv::KVStore, multi::MultiStore},
    QueryableMultiKVStore, StoreKey, TransactionalMultiKVStore,
};
use tendermint::types::{chain_id::ChainId, proto::event::Event};

use crate::types::header::Header;

use super::{
    gas::{BlockDescriptor, CtxGasMeter},
    QueryableContext, TransactionalContext,
};

#[derive(Debug, former::Former)]
pub struct TxContext2<'a, DB, SK, GM, ST> {
    pub events: Vec<Event>,

    multi_store: &'a mut MultiStore<DB, SK>,
    height: u64,
    header: Header,
    block_gas_meter: CtxGasMeter<GM, ST, BlockDescriptor>,
}

impl<'a, DB, SK, GM, ST> TxContext2<'a, DB, SK, GM, ST> {
    pub fn new(
        multi_store: &'a mut MultiStore<DB, SK>,
        height: u64,
        header: Header,
        block_gas_meter: GM,
    ) -> Self {
        Self {
            events: Vec::new(),
            multi_store,
            height,
            header,
            block_gas_meter: CtxGasMeter::new(block_gas_meter),
        }
    }
}

impl<DB: Database, SK: StoreKey, GM, ST> QueryableContext<PrefixDB<DB>, SK>
    for TxContext2<'_, DB, SK, GM, ST>
{
    type KVStore = KVStore<PrefixDB<DB>>;

    fn kv_store(&self, store_key: &SK) -> &Self::KVStore {
        self.multi_store.kv_store(store_key)
    }

    fn height(&self) -> u64 {
        self.height
    }

    fn chain_id(&self) -> &ChainId {
        &self.header.chain_id
    }
}

impl<DB: Database, SK: StoreKey, GM, ST> TransactionalContext<PrefixDB<DB>, SK>
    for TxContext2<'_, DB, SK, GM, ST>
{
    type KVStoreMut = KVStore<PrefixDB<DB>>;

    fn kv_store_mut(&mut self, store_key: &SK) -> &mut Self::KVStoreMut {
        self.multi_store.kv_store_mut(store_key)
    }

    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }
}
