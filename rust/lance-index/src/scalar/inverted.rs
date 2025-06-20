// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

pub mod builder;
mod encoding;
mod index;
mod iter;
mod merger;
pub mod query;
mod scorer;
pub mod tokenizer;
mod wand;

pub use builder::InvertedIndexBuilder;
pub use index::*;
use lance_core::Result;
pub use tokenizer::*;

use super::btree::TrainingSource;
use super::IndexStore;

pub async fn train_inverted_index(
    data_source: Box<dyn TrainingSource>,
    index_store: &dyn IndexStore,
    params: InvertedIndexParams,
    index_store_dir: Option<&str>,
) -> Result<()> {
    log::info!("[TrainInvertedIndex] index_store_dir: {}", index_store_dir.unwrap_or("None"));
    // When enable_merge is false, the index_store_dir must not be None.
    if !params.enable_merge && index_store_dir.is_none() {
        panic!("index_store_dir must not be None when enable_merge is false");
    }
    let batch_stream = data_source.scan_unordered_chunks(4096).await?;
    // mapping from item to list of the row ids where it is present.
    let mut inverted_index = InvertedIndexBuilder::new(params, index_store_dir);
    inverted_index.update(batch_stream, index_store).await
}