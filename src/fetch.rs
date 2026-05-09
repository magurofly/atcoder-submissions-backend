use std::{ops::DerefMut, task::Poll};

use bytes::Bytes;
use futures_core::Stream;
use itertools::izip;
use serde::Deserialize;
use tokio_stream::StreamExt as _;
use tokio_util::io::StreamReader;
use worker::{crypto::{DigestStream, DigestStreamAlgorithm}, send::SendWrapper, wasm_bindgen::JsValue, *};

// const SUBMISSIONS_URL: &'static str = "https://s3-ap-northeast-1.amazonaws.com/kenkoooo/submissions.csv.gz";
const SUBMISSIONS_URL: &'static str = "http://localhost:8000/submissions100.csv.gz";
const BATCH_NUM: usize = 1000;

#[derive(Deserialize)]
struct FetchRequest {
    time: u64, // now - 60 < time <= now
    token: String, // sha256(time + ':' + key)
}
impl FetchRequest {
    async fn check_token(&self, key: &str) -> bool {
        let now = Date::now().as_millis() / 1000;
        if !(now - 60 < self.time && self.time <= now) {
            return false;
        }

        let mut input = String::new();
        input.push_str(&self.time.to_string());
        input.push(':');
        input.push_str(key);
        let req_init = web_sys::RequestInit::new();
        req_init.set_method("POST");
        req_init.set_body(&JsValue::from_str(&input));
        let req = web_sys::Request::new_with_str_and_init("http://internal", &req_init).unwrap();
        let read_stream = req.body().unwrap();
        let digest_stream = DigestStream::new(DigestStreamAlgorithm::Sha256);
        let _ = read_stream.pipe_to(digest_stream.raw());
        let bytes = digest_stream.digest().await.unwrap().to_vec();
        let output = hex::encode(bytes);
        output == self.token
    }
}

#[derive(Deserialize, Debug)]
struct CsvRow {
    id: String,
    epoch_second: f64,
    problem_id: String,
    contest_id: String,
    user_id: String,
    language: String,
    point: f64,
    length: i32,
    result: String,
    execution_time: Option<f64>,
    #[serde(skip_deserializing)]
    url: Option<String>,
}

#[derive(Deserialize)]
struct IdRow {
    id: i32,
}

pub async fn process_fetch(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let request = req.query::<FetchRequest>()?;
    if !request.check_token(&ctx.env.secret("FETCH_KEY")?.to_string()).await {
        return Ok(Response::from_html("Bad Token")?.with_status(400));
    }

    let url = Url::parse(SUBMISSIONS_URL)?;
    let mut result = Fetch::Url(url).send().await?;
    let send_stream = SendByteStream(SendWrapper::new(result.stream()?));
    let bytes_stream = send_stream.map(|res| res.map(Bytes::from).map_err(std::io::Error::other) );
    let bytes_reader = StreamReader::new(bytes_stream);
    let mut decoder = async_compression::tokio::bufread::GzipDecoder::new(bytes_reader);
    decoder.multiple_members(true);
    let mut deserializer = csv_async::AsyncReaderBuilder::new().create_deserializer(decoder);
    let mut stream = deserializer.deserialize::<CsvRow>();
    let d1 = ctx.env.d1("atcoder_submissions_db")?;
    // [contest_name, task_name, user_name, language_description, timestamp, score, code_size, status, execution_time]
    // let statement = d1.prepare("
    // WITH
    //     t AS (INSERT INTO tasks(contest_name, task_name) VALUES (?1, ?2) ON CONFLICT DO UPDATE SET task_name = task_name RETURNING id),
    //     u AS (INSERT INTO users(name) VALUES (?3) ON CONFLICT DO UPDATE SET name = name RETURNING id),
    //     l AS (INSERT INTO languages(name) VALUES (?4) ON CONFLICT DO UPDATE SET name = name RETURNING id)
    //     INSERT INTO submissions(task_id, user_id, language_id, timestamp, score, code_size, status, execution_time, url)
    //         SELECT t.id, u.id, l.id, ?5, ?6, ?7, ?8, ?9, ?10 FROM t, u, l;
    // ");
    let stat_tasks = d1.prepare("INSERT INTO tasks(contest_name, task_name) VALUES (?1, ?2) ON CONFLICT DO UPDATE SET task_name = task_name RETURNING id");
    let stat_users = d1.prepare("INSERT INTO users(name) VALUES (?1) ON CONFLICT DO UPDATE SET name = name RETURNING id");
    let stat_languages = d1.prepare("INSERT INTO languages(name) VALUES (?1) ON CONFLICT DO UPDATE SET name = name RETURNING id");
    let stat_insert = d1.prepare("INSERT INTO submissions(task_id, user_id, language_id, timestamp, score, code_size, status, execution_time, url) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);");

    let mut buffer: Vec<CsvRow> = vec![];
    let run_query = async |buffer: &Vec<CsvRow>| -> Result<(), worker::Error> {
        let task_args = buffer.iter().map(|row| vec![D1Type::Text(&row.contest_id), D1Type::Text(&row.problem_id)] ).collect::<Vec<_>>();
        let task_ids = d1.batch(stat_tasks.batch_bind(task_args.iter())?).await?;
        let user_args = buffer.iter().map(|row| vec![D1Type::Text(&row.user_id)] ).collect::<Vec<_>>();
        let user_ids = d1.batch(stat_users.batch_bind(user_args.iter())?).await?;
        let language_args = buffer.iter().map(|row| vec![D1Type::Text(&row.language)] ).collect::<Vec<_>>();
        let language_ids = d1.batch(stat_languages.batch_bind(language_args.iter())?).await?;
        let insert_args = izip!(task_ids.iter(), user_ids.iter(), language_ids.iter(), buffer.iter()).map(|(task_id, user_id, language_id, csv)| {
            let task_id = D1Type::Integer(task_id.results::<IdRow>()?.pop().unwrap().id);
            let user_id = D1Type::Integer(user_id.results::<IdRow>()?.pop().unwrap().id);
            let language_id = D1Type::Integer(language_id.results::<IdRow>()?.pop().unwrap().id);
            let timestamp = D1Type::Real(csv.epoch_second);
            let score = D1Type::Real(csv.point);
            let code_size = D1Type::Integer(csv.length);
            let status = D1Type::Text(&csv.result);
            let execution_time = csv.execution_time.map(D1Type::Real).unwrap_or(D1Type::Null);
            let url = D1Type::Text(&csv.url.as_ref().unwrap());
            Ok(vec![task_id, user_id, language_id, timestamp, score, code_size, status, execution_time, url])
        }).collect::<worker::Result<Vec<_>>>()?;
        d1.batch(stat_insert.batch_bind(insert_args.iter())?).await?;
        Ok(())
    };

    while let Some(row) = stream.next().await {
        let mut row = row.map_err(std::io::Error::other)?;
        row.url = Some(format!("https://atcoder.jp/contests/{}/submissions/{}", row.contest_id, row.id));
        buffer.push(row);
        if buffer.len() >= BATCH_NUM {
            run_query(&buffer).await?;
            buffer.clear();
        }
    }
    if !buffer.is_empty() {
        run_query(&buffer).await?;
        buffer.clear();
    }

    Response::from_html("OK")
}

struct SendByteStream(SendWrapper<ByteStream>);
impl Stream for SendByteStream {
    type Item = Result<Vec<u8>>;
    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        std::pin::pin!(self.0.deref_mut()).poll_next(cx)
    }
}