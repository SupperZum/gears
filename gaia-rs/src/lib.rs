use crate::abci_handler::GaiaABCIHandler;
use crate::query::GaiaQuery;
use crate::query::GaiaQueryResponse;
use crate::store_keys::GaiaParamsStoreKey;
use anyhow::Result;
use auth::cli::query::AuthQueryHandler;
use auth::query::QueryAccountRequest;
use auth::query::QueryAccountResponse;
use auth::AuthNodeQueryRequest;
use auth::AuthNodeQueryResponse;
use axum::Router;
use bank::cli::query::BankQueryHandler;
use bank::types::query::QueryDenomMetadataRequest;
use bank::types::query::QueryDenomMetadataResponse;
use bank::BankNodeQueryRequest;
use bank::BankNodeQueryResponse;
use clap::Subcommand;
use client::tx_command_handler;
use client::GaiaQueryCommands;
use client::WrappedGaiaQueryCommands;
use distribution::DistributionNodeQueryRequest;
use distribution::DistributionNodeQueryResponse;
use gears::application::client::Client;
use gears::application::handlers::client::NodeFetcher;
use gears::application::handlers::client::{QueryHandler, TxHandler};
use gears::application::handlers::AuxHandler;
use gears::application::node::Node;
use gears::application::ApplicationInfo;
use gears::baseapp::NodeQueryHandler;
use gears::baseapp::{QueryRequest, QueryResponse};
use gears::commands::client::query::execute_query;
use gears::commands::client::tx::ClientTxContext;
use gears::commands::node::run::RouterBuilder;
use gears::commands::NilAux;
use gears::commands::NilAuxCommand;
use gears::core::Protobuf;
use gears::crypto::public::PublicKey;
use gears::grpc::health::health_server;
use gears::grpc::tx::tx_server;
use gears::rest::RestState;
use gears::types::address::AccAddress;
use gears::types::tx::Messages;
use ibc_rs::client::cli::query::IbcQueryHandler;
use rest::get_router;
use serde::Serialize;
use slashing::SlashingNodeQueryRequest;
use slashing::SlashingNodeQueryResponse;
use staking::cli::query::StakingQueryHandler;
use staking::StakingNodeQueryRequest;
use staking::StakingNodeQueryResponse;
use tonic::transport::Server;
use tonic::Status;
use tower_layer::Identity;

pub mod abci_handler;
pub mod client;
pub mod config;
pub mod genesis;
pub mod message;
pub mod modules;
pub mod params;
pub mod query;
pub mod rest;
pub mod store_keys;

#[derive(Debug, Clone, Serialize)]
pub struct GaiaApplication;

impl ApplicationInfo for GaiaApplication {
    const APP_NAME: &'static str = env!("PKG_NAME");
    const APP_VERSION: &'static str = env!("GIT_HASH");
}

pub struct GaiaCore;

pub struct GaiaCoreClient;

impl TxHandler for GaiaCoreClient {
    type Message = message::Message;
    type TxCommands = client::WrappedGaiaTxCommands;

    fn prepare_tx(
        &self,
        ctx: &mut ClientTxContext,
        command: Self::TxCommands,
        pubkey: PublicKey,
    ) -> Result<Messages<Self::Message>> {
        tx_command_handler(ctx, command.0, pubkey.get_address())
    }
}

impl QueryHandler for GaiaCoreClient {
    type QueryRequest = GaiaQuery;

    type QueryCommands = WrappedGaiaQueryCommands;

    type QueryResponse = GaiaQueryResponse;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let res = match &command.0 {
            GaiaQueryCommands::Bank(command) => {
                Self::QueryRequest::Bank(BankQueryHandler.prepare_query_request(command)?)
            }
            GaiaQueryCommands::Auth(command) => {
                Self::QueryRequest::Auth(AuthQueryHandler.prepare_query_request(command)?)
            }
            GaiaQueryCommands::Staking(command) => {
                Self::QueryRequest::Staking(StakingQueryHandler.prepare_query_request(command)?)
            }
            GaiaQueryCommands::Ibc(command) => {
                Self::QueryRequest::Ibc(IbcQueryHandler.prepare_query_request(command)?)
            }
        };

        Ok(res)
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let res = match &command.0 {
            GaiaQueryCommands::Bank(command) => Self::QueryResponse::Bank(
                BankQueryHandler.handle_raw_response(query_bytes, command)?,
            ),
            GaiaQueryCommands::Auth(command) => Self::QueryResponse::Auth(
                AuthQueryHandler.handle_raw_response(query_bytes, command)?,
            ),
            GaiaQueryCommands::Staking(command) => Self::QueryResponse::Staking(
                StakingQueryHandler.handle_raw_response(query_bytes, command)?,
            ),
            GaiaQueryCommands::Ibc(command) => {
                Self::QueryResponse::Ibc(IbcQueryHandler.handle_raw_response(query_bytes, command)?)
            }
        };

        Ok(res)
    }
}

impl AuxHandler for GaiaCore {
    type AuxCommands = GaiaAuxCmd;
    type Aux = NilAux;

    fn prepare_aux(&self, cmd: Self::AuxCommands) -> anyhow::Result<Self::Aux> {
        match cmd {
            GaiaAuxCmd::Genutil(cmd) => match cmd {
                genutil::cmd::GenesisCmd::CollectGentxs(cmd) => {
                    genutil::collect_txs::gen_app_state_from_config(cmd, "bank", "genutil")?;
                }
                genutil::cmd::GenesisCmd::Gentx(cmd) => {
                    genutil::gentx::gentx_cmd(cmd, "bank", "staking", &EmptyNodeFetcher)?;
                }
            },
        }

        Ok(NilAux)
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum GaiaAuxCli<AI: ApplicationInfo> {
    #[command(flatten)]
    Genutil(genutil::client::cli::GenesisCommands<AI>),
}

impl<AI: ApplicationInfo> TryFrom<GaiaAuxCli<AI>> for GaiaAuxCmd {
    type Error = anyhow::Error;

    fn try_from(value: GaiaAuxCli<AI>) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            GaiaAuxCli::Genutil(var) => GaiaAuxCmd::Genutil(
                genutil::client::cli::GenesisAuxCli { command: var }.try_into()?,
            ),
        })
    }
}

pub enum GaiaAuxCmd {
    Genutil(genutil::cmd::GenesisCmd),
}

impl AuxHandler for GaiaCoreClient {
    type AuxCommands = NilAuxCommand;
    type Aux = NilAux;
}

impl Client for GaiaCoreClient {}

#[derive(Clone)]
pub enum GaiaNodeQueryRequest {
    Bank(BankNodeQueryRequest),
    Auth(AuthNodeQueryRequest),
    Staking(StakingNodeQueryRequest),
    Slashing(SlashingNodeQueryRequest),
    Distribution(DistributionNodeQueryRequest),
}

impl QueryRequest for GaiaNodeQueryRequest {
    fn height(&self) -> u32 {
        0
    }
}

impl From<BankNodeQueryRequest> for GaiaNodeQueryRequest {
    fn from(req: BankNodeQueryRequest) -> Self {
        GaiaNodeQueryRequest::Bank(req)
    }
}

impl From<AuthNodeQueryRequest> for GaiaNodeQueryRequest {
    fn from(req: AuthNodeQueryRequest) -> Self {
        GaiaNodeQueryRequest::Auth(req)
    }
}

impl From<StakingNodeQueryRequest> for GaiaNodeQueryRequest {
    fn from(req: StakingNodeQueryRequest) -> Self {
        GaiaNodeQueryRequest::Staking(req)
    }
}

impl From<SlashingNodeQueryRequest> for GaiaNodeQueryRequest {
    fn from(req: SlashingNodeQueryRequest) -> Self {
        GaiaNodeQueryRequest::Slashing(req)
    }
}

impl From<DistributionNodeQueryRequest> for GaiaNodeQueryRequest {
    fn from(req: DistributionNodeQueryRequest) -> Self {
        GaiaNodeQueryRequest::Distribution(req)
    }
}

#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum GaiaNodeQueryResponse {
    Bank(BankNodeQueryResponse),
    Auth(AuthNodeQueryResponse),
    Staking(StakingNodeQueryResponse),
    Slashing(SlashingNodeQueryResponse),
    Distribution(DistributionNodeQueryResponse),
}

impl TryFrom<GaiaNodeQueryResponse> for BankNodeQueryResponse {
    type Error = Status;

    fn try_from(res: GaiaNodeQueryResponse) -> Result<Self, Status> {
        match res {
            GaiaNodeQueryResponse::Bank(res) => Ok(res),
            _ => Err(Status::internal(
                "An internal error occurred while querying the application state.",
            )),
        }
    }
}

impl TryFrom<GaiaNodeQueryResponse> for AuthNodeQueryResponse {
    type Error = Status;

    fn try_from(res: GaiaNodeQueryResponse) -> Result<Self, Status> {
        match res {
            GaiaNodeQueryResponse::Auth(res) => Ok(res),
            _ => Err(Status::internal(
                "An internal error occurred while querying the application state.",
            )),
        }
    }
}

impl TryFrom<GaiaNodeQueryResponse> for StakingNodeQueryResponse {
    type Error = Status;

    fn try_from(res: GaiaNodeQueryResponse) -> Result<Self, Status> {
        match res {
            GaiaNodeQueryResponse::Staking(res) => Ok(res),
            _ => Err(Status::internal(
                "An internal error occurred while querying the application state.",
            )),
        }
    }
}

impl TryFrom<GaiaNodeQueryResponse> for SlashingNodeQueryResponse {
    type Error = Status;

    fn try_from(res: GaiaNodeQueryResponse) -> Result<Self, Status> {
        match res {
            GaiaNodeQueryResponse::Slashing(res) => Ok(res),
            _ => Err(Status::internal(
                "An internal error occurred while querying the application state.",
            )),
        }
    }
}

impl TryFrom<GaiaNodeQueryResponse> for DistributionNodeQueryResponse {
    type Error = Status;

    fn try_from(res: GaiaNodeQueryResponse) -> Result<Self, Status> {
        match res {
            GaiaNodeQueryResponse::Distribution(res) => Ok(res),
            _ => Err(Status::internal(
                "An internal error occurred while querying the application state.",
            )),
        }
    }
}

impl QueryResponse for GaiaNodeQueryResponse {
    fn into_bytes(self) -> Vec<u8> {
        todo!()
    }
}

impl Node for GaiaCore {
    type ParamsSubspaceKey = GaiaParamsStoreKey;
    type Handler = GaiaABCIHandler;
    type ApplicationConfig = config::AppConfig;
}

impl RouterBuilder<GaiaNodeQueryRequest, GaiaNodeQueryResponse> for GaiaCore {
    fn build_router<App: NodeQueryHandler<GaiaNodeQueryRequest, GaiaNodeQueryResponse>>(
        &self,
    ) -> Router<RestState<GaiaNodeQueryRequest, GaiaNodeQueryResponse, App>> {
        get_router()
    }

    fn build_grpc_router<App: NodeQueryHandler<GaiaNodeQueryRequest, GaiaNodeQueryResponse>>(
        &self,
        app: App,
    ) -> tonic::transport::server::Router<Identity> {
        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(ibc_proto::FILE_DESCRIPTOR_SET)
            .build_v1()
            .expect("ibc_proto::FILE_DESCRIPTOR_SET is a valid proto file descriptor set");

        Server::builder()
            .add_service(reflection_service)
            .add_service(staking::grpc::new(app.clone()))
            .add_service(auth::grpc::new(app.clone()))
            .add_service(bank::grpc::new(app))
            .add_service(health_server())
            .add_service(tx_server())
    }
}

mod inner {
    pub use bank::types::query::inner::QueryDenomMetadataResponse;
    pub use gears::core::query::response::auth::QueryAccountResponse;
}

#[derive(Debug, Clone)]
pub struct EmptyNodeFetcher;

impl NodeFetcher for EmptyNodeFetcher {
    fn latest_account(
        &self,
        _address: gears::types::address::AccAddress,
        _node: impl AsRef<str>,
    ) -> anyhow::Result<Option<gears::types::account::Account>> {
        Ok(None)
    }

    fn denom_metadata(
        &self,
        _base: gears::types::denom::Denom,
        _node: impl AsRef<str>,
    ) -> anyhow::Result<Option<gears::types::tx::metadata::Metadata>> {
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct QueryNodeFetcher;

impl NodeFetcher for QueryNodeFetcher {
    fn latest_account(
        &self,
        address: AccAddress,
        node: impl AsRef<str>,
    ) -> anyhow::Result<Option<gears::types::account::Account>> {
        let query = QueryAccountRequest { address };

        Ok(
            execute_query::<QueryAccountResponse, inner::QueryAccountResponse>(
                "/cosmos.auth.v1beta1.Query/Account".into(),
                query.encode_vec(),
                node.as_ref(),
                None,
            )?
            .account,
        )
    }

    fn denom_metadata(
        &self,
        base: gears::types::denom::Denom,
        node: impl AsRef<str>,
    ) -> anyhow::Result<Option<gears::types::tx::metadata::Metadata>> {
        let query = QueryDenomMetadataRequest { denom: base };

        Ok(
            execute_query::<QueryDenomMetadataResponse, inner::QueryDenomMetadataResponse>(
                "/cosmos.bank.v1beta1.Query/DenomMetadata".into(),
                query.encode_vec(),
                node.as_ref(),
                None,
            )?
            .metadata,
        )
    }
}
