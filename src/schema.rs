use std::path::Path;

use crate::error::{Error, Result};
use tantivy::{
    directory::MmapDirectory,
    query::QueryParser,
    schema::{
        Field, IndexRecordOption, Schema, SchemaBuilder, TextFieldIndexing, TextOptions, STORED,
        STRING,
    },
    tokenizer::{AlphaNumOnlyFilter, LowerCaser, NgramTokenizer, TextAnalyzer},
    Index,
};

/// Construct an n-gram tokenizer text analyzer.
fn ngram(min_gram: usize, max_gram: usize) -> TextAnalyzer {
    TextAnalyzer::from(NgramTokenizer::all_ngrams(min_gram, max_gram))
        .filter(AlphaNumOnlyFilter)
        .filter(LowerCaser)
}

/// Builds the schema of the search store. This schema is saved in `meta.json`
/// under the index directory.
pub fn schema() -> Schema {
    let mut schema_builder = SchemaBuilder::new();
    let text_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_index_option(IndexRecordOption::WithFreqsAndPositions)
                .set_tokenizer("ngram1-4"),
        )
        .set_stored();
    schema_builder.add_text_field("subject", text_options.clone());
    schema_builder.add_text_field("code", text_options.clone());
    schema_builder.add_text_field("section", STRING | STORED);
    schema_builder.add_u64_field("semester", STORED);
    schema_builder.add_u64_field("year", STORED);
    let text_options = text_options.set_indexing_options(
        TextFieldIndexing::default()
            .set_index_option(IndexRecordOption::WithFreqsAndPositions)
            .set_tokenizer("ngram3-5"),
    );
    schema_builder.add_text_field("professor", text_options);
    schema_builder.build()
}

/// Get all fields from a schema specified by literal names. Returns an error if
/// one of the fields does not exist in the schema.
pub fn get_fields<const N: usize>(schema: Schema, names: [&'static str; N]) -> Result<[Field; N]> {
    let mut fields = [Field::from_field_id(0); N];
    fields
        .iter_mut()
        .zip(names.into_iter())
        .try_for_each(|(field, name)| {
            schema
                .get_field(name)
                .ok_or(Error::Field(name))
                .map(|f| *field = f)
        })
        .map(|_| fields)?;
    Ok(fields)
}

/// Allow the index writer 5 MiB of memory to write to the index
pub(crate) const INDEX_WRITER_ARENA_SIZE: usize = 5 * 1024 * 1024;

/// Construct the index based on the given schema. If an index directory is not
/// passed, the index is created in RAM. This index is used by tantivy to store
/// documents.
pub fn index(schema: Schema, index_dir: Option<&Path>) -> Result<Index> {
    let index = if let Some(index_dir) = index_dir {
        Index::open_or_create(MmapDirectory::open(index_dir)?, schema)?
    } else {
        Index::create_in_ram(schema)
    };
    Ok(index)
}

/// Registers the applications desired tokenizers with the given index. Right
/// now, these are `ngram2-4`, `ngram1-4`, and `ngram3-5`.
pub fn register_tokenizers(index: &Index) {
    let tokenizer_manager = index.tokenizers();
    tokenizer_manager.register("ngram2-4", ngram(2, 4));
    tokenizer_manager.register("ngram1-4", ngram(1, 4));
    tokenizer_manager.register("ngram3-5", ngram(3, 5));
}

pub fn query_parser(index: &Index) -> Result<QueryParser> {
    let [subj, code, professor] = get_fields(index.schema(), ["subject", "code", "professor"])?;
    let mut query_parser = QueryParser::for_index(index, vec![subj, code, professor]);
    // use ANDs by default
    //query_parser.set_conjunction_by_default();

    // boost prefix and catalog code so they appear above professor results
    query_parser.set_field_boost(subj, 3.0);
    query_parser.set_field_boost(code, 2.0);

    Ok(query_parser)
}
