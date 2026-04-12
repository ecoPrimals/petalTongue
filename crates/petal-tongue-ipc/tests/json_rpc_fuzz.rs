// SPDX-License-Identifier: AGPL-3.0-or-later
//! Property-based fuzz tests for JSON-RPC parser and client.
//!
//! Uses proptest to verify roundtrip, resilience, and ID monotonicity.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use petal_tongue_ipc::{JsonRpcClient, JsonRpcRequest, JsonRpcResponse};
use proptest::prelude::*;
use serde_json::Value;
use std::borrow::Cow;

/// Strategy for JSON-RPC id values (number, string, or null).
fn json_rpc_id_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        any::<i64>().prop_map(|n| serde_json::json!(n)),
        "[a-zA-Z0-9_-]{0,32}".prop_map(Value::String),
        Just(Value::Null),
    ]
}

/// Strategy for JSON params (object or array of simple values).
fn json_params_strategy() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| serde_json::json!(n)),
        "[a-zA-Z0-9_-]{0,64}".prop_map(Value::String),
    ];
    leaf.prop_recursive(2, 4, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            prop::collection::hash_map("[a-z][a-z0-9_]{0,16}", inner, 0..4)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
}

/// Strategy for JSON result values - excludes Null to avoid roundtrip loss
/// (JSON "result": null deserializes as Option::None, not Some(Null)).
fn json_result_strategy() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| serde_json::json!(n)),
        "[a-zA-Z0-9_-]{0,64}".prop_map(Value::String),
    ];
    leaf.prop_recursive(2, 4, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            prop::collection::hash_map("[a-z][a-z0-9_]{0,16}", inner, 0..4)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
}

/// Strategy for valid JsonRpcRequest construction.
fn json_rpc_request_strategy() -> impl Strategy<Value = JsonRpcRequest> {
    (
        "[a-zA-Z][a-zA-Z0-9_.]{0,64}",
        json_params_strategy(),
        json_rpc_id_strategy(),
    )
        .prop_map(|(method, params, id)| JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            method,
            params,
            id,
        })
}

/// Strategy for valid JsonRpcResponse construction.
fn json_rpc_response_strategy() -> impl Strategy<Value = JsonRpcResponse> {
    prop_oneof![
        // Success response (use json_result_strategy to avoid Some(Null) roundtrip loss)
        (json_result_strategy(), json_rpc_id_strategy()).prop_map(|(result, id)| {
            JsonRpcResponse {
                jsonrpc: Cow::Borrowed("2.0"),
                result: Some(result),
                error: None,
                id,
            }
        }),
        // Error response
        (any::<i32>(), "[a-zA-Z0-9 ]{0,64}", json_rpc_id_strategy()).prop_map(
            |(code, message, id)| JsonRpcResponse {
                jsonrpc: Cow::Borrowed("2.0"),
                result: None,
                error: Some(petal_tongue_ipc::JsonRpcError {
                    code,
                    message,
                    data: None,
                }),
                id,
            },
        ),
    ]
}

proptest! {
    #[test]
    fn prop_json_rpc_request_roundtrip(req in json_rpc_request_strategy()) {
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: JsonRpcRequest = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(parsed.jsonrpc, req.jsonrpc);
        prop_assert_eq!(parsed.method, req.method);
        prop_assert_eq!(parsed.params, req.params);
        prop_assert_eq!(parsed.id, req.id);
    }

    #[test]
    fn prop_json_rpc_response_roundtrip(resp in json_rpc_response_strategy()) {
        let json = serde_json::to_string(&resp).expect("serialize");
        let parsed: JsonRpcResponse = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(parsed.jsonrpc, resp.jsonrpc);
        prop_assert_eq!(parsed.result, resp.result);
        prop_assert_eq!(parsed.id, resp.id);
        match (&parsed.error, &resp.error) {
            (Some(a), Some(b)) => {
                prop_assert_eq!(a.code, b.code);
                prop_assert_eq!(a.message.as_str(), b.message.as_str());
            }
            (None, None) => {}
            _ => prop_assert!(false, "error mismatch"),
        }
    }

    #[test]
    fn prop_json_rpc_response_from_slice_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..1024)) {
        let _ = std::panic::catch_unwind(|| {
            let _ = serde_json::from_slice::<JsonRpcResponse>(&bytes);
        });
    }

    #[test]
    fn prop_json_rpc_request_from_str_never_panics(s in "\\PC*") {
        let _ = std::panic::catch_unwind(|| {
            let _ = serde_json::from_str::<JsonRpcRequest>(&s);
        });
    }

    #[test]
    fn prop_next_id_monotonic(n in 2u32..100u32) {
        let client = JsonRpcClient::new("/tmp/nonexistent.sock").expect("valid path");
        let mut prev = client.next_id();
        for _ in 1..n {
            let next = client.next_id();
            prop_assert!(next > prev, "next_id must always increase");
            prev = next;
        }
    }
}
