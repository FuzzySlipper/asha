# Rust-Owned Voxel Conversion Texture Sampling

Status: design record for Den task #4596.

## Purpose

ASHA voxel conversion currently maps source material slots to voxel material ids
in Rust. It does not sample source textures or UVs. This document defines the
future authority lane for texture and UV sampling so later implementation does
not drift into Studio, renderer, Three.js, or raw image-buffer authority.

The durable path remains:

```text
generated voxel-conversion DTOs
  -> RuntimeSessionFacade / runtime bridge
  -> Rust source asset snapshot validation
  -> svc-voxel-conversion or a dedicated Rust sampler service
  -> deterministic material/sample diagnostics and voxel output
```

## Authority Rules

- Rust owns texture sampling, filtering, hashing, and diagnostics.
- Generated protocol DTOs describe source texture refs, UV attributes, sampling
  policy, and accepted outputs.
- Studio and downstream projects may author references and display diagnostics.
  They must not provide renderer-resident pixels as trusted conversion input.
- Renderer buffers, Three.js texture state, browser canvas data, or local image
  memory are projection/tooling data unless Rust has imported and hashed them as
  authority-visible assets.
- A conversion request that asks for texture sampling without authority-visible
  texture snapshots fails closed with typed diagnostics and no partial output.

## Protocol Additions

The existing generated voxel-conversion protocol should gain a small additive
family. Names are proposed, not implemented in this design slice.

```rust
pub struct VoxelConversionUvAttributeRef {
    pub attribute_name: String,
    pub source_hash: String,
}

pub struct VoxelConversionTextureSourceRef {
    pub texture_asset_id: String,
    pub asset_version: u64,
    pub content_hash: String,
    pub color_space: VoxelConversionTextureColorSpace,
    pub channel_layout: VoxelConversionTextureChannelLayout,
}

pub struct VoxelConversionTextureBinding {
    pub source_material_slot: u32,
    pub texture: VoxelConversionTextureSourceRef,
    pub uv_attribute: VoxelConversionUvAttributeRef,
    pub sampling: VoxelConversionTextureSamplingPolicy,
}

pub struct VoxelConversionTextureMaterialRule {
    pub source_material_slot: u32,
    pub mode: VoxelConversionTextureMaterialMode,
    pub output_palette: Option<VoxelConversionTexturePaletteRef>,
}
```

Suggested stable vocabularies:

- `VoxelConversionTextureColorSpace`: `linear`, `srgb`.
- `VoxelConversionTextureChannelLayout`: `rgba8`, `rgb8`, `grayscale8`.
- `VoxelConversionTextureSamplingPolicy`: `nearest_texel`,
  `nearest_mipmap_level0`, `bilinear_level0`.
- `VoxelConversionTextureMaterialMode`: `sample_palette_index`,
  `sample_luminance_threshold`, `sample_average_color_to_palette`.

The first implementation should prefer `nearest_texel` and a palette/index mode.
More visual sampling modes can be added later without changing the authority
boundary.

## Source Snapshot Model

Texture input must be represented as an authority-visible source snapshot:

- texture asset id and version;
- content hash of decoded canonical pixels, not only source file bytes;
- canonical dimensions and channel layout;
- color-space interpretation;
- optional import settings hash when decoding/transcoding affects samples;
- UV attribute hash for the source mesh primitive.

The source snapshot can be produced by an asset-import lane, a dedicated Rust
texture-snapshot service, or the voxel conversion service itself. In every case,
the conversion request references hashes that Rust can validate before sampling.

If a downstream tool has only a browser/renderer texture, it must first route
that texture through the same import/snapshot process. It cannot hand pixels to
conversion as trusted runtime memory.

## Deterministic Sampling Rules

Initial texture sampling should be intentionally boring:

- UVs are read from a named source mesh UV attribute.
- Triangle sample UV is derived in Rust from the same primitive used for
  conversion, using deterministic barycentric or representative-cell sampling.
- UV wrapping is explicit per binding. The initial allowed value should be
  `clamp_to_edge`; repeat/mirror can be added later.
- Filtering is explicit per binding. The initial implementation should support
  `nearest_texel`; bilinear support must define exact rounding and color-space
  conversion before acceptance.
- Palette or threshold mapping produces ASHA voxel material ids. Raw sampled
  colors are evidence/projection only unless a generated DTO explicitly defines
  them as authority output.
- All floating-point decisions that affect material assignment must be covered
  by deterministic tests and stable summary hashes.

## Diagnostics

Add diagnostic codes only through the generated protocol vocabulary. Suggested
codes:

- `texture_sampling_unavailable`: backend does not support texture sampling.
- `missing_texture_source`: a requested texture ref is not authority-visible.
- `texture_hash_mismatch`: request hash does not match the validated snapshot.
- `missing_uv_attribute`: source mesh lacks the named UV attribute.
- `uv_hash_mismatch`: UV attribute hash does not match the validated snapshot.
- `unsupported_texture_format`: layout/color-space is not supported.
- `unsupported_sampling_policy`: requested sampling/filter/wrap is unsupported.
- `invalid_texture_material_rule`: palette/threshold/material output mapping is
  malformed or incomplete.

Unsupported texture sampling is an error for requests that require it, not a
best-effort fallback to source material slots. Callers can choose material-slot
mapping explicitly when they want that fallback behavior.

## Evidence And Hashes

Plans, previews, receipts, model-info readouts, and exported evidence should
include texture sampling facts when texture sampling participates in output:

- texture snapshot refs and content hashes;
- UV attribute refs and hashes;
- sampling policy and material rule hashes;
- output material counts;
- stable sample summary hash;
- diagnostics for skipped or rejected texture bindings.

These facts are readouts. They do not grant Studio or TypeScript mutation rights.

## Implementation Path

1. Extend `protocol-voxel-conversion` with texture source, UV, sampling policy,
   material-rule, and diagnostic DTOs.
2. Regenerate `@asha/contracts` through `protocol-codegen` and add Rust
   round-trip coverage for every Rust-mirrored DTO.
3. Add a Rust texture snapshot/input model in `svc-voxel-conversion` or a
   dedicated service that `svc-voxel-conversion` calls.
4. Validate snapshot hashes, UV hashes, texture formats, and sampling policy
   before output generation.
5. Add one tiny deterministic fixture: a textured quad with two UV regions that
   map to two voxel material ids.
6. Add negative fixtures for missing texture, hash mismatch, missing UV, and
   unsupported format/policy.
7. Surface readout/evidence through existing RuntimeSession conversion methods
   before adding any Studio UI.

## Non-Claims

- This document does not implement texture sampling.
- It does not permit Studio or renderer-owned texture data to become authority.
- It does not define atlas packing, material authoring UI, complex PBR material
  evaluation, mip generation, or GPU-assisted sampling.
- It does not require changing current material-slot mapping behavior.
