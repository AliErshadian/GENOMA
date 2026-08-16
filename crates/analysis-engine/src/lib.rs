//! Streaming analysis pipeline, similarity, anomalies, mutations, clustering, embedding, and ML experiments.

mod anomaly;
mod cluster;
mod embed;
mod ml_experiments;
mod mutation;
mod pipeline;
mod similarity;

pub use anomaly::{detect_anomalies, Anomaly, ANOMALY_METHOD};
pub use cluster::{cluster_files, CLUSTER_DISTANCE_CUT};
pub use embed::{embed_files, GalaxyLink, GALAXY_LINK_SIMILARITY};
pub use ml_experiments::{isolation_score, knn_density, ExperimentResult, ExperimentScore};
pub use mutation::{detect_mutations, Mutation};
pub use pipeline::{
    analyze_reader, AnalysisResult, NoopProgress, ProgressEvent, ProgressSink, Stage,
};
pub use similarity::{compare_dna, SimilarityBreakdown, SimilarityWeights};
