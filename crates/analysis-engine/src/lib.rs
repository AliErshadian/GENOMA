//! Streaming analysis pipeline, similarity, anomalies, mutations, clustering, and embedding.

mod anomaly;
mod cluster;
mod embed;
mod mutation;
mod pipeline;
mod similarity;

pub use anomaly::{detect_anomalies, Anomaly};
pub use cluster::{cluster_files, CLUSTER_DISTANCE_CUT};
pub use embed::{embed_files, GalaxyLink, GALAXY_LINK_SIMILARITY};
pub use mutation::{detect_mutations, Mutation};
pub use pipeline::{
    analyze_reader, AnalysisResult, NoopProgress, ProgressEvent, ProgressSink, Stage,
};
pub use similarity::{compare_dna, SimilarityBreakdown, SimilarityWeights};
