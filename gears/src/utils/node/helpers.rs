use address::AccAddress;
use bytes::Bytes;

use core_types::Protobuf as _;
use extensions::infallible::UnwrapInfallible;
use prost::Message;
use tendermint::types::chain_id::ChainId;

use crate::{
    crypto::info::SigningInfo,
    types::{
        auth::fee::Fee,
        base::coins::Coins,
        tx::{body::TxBody, Tx, TxMessage},
    },
};

use super::User;

pub const ACC_ADDRESS: &str = "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux";

pub fn acc_address() -> AccAddress {
    AccAddress::from_bech32(ACC_ADDRESS).expect("Default Address should be valid")
}

pub fn generate_txs<M: TxMessage>(
    msgs: impl IntoIterator<Item = (u64, M)>,
    user: &User,
    chain_id: ChainId,
) -> Vec<Bytes> {
    let fee = Fee {
        amount: Some(
            Coins::new(vec!["1uatom".parse().expect("hard coded coin is valid")])
                .expect("hard coded coins are valid"),
        ),
        gas_limit: 200_000_u64
            .try_into()
            .expect("hard coded gas limit is valid"),
        payer: None,
        granter: "".into(),
    };

    let mut result = Vec::new();

    for (sequence, msg) in msgs {
        let signing_info = SigningInfo {
            key: &user.key_pair,
            sequence,
            account_number: user.account_number,
        };

        let body = TxBody::new_with_defaults(vec1::vec1![msg]);

        let Tx {
            body,
            auth_info,
            signatures,
            signatures_data: _,
        } = crate::crypto::info::create_signed_transaction_direct(
            vec![signing_info],
            chain_id.to_owned(),
            fee.to_owned(),
            None,
            body,
        )
        .unwrap_infallible();

        result.push(
            core_types::tx::raw::TxRaw {
                body_bytes: body.encode_vec(),
                auth_info_bytes: auth_info.encode_vec(),
                signatures,
            }
            .encode_to_vec()
            .into(),
        )
    }

    result
}
