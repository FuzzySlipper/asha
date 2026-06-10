# Boundary rules

1. TypeScript may never mutate authoritative state.
2. Policy code receives generated read-only views; it returns proposed commands only.
3. Rust validates all commands. TypeScript does not validate.
4. Generated contract files in ts/packages/contracts/src/generated/ are not hand-edited.
5. No lower-level Rust crate may depend on a higher-level crate.
6. Policy/catalog packages may not import renderer, UI, WASM bridge, or Electron packages.
7. Renderer packages may not import policy packages.
8. Tool omniscience must not leak into runtime packages.
