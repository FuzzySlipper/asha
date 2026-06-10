# Architecture overview

See docs/design.md for the full design document.

Core split: Rust owns authority. TypeScript owns expression and projection.
Generated contracts define the border.

Layer order (lowest to highest): foundation → state → protocol → sim/services/rules → render/wasm/tools.
