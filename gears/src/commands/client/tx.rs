use std::path::PathBuf;

use core_types::tx::mode_info::SignMode;
use prost::Message;
use tendermint::rpc::client::{Client, HttpClient};
use tendermint::rpc::response::tx::broadcast::Response;
use tendermint::types::chain_id::ChainId;

use crate::application::handlers::client::{TxExecutionResult, TxHandler};
use crate::commands::client::query::execute_query;
use crate::crypto::any_key::AnyKey;
use crate::crypto::keys::GearsPublicKey;
use crate::crypto::ledger::LedgerProxyKey;
use crate::runtime::runtime;
use crate::types::auth::gas::Gas;
use crate::types::base::coins::UnsignedCoins;
use crate::types::tx::raw::TxRaw;

use super::keys::KeyringBackend;

#[derive(Debug, Clone)]
pub enum AccountProvider {
    Offline { sequence: u64, account_number: u64 },
    Online,
}

#[derive(Debug, Clone, former::Former)]
pub struct TxCommand<C> {
    pub ctx: ClientTxContext,
    pub inner: C,
}

#[derive(Debug, Clone)]
pub struct ClientTxContext {
    pub node: url::Url,
    pub home: PathBuf,
    pub keyring: Keyring,
    pub memo: Option<String>,
    pub account: AccountProvider,
    pub gas_limit: Gas,
    pub chain_id: ChainId,
    pub fees: Option<UnsignedCoins>,
    pub timeout_height: Option<u32>,
}

impl ClientTxContext {
    pub fn query<Response: TryFrom<Raw>, Raw: Message + Default + std::convert::From<Response>>(
        &self,
        path: String,
        query_bytes: Vec<u8>,
    ) -> anyhow::Result<Response>
    where
        <Response as TryFrom<Raw>>::Error: std::fmt::Display,
    {
        execute_query(path, query_bytes, self.node.as_str(), None)
    }

    pub fn new_online(
        home: PathBuf,
        gas_limit: Gas,
        node: url::Url,
        chain_id: ChainId,
        from_key: &str,
    ) -> Self {
        Self {
            account: crate::commands::client::tx::AccountProvider::Online,
            gas_limit,
            home,
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: from_key.to_owned(),
            }),
            node,
            chain_id,
            fees: None,
            memo: None,
            timeout_height: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Keyring {
    Ledger,
    Local(LocalInfo),
}

#[derive(Debug, Clone)]
pub struct LocalInfo {
    pub keyring_backend: KeyringBackend,
    pub from_key: String,
}

#[derive(Debug, Clone)]
pub enum RuntxResult {
    Broadcast(Vec<Response>),
    File(PathBuf),
    None,
}

impl RuntxResult {
    pub fn broadcast(self) -> Option<Vec<Response>> {
        match self {
            Self::Broadcast(var) => Some(var),
            Self::File(_) => None,
            Self::None => None,
        }
    }

    pub fn file(self) -> Option<PathBuf> {
        match self {
            Self::Broadcast(_) => None,
            Self::File(var) => Some(var),
            Self::None => None,
        }
    }
}

impl From<TxExecutionResult> for RuntxResult {
    fn from(value: TxExecutionResult) -> Self {
        match value {
            TxExecutionResult::Broadcast(var) => Self::Broadcast(vec![var]),
            TxExecutionResult::File(var) => Self::File(var),
            TxExecutionResult::None => Self::None,
        }
    }
}

fn handle_key(client_tx_context: &ClientTxContext) -> anyhow::Result<AnyKey> {
    match client_tx_context.keyring {
        Keyring::Ledger => Ok(AnyKey::Ledger(LedgerProxyKey::new()?)),
        Keyring::Local(ref local) => {
            let keyring_home = client_tx_context
                .home
                .join(local.keyring_backend.get_sub_dir());
            let key = keyring::key_by_name(
                &local.from_key,
                local.keyring_backend.to_keyring_backend(&keyring_home),
            )?;

            Ok(AnyKey::Local(key))
        }
    }
}

pub fn run_tx<C, H: TxHandler<TxCommands = C>>(
    TxCommand { mut ctx, inner }: TxCommand<C>,
    handler: &H,
) -> anyhow::Result<RuntxResult> {
    let key = handle_key(&mut ctx)?;

    let messages = handler.prepare_tx(&mut ctx, inner, key.get_gears_public_key())?;

    if messages.chunk_size() > 0
    // TODO: uncomment and update logic when command will be extended by broadcast_mode
    /* && command.broadcast_mode == BroadcastMode::Block */
    {
        let chunk_size = messages.chunk_size();
        let msgs = messages.into_msgs();

        let mut res = vec![];
        for slice in msgs.chunks(chunk_size) {
            res.push(
                handler
                    .handle_tx(
                        handler.sign_msg(
                            slice
                                .to_vec()
                                .try_into()
                                .expect("chunking of the messages excludes empty vectors"),
                            &key,
                            SignMode::Direct,
                            &mut ctx,
                        )?,
                        &mut ctx,
                    )?
                    .broadcast()
                    .ok_or(anyhow::anyhow!("tx is not broadcasted"))?,
            );
        }
        Ok(RuntxResult::Broadcast(res))
    } else {
        // TODO: can be reduced by changing variable `step`. Do we need it?
        handler
            .handle_tx(
                handler.sign_msg(messages, &key, SignMode::Direct, &mut ctx)?,
                &mut ctx,
            )
            .map(Into::into)
    }
}

pub fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> anyhow::Result<Response> {
    let res = runtime().block_on(
        client.broadcast_tx_commit(core_types::tx::raw::TxRaw::from(raw_tx).encode_to_vec()),
    )?;

    Ok(res)
}
