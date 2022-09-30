use rocket::fairing::{AdHoc, Fairing};
use tantivy::Index;

use crate::{
    data::load_data,
    schema::{index, query_parser, register_tokenizers, schema, INDEX_WRITER_ARENA_SIZE},
    Config,
};

pub fn tantivy_setup_fairing() -> impl Fairing {
    AdHoc::try_on_ignite("Set up Tantivy", |rocket| async {
        let Config { index_dir, .. } = rocket.state().expect("config not loaded");
        match index(schema(), index_dir.as_deref()) {
            Ok(index) => {
                register_tokenizers(&index);
                let reader = index.reader().unwrap();
                let query_parser = query_parser(&index).unwrap();
                Ok(rocket.manage(index).manage(reader).manage(query_parser))
            }
            Err(_err) => Err(rocket),
        }
    })
}

pub fn load_data_fairing() -> impl Fairing {
    // TODO: error handling
    AdHoc::on_liftoff("Load Data", |rocket| {
        Box::pin(async move {
            let Config { data_dir, .. } = rocket.state().expect("config not loaded");
            let index = rocket
                .state::<Index>()
                .expect("tantivy not loaded into rocket");
            let index_writer = index.writer(INDEX_WRITER_ARENA_SIZE).unwrap();
            load_data(index_writer, data_dir).await.unwrap();
        })
    })
}
