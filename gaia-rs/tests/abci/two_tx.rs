use gears::types::base::coins::Coins;
use gears::types::msg::send::MsgSend;
use gears::utils::node::generate_txs;
use gears::{tendermint::types::time::timestamp::Timestamp, types::address::AccAddress};

use crate::setup_mock_node;

#[test]
/// In this scenario, we test the initialization of the application and submit a balance transfer on block three in two different blocks
fn two_tx_in_different_block() {
    let (mut node, user) = setup_mock_node(None::<&str>);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH);
    assert_eq!(
        hex::encode(app_hash),
        "36fd98b5248f0e4bfa6ef4e311134403b1b3deb8865bdbba7187cf05e5644a83"
    );

    node.step(vec![], Timestamp::UNIX_EPOCH);
    node.step(vec![], Timestamp::UNIX_EPOCH);

    let to_address: AccAddress = "cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut"
        .parse()
        .expect("hard coded address is valid");
    let amount = Coins::new(vec!["10uatom".parse().expect("hard coded coin is valid")])
        .expect("hard coded coins are valid");

    let msg = gaia_rs::message::Message::Bank(bank::Message::Send(MsgSend {
        from_address: user.address(),
        to_address: to_address.to_owned(),
        amount: amount.to_owned(),
    }));

    let txs = generate_txs([(0, msg)], &user, node.chain_id().clone());

    let app_hash = node.step(txs, Timestamp::UNIX_EPOCH);
    assert_eq!(
        hex::encode(app_hash),
        "8eb5f41a3f77e034185be06e5385ff0d0a42f8d0f59171b1cc12b1ac6a66bbef"
    );

    let msg = gaia_rs::message::Message::Bank(bank::Message::Send(MsgSend {
        from_address: user.address(),
        to_address,
        amount,
    }));

    let txs = generate_txs([(1, msg)], &user, node.chain_id().clone());

    let app_hash = node.step(txs, Timestamp::UNIX_EPOCH);
    assert_eq!(
        hex::encode(app_hash),
        "8b8d21c8e981b475ae1ed9db0d17810c5828d16a03d902f5d79787a303f9cd33"
    );
}

#[test]
/// In this scenario, we test the initialization of the application and submit a balance transfer on block three in single block with changed sequence
fn two_tx_in_single_block() {
    let (mut node, user) = setup_mock_node(None::<&str>);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH);
    assert_eq!(
        hex::encode(app_hash),
        "36fd98b5248f0e4bfa6ef4e311134403b1b3deb8865bdbba7187cf05e5644a83"
    );

    node.step(vec![], Timestamp::UNIX_EPOCH);
    node.step(vec![], Timestamp::UNIX_EPOCH);

    let to_address: AccAddress = "cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut"
        .parse()
        .expect("hard coded address is valid");
    let amount = Coins::new(vec!["10uatom".parse().expect("hard coded coin is valid")])
        .expect("hard coded coins are valid");

    let msg1 = gaia_rs::message::Message::Bank(bank::Message::Send(MsgSend {
        from_address: user.address(),
        to_address: to_address.to_owned(),
        amount: amount.to_owned(),
    }));

    let msg2 = gaia_rs::message::Message::Bank(bank::Message::Send(MsgSend {
        from_address: user.address(),
        to_address,
        amount,
    }));

    let txs = generate_txs([(0, msg1), (1, msg2)], &user, node.chain_id().clone());

    let app_hash = node.step(txs, Timestamp::UNIX_EPOCH);
    assert_eq!(
        hex::encode(app_hash),
        "1fa056a16da50831fe673b592ad83628a57d6a15cc8877edb9b85a0e9b5e1797"
    );
}
