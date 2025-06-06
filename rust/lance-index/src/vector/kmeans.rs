// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

use std::sync::Arc;

use arrow_array::{types::ArrowPrimitiveType, FixedSizeListArray, PrimitiveArray};
use lance_arrow::FixedSizeListArrayExt;
use log::info;
use snafu::location;

use lance_core::{Error, Result};
use lance_linalg::{
    distance::{DistanceType, Dot, Normalize, L2},
    kmeans::{KMeans, KMeansParams},
};

/// Train KMeans model and returns the centroids of each cluster.
///
/// Parameters
/// ----------
/// - *centroids*: initial centroids, use the random initialization if None
/// - *array*: a flatten floating number array of vectors
/// - *dimension*: dimension of the vector
/// - *k*: number of clusters
/// - *max_iterations*: maximum number of iterations
/// - *redos*: number of times to redo the k-means clustering
/// - *distance_type*: distance type to compute pair-wise vector distance
/// - *sample_rate*: sample rate to select the data for training
#[allow(clippy::too_many_arguments)]
pub fn train_kmeans<T: ArrowPrimitiveType>(
    centroids: Option<Arc<FixedSizeListArray>>,
    array: &PrimitiveArray<T>,
    dimension: usize,
    k: usize,
    max_iterations: u32,
    redos: usize,
    distance_type: DistanceType,
    sample_rate: usize,
) -> Result<KMeans>
where
    T::Native: Dot + L2 + Normalize,
    PrimitiveArray<T>: From<Vec<T::Native>>,
{
    let num_rows = array.len() / dimension;
    if num_rows < k {
        return Err(Error::Index{message: format!(
            "KMeans: can not train {k} centroids with {num_rows} vectors, choose a smaller K (< {num_rows}) instead"
        ),location: location!()});
    }

    // Only sample sample_rate * num_clusters. See Faiss
    let data = if num_rows > sample_rate * k {
        info!(
            "Sample {} out of {} to train kmeans of {} dim, {} clusters",
            sample_rate * k,
            array.len() / dimension,
            dimension,
            k,
        );
        let sample_size = sample_rate * k;
        array.slice(0, sample_size * dimension)
    } else {
        array.clone()
    };

    let params = KMeansParams::new(centroids, max_iterations, redos, distance_type);
    let data = FixedSizeListArray::try_new_from_values(data, dimension as i32)?;
    let model = KMeans::new_with_params(&data, k, &params)?;
    Ok(model)
}
