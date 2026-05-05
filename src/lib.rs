use regex::Regex;
use serde::{Deserialize, Serialize};
use worker::{wasm_bindgen::JsValue, *};

#[event(fetch)]
async fn fetch(
    req: Request,
    env: Env,
    _ctx: Context,
) -> Result<Response> {
    Router::new()
        .get_async("/submission", process_submissions)
        .get_async("/fetch", process_fetch)
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

#[derive(Deserialize)]
struct SubmissionsRequest {
    contest: String,
    task: Option<String>,
    user: Option<String>,
    language: Option<String>,
    status: Option<String>,
    order_by: Option<String>,
    order_desc: Option<bool>,
    offset: Option<usize>,
    count: Option<i64>,
}

#[derive(Serialize, Deserialize)]
struct SubmissionRow {
    task: String,
    user: String,
    language: String,
    timestamp: usize,
    status: String,
    code_size: usize,
    score: Option<usize>,
    execution_time: Option<usize>,
    memory_usage: Option<usize>,
}

async fn process_submissions(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Ok(query) = req.query::<SubmissionsRequest>() else {
        return api_error("query parsing error");
    };
    
    let d1 = ctx.env.d1("atcoder_submissions_db")?;

    // build statement
    let query = {
        let mut args = vec![];
        let mut where_clauses = vec![];
        {
            args.push(JsValue::from_str(&query.contest));
            where_clauses.push(format!("contests.name = ?{}", args.len()));
            where_clauses.push("tasks.contest_id = contests.id".to_string());
        }
        if let Some(task) = &query.task {
            if !Regex::new(r"^[-_0-9A-Za-z]+$").unwrap().is_match(task) {
                return api_error("invalid task");
            }
            args.push(JsValue::from_str(task));
            where_clauses.push(format!("tasks.name = ?{}", args.len()));
        }
        if let Some(user) = &query.user {
            if !Regex::new(r"^[_0-9A-Za-z]+$").unwrap().is_match(user) {
                return api_error("invalid user");
            }
            args.push(JsValue::from_str(user));
            where_clauses.push(format!("users.name = ?{}", args.len()));
        }
        if let Some(language) = &query.language {
            args.push(JsValue::from_str(language));
            where_clauses.push(format!("languages.description = ?{}", args.len()));
        }
        if let Some(status) = &query.status {
            if !Regex::new(r"^(AC|WA|TLE|MLE|RE|CE|QLE|OLE|IE)$").unwrap().is_match(status) {
                Err("invalid user")?;
            }
            args.push(JsValue::from_str(status));
            where_clauses.push(format!("status = ?{}", args.len()));
        }

        let mut order_by = vec!["timestamp"];
        if let Some(query) = &query.order_by {
            order_by = match query.to_lowercase().as_str() {
                "timestamp" => vec!["timestamp"],
                "score" => vec!["score", "timestamp"],
                "code_size" => vec!["code_size", "timestamp"],
                "execution_time" => vec!["execution_time", "timestamp"],
                "memory_usage" => vec!["memory_usage", "timestamp"],
                _ => Err("invalid order_by")?,
            };
        }

        let mut order = "ASC";
        if query.order_desc == Some(true) {
            order = "DESC";
        }

        let mut offset = 0;
        if let Some(query) = query.offset {
            offset = query;
        }

        let mut count = -1;
        if let Some(query) = query.count {
            if query < 0 {
                Err("invalid count")?;
            }
            count = query;
        }

        let mut stat = String::new();
        stat.push_str("SELECT tasks.name, users.name, languages.description, timestamp, status, code_size, score, execution_time, memory_usage FROM submissions, tasks, contests, users, languages");
        stat.push_str(" WHERE task_id = tasks.id AND user_id = users.id AND language_id = languages.id");
        for clause in where_clauses {
            stat.push_str(" AND ");
            stat.push_str(&clause);
        }
        stat.push_str(" ORDER BY ");
        stat.push_str(&order_by.iter().map(|key| format!("{key} {order}") ).collect::<Vec<_>>().join(", "));
        stat.push_str(&format!(" LIMIT {offset}, {count}"));
        // Err(format!("{stat:?}").as_str())?;
        d1.prepare(stat).bind(&args)?
    };
    let result = query.all().await?.results::<SubmissionRow>()?;

    Response::from_json(&APIResponse {
        error: None,
        result: Some(result),
    })
}


async fn process_fetch(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    Response::ok("")
}