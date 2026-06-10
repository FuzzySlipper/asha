//! Std-only JSON encoder for render-diff fixtures.
//!
//! The workspace has zero external dependencies, so this hand-writes the exact
//! JSON shape of the generated `render.ts` contract. It exists so a Rust test
//! can emit a fixture that the TypeScript `wasm-bridge` decoder consumes — the
//! shared, inspectable artifact at the render boundary.
//!
//! Each diff op is written on one line (compact) inside an indented frame array,
//! which keeps the committed fixture small and reviewable.

use protocol_render::{
    Geometry, Material, RenderDiff, RenderFrameDiff, RenderMetadata, RenderNode, Transform,
};

/// Encode a sequence of frames as a pretty JSON array of frame objects.
pub fn encode_sequence(frames: &[RenderFrameDiff]) -> String {
    let mut out = String::from("[\n");
    for (fi, frame) in frames.iter().enumerate() {
        out.push_str("  { \"ops\": [\n");
        for (oi, op) in frame.ops.iter().enumerate() {
            out.push_str("    ");
            encode_diff(&mut out, op);
            if oi + 1 < frame.ops.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ] }");
        if fi + 1 < frames.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("]\n");
    out
}

fn encode_diff(out: &mut String, diff: &RenderDiff) {
    match diff {
        RenderDiff::Create {
            handle,
            parent,
            node,
        } => {
            out.push_str(&format!(
                "{{ \"op\": \"create\", \"handle\": {}, \"parent\": ",
                handle.raw()
            ));
            match parent {
                Some(p) => out.push_str(&p.raw().to_string()),
                None => out.push_str("null"),
            }
            out.push_str(", \"node\": ");
            encode_node(out, node);
            out.push_str(" }");
        }
        RenderDiff::Update {
            handle,
            transform,
            material,
            visible,
            metadata,
        } => {
            out.push_str(&format!(
                "{{ \"op\": \"update\", \"handle\": {}, \"transform\": ",
                handle.raw()
            ));
            encode_opt(out, transform.as_ref(), encode_transform);
            out.push_str(", \"material\": ");
            encode_opt(out, material.as_ref(), encode_material);
            out.push_str(", \"visible\": ");
            match visible {
                Some(v) => out.push_str(if *v { "true" } else { "false" }),
                None => out.push_str("null"),
            }
            out.push_str(", \"metadata\": ");
            encode_opt(out, metadata.as_ref(), encode_metadata);
            out.push_str(" }");
        }
        RenderDiff::Destroy { handle } => {
            out.push_str(&format!(
                "{{ \"op\": \"destroy\", \"handle\": {} }}",
                handle.raw()
            ));
        }
    }
}

fn encode_node(out: &mut String, node: &RenderNode) {
    out.push_str("{ \"geometry\": ");
    encode_geometry(out, &node.geometry);
    out.push_str(", \"material\": ");
    encode_material(out, &node.material);
    out.push_str(", \"transform\": ");
    encode_transform(out, &node.transform);
    out.push_str(&format!(
        ", \"visible\": {}, \"layer\": \"{}\", \"metadata\": ",
        node.visible,
        match node.layer {
            protocol_render::RenderLayer::Scene => "scene",
            protocol_render::RenderLayer::Debug => "debug",
        }
    ));
    encode_metadata(out, &node.metadata);
    out.push_str(" }");
}

fn encode_geometry(out: &mut String, geometry: &Geometry) {
    match geometry {
        Geometry::Cube => out.push_str("{ \"shape\": \"cube\" }"),
        Geometry::Sphere => out.push_str("{ \"shape\": \"sphere\" }"),
        Geometry::Quad => out.push_str("{ \"shape\": \"quad\" }"),
        Geometry::Point => out.push_str("{ \"shape\": \"point\" }"),
        Geometry::Line { a, b } => {
            out.push_str("{ \"shape\": \"line\", \"a\": ");
            encode_f32_array(out, a);
            out.push_str(", \"b\": ");
            encode_f32_array(out, b);
            out.push_str(" }");
        }
    }
}

fn encode_material(out: &mut String, material: &Material) {
    out.push_str("{ \"color\": ");
    encode_f32_array(out, &material.color);
    out.push_str(&format!(", \"wireframe\": {} }}", material.wireframe));
}

fn encode_transform(out: &mut String, t: &Transform) {
    out.push_str("{ \"translation\": ");
    encode_f32_array(out, &t.translation);
    out.push_str(", \"rotation\": ");
    encode_f32_array(out, &t.rotation);
    out.push_str(", \"scale\": ");
    encode_f32_array(out, &t.scale);
    out.push_str(" }");
}

fn encode_metadata(out: &mut String, metadata: &RenderMetadata) {
    out.push_str("{ \"source\": ");
    match metadata.source {
        Some(id) => out.push_str(&id.raw().to_string()),
        None => out.push_str("null"),
    }
    out.push_str(", \"tags\": [");
    for (i, tag) in metadata.tags.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&tag.raw().to_string());
    }
    out.push_str("], \"label\": ");
    match &metadata.label {
        Some(label) => out.push_str(&encode_json_string(label)),
        None => out.push_str("null"),
    }
    out.push_str(" }");
}

fn encode_opt<T>(out: &mut String, value: Option<&T>, encode: fn(&mut String, &T)) {
    match value {
        Some(v) => encode(out, v),
        None => out.push_str("null"),
    }
}

fn encode_f32_array(out: &mut String, values: &[f32]) {
    out.push('[');
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("{v}"));
    }
    out.push(']');
}

fn encode_json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}
