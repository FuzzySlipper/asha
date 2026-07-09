#!/usr/bin/env node

const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '../../..');
const fixtureDir = path.join(repoRoot, 'harness/fixtures/mesh-animation');
const manifestPath = path.join(fixtureDir, 'kenney-retro-character-medium.manifest.json');
const glbPath = path.join(fixtureDir, 'kenney-retro-character-medium.glb');

function fail(message) {
  console.error(`FAIL: ${message}`);
  process.exit(1);
}

function readGlb(filePath) {
  const bytes = fs.readFileSync(filePath);
  if (bytes.toString('utf8', 0, 4) !== 'glTF') {
    fail(`${filePath} is not a GLB file`);
  }
  const version = bytes.readUInt32LE(4);
  if (version !== 2) {
    fail(`${filePath} is GLB version ${version}, expected 2`);
  }
  const declaredLength = bytes.readUInt32LE(8);
  if (declaredLength !== bytes.length) {
    fail(`${filePath} declares ${declaredLength} bytes but contains ${bytes.length}`);
  }
  const jsonLength = bytes.readUInt32LE(12);
  const jsonType = bytes.toString('utf8', 16, 20);
  if (jsonType !== 'JSON') {
    fail(`${filePath} first chunk is ${jsonType}, expected JSON`);
  }
  const json = JSON.parse(bytes.toString('utf8', 20, 20 + jsonLength).trim());
  return { bytes, json };
}

function sha256(bytes) {
  return crypto.createHash('sha256').update(bytes).digest('hex');
}

function animationDurationSeconds(gltf, animation) {
  let duration = 0;
  for (const sampler of animation.samplers || []) {
    const input = gltf.accessors?.[sampler.input];
    if (input?.max?.[0] > duration) {
      duration = input.max[0];
    }
  }
  return duration;
}

function vec3Bounds(gltf) {
  const min = [Infinity, Infinity, Infinity];
  const max = [-Infinity, -Infinity, -Infinity];
  for (const accessor of gltf.accessors || []) {
    if (accessor.type === 'VEC3' && Array.isArray(accessor.min) && Array.isArray(accessor.max)) {
      for (let i = 0; i < 3; i += 1) {
        min[i] = Math.min(min[i], accessor.min[i]);
        max[i] = Math.max(max[i], accessor.max[i]);
      }
    }
  }
  return { min, max };
}

const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
const { bytes, json } = readGlb(glbPath);
const contentHash = sha256(bytes);
if (manifest.contentHashSha256 !== contentHash) {
  fail(`manifest contentHashSha256 is ${manifest.contentHashSha256}, actual is ${contentHash}`);
}

const clipSummaries = (json.animations || []).map((animation) => ({
  id: animation.name,
  durationSeconds: animationDurationSeconds(json, animation),
  channelCount: animation.channels?.length || 0,
  samplerCount: animation.samplers?.length || 0,
}));
const clipIds = clipSummaries.map((clip) => clip.id);
for (const expected of manifest.clipIds || []) {
  if (!clipIds.includes(expected)) {
    fail(`missing expected clip ${expected}`);
  }
}
if (manifest.primaryProofClipId && !clipIds.includes(manifest.primaryProofClipId)) {
  fail(`missing primary proof clip ${manifest.primaryProofClipId}`);
}
if (clipIds[0] === manifest.primaryProofClipId) {
  fail(`primary proof clip ${manifest.primaryProofClipId} must not be the first/default clip`);
}

const externalUris = ['buffers', 'images']
  .flatMap((key) => (json[key] || []).map((entry) => entry.uri).filter(Boolean));
if (externalUris.length > 0) {
  fail(`fixture contains external URIs: ${externalUris.join(', ')}`);
}
if ((json.meshes || []).length < 1) {
  fail('fixture has no mesh');
}
if ((json.skins || []).length < 1) {
  fail('fixture has no skin');
}

const summary = {
  assetId: manifest.assetId,
  runtimeFormat: manifest.runtimeFormat,
  contentHashSha256: contentHash,
  byteLength: bytes.length,
  meshCount: json.meshes?.length || 0,
  skinCount: json.skins?.length || 0,
  nodeCount: json.nodes?.length || 0,
  imageCount: json.images?.length || 0,
  materialCount: json.materials?.length || 0,
  boundsHint: vec3Bounds(json),
  clips: clipSummaries,
  primaryProofClipId: manifest.primaryProofClipId,
};

console.log(JSON.stringify(summary, null, 2));
