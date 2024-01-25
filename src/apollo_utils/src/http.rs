use ic_cdk::{
    api::management_canister::http_request::{
        HttpResponse, TransformArgs, TransformContext, TransformFunc,
    },
    query,
};
use ic_web3_rs::transports::ic_http_client::{CallOptions, CallOptionsBuilder};

#[query]
fn transform(response: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: response.response.status,
        body: response.response.body,
        headers: Vec::new(),
    }
}

pub fn transform_ctx() -> CallOptions {
    get_transform_ctx("transform")
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
