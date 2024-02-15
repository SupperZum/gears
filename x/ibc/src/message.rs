use proto_messages::cosmos::ibc_types::protobuf::Any;
use proto_types::AccAddress;

#[derive(Debug, Clone, serde::Serialize)]
pub enum Message {
    // TODO: More strict struct which serializes in proto-types crate
    ClientCreate(ibc::core::client::types::proto::v1::MsgCreateClient),
    ClientUpdate(ibc::core::client::types::proto::v1::MsgUpdateClient),
    SubmitMisbehaviour(ibc::core::client::types::proto::v1::MsgSubmitMisbehaviour),
    RecoverClient(ibc::core::client::types::proto::v1::MsgRecoverClient),
}

impl proto_messages::cosmos::tx::v1beta1::message::Message for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        unimplemented!()
    }

    fn validate_basic(&self) -> Result<(), String> {
        unimplemented!()
    }

    fn type_url(&self) -> &'static str {
        unimplemented!()
    }
}

impl From<Message> for Any {
    fn from(_msg: Message) -> Self {
        unimplemented!()
    }
}

impl TryFrom<Any> for Message {
    type Error = proto_messages::Error;

    fn try_from(_value: Any) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}
