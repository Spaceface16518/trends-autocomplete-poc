use std::path::PathBuf;

use itertools::Itertools;
use rocket::{
    fairing::AdHoc,
    fs::FileServer,
    get,
    http::Status,
    routes,
    serde::{json::Json, Deserialize},
    State,
};
use tantivy::{
    collector::TopDocs, doc, query::QueryParser, schema::NamedFieldDocument, Index, IndexReader,
};

use crate::error::Error;

use self::{
    fairings::{load_data_fairing, tantivy_setup_fairing},
    util::RequestTimer,
};

mod data;
mod error;
mod fairings;
mod schema;
mod util;

/// Autocompletion endpoint
///
/// Given a query parameter `q`, it returns a list of suggestions as JSON.
#[get("/complete?<q>")]
fn complete(
    q: &str,
    index: &State<Index>,
    index_reader: &State<IndexReader>,
    query_parser: &State<QueryParser>,
) -> Result<Json<Vec<NamedFieldDocument>>, Error> {
    let searcher = index_reader.searcher();
    let query = query_parser.parse_query(q)?;
    // we're only interested in the first 5 documents
    // TODO: make this limit configurable
    let top_docs = searcher.search(&query, &TopDocs::with_limit(5))?;
    let schema = index.schema();
    // TODO: don't abort on first failure
    let results = top_docs
        .into_iter()
        .map(|(_score, address)| searcher.doc(address))
        .map_ok(|doc| schema.to_named_doc(&doc))
        .collect::<Result<Vec<_>, _>>()?;
    // TODO: correctly format results before sending
    Ok(Json(results))
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Config {
    index_dir: Option<PathBuf>,
    data_dir: PathBuf,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let rocket = rocket::build()
        .attach(AdHoc::config::<Config>())
        .attach(tantivy_setup_fairing())
        .attach(load_data_fairing())
        .attach(RequestTimer)
        .mount("/", routes![complete, healthcheck])
        .mount("/", FileServer::from("dist"));
    let _rocket = rocket.launch().await?;

    Ok(())
}

#[get("/healthz")]
fn healthcheck() -> Status {
    Status::Ok
}
