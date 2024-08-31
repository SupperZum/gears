use std::{marker::PhantomData, path::PathBuf, str::FromStr};

use clap::{ArgAction, Args, Subcommand, ValueEnum, ValueHint};
use strum::Display;
use tendermint::types::chain_id::ChainId;

use crate::{
    application::ApplicationInfo,
    commands::client::{
        keys::KeyringBackend,
        tx::{AccountProvider, Keyring as TxKeyring, LocalInfo, TxCommand},
    },
    config::DEFAULT_TENDERMINT_RPC_ADDRESS,
    types::base::coins::UnsignedCoins,
};

/// Transaction subcommands
#[derive(Debug, Clone, ::clap::Args)]
pub struct CliTxCommand<T: ApplicationInfo, C: Args> {
    #[arg(long, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = T::home_dir(), help = "directory for config and data")]
    home: PathBuf,
    /// <host>:<port> to Tendermint RPC interface for this chain
    #[arg(long, global = true, action = ArgAction::Set, value_hint = ValueHint::Url, default_value_t = DEFAULT_TENDERMINT_RPC_ADDRESS.parse().expect( "const should be valid"))]
    pub node: url::Url,
    /// the network chain-id
    #[arg(long =  "chain-id", global = true, action = ArgAction::Set, default_value_t = ChainId::from_str( "test-chain" ).expect("unreachable: default should be valid"))]
    pub chain_id: ChainId,
    /// TODO
    #[arg(long, global = true, action = ArgAction::Set)]
    pub fees: Option<UnsignedCoins>,

    #[arg(long, short, default_value_t = Keyring::Local)]
    pub keyring: Keyring,

    #[command(flatten)]
    #[group(id = "local", conflicts_with = Keyring::Ledger, global = true)]
    pub local: Option<Local>,

    #[command(flatten)]
    #[group(id = "Broadcast mode", global = true)]
    pub mode: Mode,

    #[command(flatten)]
    pub command: C,

    #[arg(skip)]
    _marker: PhantomData<T>,
}

#[derive(Debug, Clone, ::clap::Args)]
pub struct Mode {
    /// makes sure that the client will not reach out to full node.
    /// As a result, the account and sequence number queries will not be performed and
    /// it is required to set such parameters manually. Note, invalid values will cause
    /// the transaction to fail.
    #[arg(long, default_value_t = false)]
    pub offline: bool,
    #[arg(long, required = false)]
    pub sequence: Option<u64>,
    #[arg(long, required = false)]
    pub account_number: Option<u64>,
}

#[derive(ValueEnum, Debug, Clone, Display)]
pub enum Keyring {
    /// Use a Ledger device to sign the transaction
    #[strum(to_string = "ledger")]
    Ledger,
    /// Use a local keyring to source the signing key
    #[strum(to_string = "local")]
    Local,
}

#[derive(Debug, Clone, ::clap::Args)]
pub struct Local {
    /// from key
    #[arg(long, short = 'f', global = true, required = false)]
    #[arg(help_heading = "Local signing options")]
    from_key: String,

    /// select keyring's backend
    #[arg(long = "keyring-backend", short = 'b',  global = true, action = ArgAction::Set, default_value_t = KeyringBackend::File )]
    #[arg(help_heading = "Local signing options")]
    keyring_backend: KeyringBackend,
}

#[derive(Debug, Clone, ::clap::Args)]
pub struct Ledger<C: Subcommand> {
    #[command(subcommand)]
    command: C,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Missing options for: {0}")]
pub struct MissingCliOptions(pub String);

impl<T, C, AC> TryFrom<CliTxCommand<T, C>> for TxCommand<AC>
where
    T: ApplicationInfo,
    C: Args,
    AC: TryFrom<C, Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    fn try_from(value: CliTxCommand<T, C>) -> Result<Self, Self::Error> {
        let CliTxCommand {
            home,
            node,
            chain_id,
            fees: fee,
            _marker,
            keyring,
            local,
            mode,
            command,
        } = value;

        let keyring = match keyring {
            Keyring::Ledger => TxKeyring::Ledger,
            Keyring::Local => {
                let Local {
                    from_key,
                    keyring_backend,
                } = local.ok_or(MissingCliOptions(
                    "local signing options: from-key".to_owned(),
                ))?;

                TxKeyring::Local(LocalInfo {
                    keyring_backend,
                    from_key,
                })
            }
        };

        let account = match mode {
            Mode {
                offline: true,
                sequence,
                account_number,
            } => AccountProvider::Offline {
                sequence: sequence.unwrap_or_default(),
                account_number: account_number.unwrap_or_default(),
            },
            _ => AccountProvider::Online,
        };

        Ok(Self {
            home,
            node,
            chain_id,
            fees: fee,
            keyring,
            inner: command.try_into()?,
            account,
        })
    }
}
