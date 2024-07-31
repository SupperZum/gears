use crate::{
    commands::client::{
        keys::keys, query::run_query, tx::run_tx, ClientCommands, ExtendedQueryCommand,
    },
    x::query::tx_query::{TxQueryHandler, TxsQueryHandler},
};

use super::handlers::{
    client::{QueryHandler, TxHandler},
    AuxHandler,
};

/// A Gears client application.
pub trait Client: TxHandler + QueryHandler + AuxHandler {}

pub struct ClientApplication<Core: Client> {
    core: Core,
}

impl<Core: Client> ClientApplication<Core> {
    pub fn new(core: Core) -> Self {
        Self { core }
    }

    /// Runs the command passed
    pub fn execute(
        &self,
        command: ClientCommands<Core::AuxCommands, Core::TxCommands, Core::QueryCommands>,
    ) -> anyhow::Result<()> {
        match command {
            ClientCommands::Aux(cmd) => {
                let cmd = self.core.prepare_aux(cmd)?;
                self.core.handle_aux(cmd)?;
            }
            ClientCommands::Tx(cmd) => {
                let tx = run_tx(cmd, &self.core)?;

                println!("{}", serde_json::to_string_pretty(&tx)?);
            }
            ClientCommands::Query(cmd) => {
                let query = match cmd {
                    ExtendedQueryCommand::QueryCmd(cmd) => {
                        serde_json::to_string_pretty(&run_query(cmd, &self.core)?)?
                    }
                    ExtendedQueryCommand::Tx(cmd) => serde_json::to_string_pretty(&run_query(
                        cmd,
                        &TxQueryHandler::<Core::Message>::new(),
                    )?)?,
                    ExtendedQueryCommand::Txs(cmd) => serde_json::to_string_pretty(&run_query(
                        cmd,
                        &TxsQueryHandler::<Core::Message>::new(),
                    )?)?,
                };

                println!("{}", query);
            }
            ClientCommands::Keys(cmd) => keys(cmd)?,
        };

        Ok(())
    }
}
