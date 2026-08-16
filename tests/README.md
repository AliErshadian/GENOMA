Crate tests are the source of truth for Phase 1:

- `crates/genoma-core` — streaming chunker, including a 10 MB bounded-buffer test
- `crates/pi-engine` — prefix, range, wrap, bundled 100k-digit dataset
- `crates/feature-engine` — entropy/repetition goldens, analysis levels
- `crates/dna-engine` — Givens transform stability
- `crates/analysis-engine/tests/determinism.rs` — three-run identical DNA JSON, similarity/mutation monotonicity
- `apps/web/lib/mappings.test.ts` — visual mapping purity

```bash
cargo test --workspace
pnpm --filter @genoma/web test
```
