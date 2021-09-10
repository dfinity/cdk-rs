use super::{
    fork, labeled,
    HashTree::{Empty, Leaf},
};
use std::borrow::Cow;

//─┬─┬╴"a" ─┬─┬╴"x" ─╴"hello"
// │ │      │ └╴Empty
// │ │      └╴  "y" ─╴"world"
// │ └╴"b" ──╴"good"
// └─┬╴"c" ──╴Empty
//   └╴"d" ──╴"morning"
#[test]
fn test_public_spec_example() {
    let t = fork(
        fork(
            labeled(
                b"a",
                fork(
                    fork(labeled(b"x", Leaf(Cow::Borrowed(b"hello"))), Empty),
                    labeled(b"y", Leaf(Cow::Borrowed(b"world"))),
                ),
            ),
            labeled(b"b", Leaf(Cow::Borrowed(b"good"))),
        ),
        fork(
            labeled(b"c", Empty),
            labeled(b"d", Leaf(Cow::Borrowed(b"morning"))),
        ),
    );

    assert_eq!(
        hex::encode(&t.reconstruct()[..]),
        "eb5c5b2195e62d996b84c9bcc8259d19a83786a2f59e0878cec84c811f669aa0".to_string()
    );

    assert_eq!(
        hex::encode(serde_cbor::to_vec(&t).unwrap()),
        "8301830183024161830183018302417882034568656c6c6f810083024179820345776f726c6483024162820344676f6f648301830241638100830241648203476d6f726e696e67".to_string());
}
