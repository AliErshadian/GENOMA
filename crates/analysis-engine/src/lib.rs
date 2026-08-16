//! Streaming analysis pipeline, similarity, statistical anomalies, and mutations.

mod anomaly;
mod mutation;
mod pipeline;
mod similarity;

pub use anomaly::{detect_anomalies, Anomaly};
pub use mutation::{detect_mutations, Mutation};
pub use pipeline::{
    analyze_reader, AnalysisResult, NoopProgress, ProgressEvent, ProgressSink, Stage,
};
pub use similarity::{compare_dna, SimilarityBreakdown, SimilarityWeights};
