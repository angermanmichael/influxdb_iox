//! Config-related stuff.
use std::{num::NonZeroUsize, sync::Arc};

use backoff::{Backoff, BackoffConfig};
use data_types::{ShardId, ShardIndex};
use iox_catalog::interface::Catalog;
use iox_query::exec::Executor;
use iox_time::TimeProvider;
use parquet_file::storage::ParquetStorage;

/// Config to set up a compactor.
#[derive(Debug, Clone)]
pub struct Config {
    /// Shard Id
    pub shard_id: ShardId,

    /// Metric registry.
    pub metric_registry: Arc<metric::Registry>,

    /// Central catalog.
    pub catalog: Arc<dyn Catalog>,

    /// Store holding the parquet files.
    pub parquet_store: ParquetStorage,

    /// Executor.
    pub exec: Arc<Executor>,

    /// Time provider.
    pub time_provider: Arc<dyn TimeProvider>,

    /// Backoff config
    pub backoff_config: BackoffConfig,

    /// Number of partitions that should be compacted in parallel.
    ///
    /// This should usually be larger than the compaction job concurrency since one partition can spawn multiple compaction jobs.
    pub partition_concurrency: NonZeroUsize,

    /// Number of concurrent compaction jobs.
    ///
    /// This should usually be smaller than the partition concurrency since one partition can spawn multiple compaction jobs.
    pub job_concurrency: NonZeroUsize,

    /// Partitions with recent created files these last minutes are selected for compaction.
    pub partition_minute_threshold: u64,

    /// Desired max size of compacted parquet files
    /// It is a target desired value than a guarantee
    pub max_desired_file_size_bytes: u64,

    /// Percentage of desired max file size.
    /// If the estimated compacted result is too small, no need to split it.
    /// This percentage is to determine how small it is:
    ///    < percentage_max_file_size * max_desired_file_size_bytes:
    /// This value must be between (0, 100)
    pub percentage_max_file_size: u16,

    /// Split file percentage
    /// If the estimated compacted result is neither too small nor too large, it will be split
    /// into 2 files determined by this percentage.
    ///    . Too small means: < percentage_max_file_size * max_desired_file_size_bytes
    ///    . Too large means: > max_desired_file_size_bytes
    ///    . Any size in the middle will be considered neither too small nor too large
    /// This value must be between (0, 100)
    pub split_percentage: u16,

    /// Maximum duration of the per-partition compaction task in seconds.
    pub partition_timeout_secs: u64,
}

impl Config {
    /// Fetch shard ID.
    ///
    /// This is likely required to construct a [`Config`] object.
    pub async fn fetch_shard_id(
        catalog: Arc<dyn Catalog>,
        backoff_config: BackoffConfig,
        topic_name: String,
        shard_index: i32,
    ) -> ShardId {
        // Get shardId from topic and shard_index
        // Fetch topic
        let topic = Backoff::new(&backoff_config)
            .retry_all_errors("topic_of_given_name", || async {
                catalog
                    .repositories()
                    .await
                    .topics()
                    .get_by_name(topic_name.as_str())
                    .await
            })
            .await
            .expect("retry forever");

        if topic.is_none() {
            panic!("Topic {} not found", topic_name);
        }
        let topic = topic.unwrap();

        // Fetch shard
        let shard = Backoff::new(&backoff_config)
            .retry_all_errors("sahrd_of_given_index", || async {
                catalog
                    .repositories()
                    .await
                    .shards()
                    .get_by_topic_id_and_shard_index(topic.id, ShardIndex::new(shard_index))
                    .await
            })
            .await
            .expect("retry forever");

        match shard {
            Some(shard) => shard.id,
            None => {
                panic!(
                    "Topic {} and Shard Index {} not found",
                    topic_name, shard_index
                )
            }
        }
    }
}