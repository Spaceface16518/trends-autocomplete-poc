use crate::error::Result;
use std::path::Path;

use crate::{error::name_error, schema::get_fields};
use csv_async::AsyncReaderBuilder;
use rocket::{
    futures::StreamExt,
    serde::Deserialize,
    tokio::fs::{read_dir, File},
};
use tantivy::{doc, IndexWriter};

/// Load data into the index through the `index_writer` given a data directory
/// `data_dir`. The data is expected to provided in several CSV files named
/// `[Semester] [Year].csv`, for example `Spring 2017.csv`. These CSV files must
/// contain all the required data.
pub async fn load_data(mut index_writer: IndexWriter, data_dir: &Path) -> Result<()> {
    let mut dir_reader = read_dir(&data_dir).await?;
    while let Some(file) = dir_reader.next_entry().await? {
        // get the file path
        let path = file.file_name();
        let path: &Path = path.as_ref();
        // skip if this is not a csv file (to allow other files in data directory)
        if path.extension().unwrap_or_default() != "csv" {
            continue;
        }
        // if this is a csv file, extract the non-extension component of the path
        let name = path.file_stem().ok_or_else(|| name_error(&file))?;
        let name = name.to_string_lossy();
        // split the name into two parts
        let mut splits = name.splitn(2, ' ');
        // extract the two parts into the semester and year
        let semester = splits.next().ok_or_else(|| name_error(&file))?;
        let year = splits.next().ok_or_else(|| name_error(&file))?;

        // grab the necessary fields from the schema
        let schema = index_writer.index().schema();
        let [subj, code, sec, sem, yr, prof] = get_fields(
            schema,
            [
                "subject",
                "code",
                "section",
                "semester",
                "year",
                "professor",
            ],
        )?;

        // prepare to write to the index

        // read data from data file
        let reader = File::open(data_dir.join(path)).await?;
        let mut reader = AsyncReaderBuilder::new()
            .has_headers(true)
            .buffer_capacity(1024 * 1024) // 1 MiB buffer
            .create_deserializer(reader);
        // write each record to index
        #[derive(Deserialize, Debug)]
        #[serde(crate = "rocket::serde")]
        struct Record {
            #[serde(rename = "Subject")]
            subj: String,
            #[serde(rename = "Catalog Nbr")]
            #[serde(alias = "Catalog Number")]
            code: String,
            #[serde(rename = "Section")]
            sec: String,
            #[serde(rename = "Instructor 1")]
            ins1: String,
            #[serde(rename = "Instructor 2")]
            ins2: String,
            #[serde(rename = "Instructor 3")]
            ins3: String,
            #[serde(rename = "Instructor 4")]
            ins4: String,
            #[serde(rename = "Instructor 5")]
            ins5: String,
            #[serde(rename = "Instructor 6")]
            ins6: String,
        }
        let mut records = reader.deserialize::<Record>();
        while let Some(record) = records.next().await {
            let record: Record = record?;
            let entry = doc!(
                subj => record.subj,
                code => record.code,
                sec => record.sec,
                sem => semester,
                yr => year,
                prof => record.ins1,
                prof => record.ins2,
                prof => record.ins3,
                prof => record.ins4,
                prof => record.ins5,
                prof => record.ins6,
            );
            index_writer.add_document(entry)?;
        }

        // commit after each file
        index_writer.commit()?;
    }
    index_writer.wait_merging_threads()?;
    Ok(())
}
