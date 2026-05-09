use serde::Serialize;
use worker::*;

pub mod submissions;
pub mod fetch;

#[event(fetch)]
async fn fetch(
    req: Request,
    env: Env,
    _ctx: Context,
) -> Result<Response> {
    Router::new()
        .get_async("/submissions", submissions::process_submissions)
        .get_async("/fetch", fetch::process_fetch)
        .run(req, env)
        .await
}

#[derive(Serialize)]
struct APIResponse<T: Serialize> {
    error: Option<String>,
    result: Option<T>,
}

fn api_error(error: &str) -> Result<Response> {
    return Response::from_json(&APIResponse::<()> { error: Some(error.to_string()), result: None });
}