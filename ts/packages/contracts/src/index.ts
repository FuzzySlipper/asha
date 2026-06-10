// Public surface of @asha/contracts.
//
// Every contract type and branded-ID constructor exported here is generated
// from the Rust protocol crates by `protocol-codegen` and re-exported
// unchanged. Do not hand-write contract types in this package: change the Rust
// source under engine-rs/crates/protocol/* and regenerate
// (`cargo run -p protocol-codegen`).
//
// The generated barrel (./generated/index.js) is the single, stable entry point
// for branded IDs (ids.ts), script views/commands/rejections (script.ts),
// retained-mode render diffs (render.ts), and replay records (replay.ts).
export * from './generated/index.js';
