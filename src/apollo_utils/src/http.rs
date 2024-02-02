use ic_cdk::{
    api::management_canister::http_request::{
        HttpResponse, TransformArgs, TransformContext, TransformFunc,
    },
    query,
};
use ic_web3_rs::{
    transforms::{processors, transform::TransformProcessor},
    transports::ic_http_client::{CallOptions, CallOptionsBuilder},
};

#[query]
fn transform(response: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: response.response.status,
        body: response.response.body,
        headers: Vec::new(),
    }
}

#[query]
fn transform_tx_with_logs(args: TransformArgs) -> HttpResponse {
    crate::processors::raw_tx_execution_transform_processor().transform(args)
}

#[query]
fn transform_tx(args: TransformArgs) -> HttpResponse {
    processors::send_transaction_processor().transform(args)
}

pub fn transform_ctx() -> CallOptions {
    get_transform_ctx("transform")
}

pub fn transform_ctx_tx_with_logs() -> CallOptions {
    get_transform_ctx("transform_tx_with_logs")
}

pub fn transform_ctx_tx() -> CallOptions {
    get_transform_ctx("transform_tx")
}

fn get_transform_ctx(method: &str) -> CallOptions {
    CallOptionsBuilder::default()
        .transform(Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: method.into(),
            }),
            context: vec![],
        }))
        .cycles(None)
        .max_resp(None)
        .build()
        .expect("failed to build call options")
}
