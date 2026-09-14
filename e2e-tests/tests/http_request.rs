use candid::{Decode, Encode, Principal};
use pocket_ic::PocketIc;
use pocket_ic::common::rest::{
    CanisterHttpHeader, CanisterHttpPricingVersion, CanisterHttpReject, CanisterHttpReplication,
    CanisterHttpReply, CanisterHttpRequest, CanisterHttpResponse, MockCanisterHttpResponse,
    MockFlexibleCanisterHttpResponse,
};

mod test_utilities;
use test_utilities::{cargo_build_canister, pic_base};

#[test]
fn test_http_request() {
    let wasm = cargo_build_canister("http_request");
    let pic = pic_base().build();

    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 3_000_000_000_000u128);
    pic.install_canister(canister_id, wasm, vec![], None);

    test_one_http_request(&pic, canister_id, "get_without_transform");
    test_one_http_request(&pic, canister_id, "post");
    test_one_http_request(&pic, canister_id, "head");
    test_one_http_request(&pic, canister_id, "get_with_transform");
    test_one_http_request(&pic, canister_id, "get_with_transform_closure");
    test_one_http_request(&pic, canister_id, "non_replicated");
    // `get_cost` is pure, so this needs no mocked response.
    pic.update_call(
        canister_id,
        Principal::anonymous(),
        "expected_usage_lowers_cost",
        vec![],
    )
    .expect("expected_usage_lowers_cost failed");
}

#[test]
fn test_flexible_http_request() {
    let wasm = cargo_build_canister("http_request");
    let pic = pic_base().build();

    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 3_000_000_000_000u128);
    pic.install_canister(canister_id, wasm, vec![], None);

    // Answering exactly `min_responses` of the committee is what the outcall waits for, so
    // that is how many responses come back.
    let mut expected_responses = 0;
    let res = test_one_flexible_http_request(&pic, canister_id, "flexible_default", |request| {
        let CanisterHttpReplication::Flexible { min_responses, .. } = request.replication else {
            panic!("expected a flexible outcall, got {:?}", request.replication);
        };
        expected_responses = min_responses;
        vec![reply(); min_responses as usize]
    });
    assert_eq!(Decode!(&res, u32).unwrap(), expected_responses);

    test_one_flexible_http_request(&pic, canister_id, "flexible_single", |_| vec![reply()]);
    test_one_flexible_http_request(&pic, canister_id, "flexible_with_transform_closure", |_| {
        vec![reply()]
    });
    test_one_flexible_http_request(&pic, canister_id, "flexible_too_many_rejects", |_| {
        vec![CanisterHttpResponse::CanisterHttpReject(
            CanisterHttpReject {
                reject_code: 1,
                message: "rejected".to_string(),
            },
        )]
    });
    // `get_cost` is pure, so these need no mocked response.
    for method in [
        "flexible_expected_usage_lowers_cost",
        "flexible_default_caps_transformed_bytes",
    ] {
        pic.update_call(canister_id, Principal::anonymous(), method, vec![])
            .unwrap_or_else(|e| panic!("{method} failed: {e}"));
    }
}

fn reply() -> CanisterHttpResponse {
    CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
        status: 200,
        headers: vec![CanisterHttpHeader {
            name: "response_header_name".to_string(),
            value: "response_header_value".to_string(),
        }],
        body: vec![42],
    })
}

/// Answers the committee of one pending flexible outcall with what `responses` builds from
/// the replication counts the outcall was issued with.
fn test_one_flexible_http_request(
    pic: &PocketIc,
    canister_id: Principal,
    method: &str,
    responses: impl FnOnce(&CanisterHttpRequest) -> Vec<CanisterHttpResponse>,
) -> Vec<u8> {
    let call_id = pic
        .submit_call(
            canister_id,
            Principal::anonymous(),
            method,
            Encode!(&()).unwrap(),
        )
        .unwrap();
    let canister_http_requests = tick_until_next_request(pic);
    assert_eq!(canister_http_requests.len(), 1);
    let request = &canister_http_requests[0];
    assert!(matches!(
        request.replication,
        CanisterHttpReplication::Flexible { .. }
    ));
    // Flexible outcalls are priced by version 2 only.
    assert_eq!(
        request.pricing_version,
        CanisterHttpPricingVersion::PayAsYouGo
    );
    pic.mock_flexible_canister_http_response(MockFlexibleCanisterHttpResponse {
        subnet_id: request.subnet_id,
        request_id: request.request_id,
        responses: responses(request),
    });
    pic.await_call(call_id).unwrap()
}

fn test_one_http_request(pic: &PocketIc, canister_id: Principal, method: &str) {
    let call_id = pic
        .submit_call(
            canister_id,
            Principal::anonymous(),
            method,
            Encode!(&()).unwrap(),
        )
        .unwrap();
    let canister_http_requests = tick_until_next_request(pic);
    assert_eq!(canister_http_requests.len(), 1);
    let request = &canister_http_requests[0];
    // The builder switched to version 2 pricing exclusively.
    assert_eq!(
        request.pricing_version,
        CanisterHttpPricingVersion::PayAsYouGo
    );
    pic.mock_canister_http_response(MockCanisterHttpResponse {
        subnet_id: request.subnet_id,
        request_id: request.request_id,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: vec![CanisterHttpHeader {
                name: "response_header_name".to_string(),
                value: "response_header_value".to_string(),
            }],
            body: vec![42],
        }),
        additional_responses: vec![],
    });
    pic.await_call(call_id).unwrap();
}

fn tick_until_next_request(pic: &PocketIc) -> Vec<CanisterHttpRequest> {
    for _ in 0..10 {
        let requests = pic.get_canister_http();
        if !requests.is_empty() {
            return requests;
        }
        pic.tick();
    }
    vec![]
}
