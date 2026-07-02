#!/usr/bin/env node
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const VALID_TYPES = new Set(['lib', 'shell', 'testing', 'tool']);
const VALID_LAYERS = new Set([
  'protocol',
  'transport',
  'domain',
  'renderer',
  'components',
  'shell',
  'testing-fixtures',
  'tool',
]);

function usage() {
  return `Usage:
  node ts/scripts/new-package.mjs <name> --lane <lane> --type <type> --layer <layer> [options]

Options:
  --repo-root <path>       Repository root. Defaults to this script's ASHA repo.
  --may-import <list>      Comma-separated @asha/* root packages this package may import.
  --may-not-import <list>  Comma-separated @asha/* root packages this package may not import.

Examples:
  node ts/scripts/new-package.mjs diagnostics-view --lane ts-tools --type tool --layer tool --may-import @asha/contracts
`;
}

function parseArgs(argv) {
  const positional = [];
  const options = new Map();
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token?.startsWith('--')) {
      const value = argv[index + 1];
      if (!value || value.startsWith('--')) {
        throw new Error(`missing value for ${token}`);
      }
      options.set(token.slice(2), value);
      index += 1;
      continue;
    }
    if (token) positional.push(token);
  }
  if (positional.length !== 1) {
    throw new Error('expected exactly one package name');
  }
  return { name: positional[0], options };
}

function parseList(value) {
  if (!value) return [];
  return value.split(',').map(item => item.trim()).filter(Boolean);
}

function normalizePackageName(input) {
  const rawName = input.startsWith('@asha/') ? input.slice('@asha/'.length) : input;
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(rawName)) {
    throw new Error(`invalid package name "${input}"; use scoped-kebab-case such as @asha/example-tool`);
  }
  return { shortName: rawName, packageName: `@asha/${rawName}` };
}

function requireOption(options, key) {
  const value = options.get(key);
  if (!value) throw new Error(`missing required --${key}`);
  return value;
}

function assertPackageList(label, packages) {
  for (const packageName of packages) {
    if (!/^@asha\/[a-z0-9]+(?:-[a-z0-9]+)*$/.test(packageName)) {
      throw new Error(`${label} contains invalid ASHA package root "${packageName}"`);
    }
  }
}

function packageDirName(packageName) {
  return packageName.slice('@asha/'.length);
}

function renderJson(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function renderArray(values) {
  return `[${values.map(value => JSON.stringify(value)).join(', ')}]`;
}

function exportedFunctionName(shortName) {
  return `describe${shortName.split('-').map(part => `${part[0].toUpperCase()}${part.slice(1)}`).join('')}Package`;
}

function renderIndex(packageName, shortName) {
  const functionName = exportedFunctionName(shortName);
  return `// ${packageName} - scaffolded ASHA TypeScript package.
//
// Replace this placeholder surface with the narrow public API approved for this
// package. Keep sibling ASHA imports routed through package root barrels only.

export const ashaPackageName = '${packageName}';

export interface AshaPackageScaffold {
  readonly name: string;
}

export function ${functionName}(): AshaPackageScaffold {
  return { name: ashaPackageName };
}
`;
}

function renderTest(packageName, shortName) {
  const functionName = exportedFunctionName(shortName);
  return `import { test } from 'node:test';
import assert from 'node:assert/strict';

import { ashaPackageName, ${functionName} } from './index.js';

test('${packageName} scaffold exports its package identity', () => {
  assert.equal(ashaPackageName, '${packageName}');
  assert.deepEqual(${functionName}(), { name: '${packageName}' });
});
`;
}

function renderTsconfig(mayImport) {
  const references = mayImport.map(packageName => ({ path: `../${packageDirName(packageName)}` }));
  return renderJson({
    extends: '../../tsconfig.base.json',
    compilerOptions: {
      outDir: './dist',
      rootDir: './src',
    },
    include: ['src'],
    references,
  });
}

function renderPackageJson(packageName, mayImport) {
  const dependencies = Object.fromEntries(mayImport.map(packageName => [packageName, 'workspace:*']));
  const packageJson = {
    name: packageName,
    version: '0.1.0',
    private: true,
    type: 'module',
    main: './dist/index.js',
    types: './dist/index.d.ts',
    exports: {
      '.': {
        import: './dist/index.js',
        types: './dist/index.d.ts',
      },
    },
    scripts: {
      build: 'tsc --build',
      typecheck: 'tsc --build',
      test: "tsc --build && node --test 'dist/**/*.test.js'",
    },
  };
  if (Object.keys(dependencies).length > 0) {
    packageJson.dependencies = dependencies;
  }
  return renderJson(packageJson);
}

function renderOwnershipBlock({ shortName, lane, type, layer, mayImport, mayNotImport }) {
  return [
    '',
    `# Scaffolded by ts/scripts/new-package.mjs. Replace this comment with package-specific boundary notes before widening imports.`,
    `[package."ts/packages/${shortName}"]`,
    `lane = ${JSON.stringify(lane)}`,
    `type = ${JSON.stringify(type)}`,
    `layer = ${JSON.stringify(layer)}`,
    `may_import = ${renderArray(mayImport)}`,
    `may_not_import = ${renderArray(mayNotImport)}`,
    '',
  ].join('\n');
}

function insertOwnershipBlock(existingText, block) {
  const marker = '# Rust: bridge layer';
  const markerIndex = existingText.indexOf(marker);
  if (markerIndex === -1) {
    return `${existingText.trimEnd()}\n${block}`;
  }
  const before = existingText.slice(0, markerIndex).trimEnd();
  const after = existingText.slice(markerIndex);
  return `${before}\n${block}\n${after}`;
}

async function main() {
  const scriptRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
  const { name, options } = parseArgs(process.argv.slice(2));
  const repoRoot = resolve(options.get('repo-root') ?? scriptRoot);
  const { shortName, packageName } = normalizePackageName(name);
  const lane = requireOption(options, 'lane');
  const type = requireOption(options, 'type');
  const layer = requireOption(options, 'layer');
  const mayImport = parseList(options.get('may-import'));
  const mayNotImport = parseList(options.get('may-not-import'));

  if (!VALID_TYPES.has(type)) {
    throw new Error(`invalid --type "${type}"; expected one of ${[...VALID_TYPES].sort().join(', ')}`);
  }
  if (!VALID_LAYERS.has(layer)) {
    throw new Error(`invalid --layer "${layer}"; expected one of ${[...VALID_LAYERS].sort().join(', ')}`);
  }
  assertPackageList('--may-import', mayImport);
  assertPackageList('--may-not-import', mayNotImport);

  const packageDir = resolve(repoRoot, 'ts', 'packages', shortName);
  const ownershipPath = resolve(repoRoot, 'governance', 'ownership.toml');
  const ownershipText = await readFile(ownershipPath, 'utf8');
  const ownershipKey = `[package."ts/packages/${shortName}"]`;

  if (existsSync(packageDir)) {
    throw new Error(`refusing to overwrite existing package directory: ${packageDir}`);
  }
  if (ownershipText.includes(ownershipKey)) {
    throw new Error(`refusing to overwrite existing ownership entry: ${ownershipKey}`);
  }
  for (const packageName of mayImport) {
    const dependencyDir = resolve(repoRoot, 'ts', 'packages', packageDirName(packageName));
    if (!existsSync(resolve(dependencyDir, 'package.json'))) {
      throw new Error(`--may-import package does not exist in this workspace: ${packageName}`);
    }
  }

  await mkdir(resolve(packageDir, 'src'), { recursive: true });
  await writeFile(resolve(packageDir, 'package.json'), renderPackageJson(packageName, mayImport));
  await writeFile(resolve(packageDir, 'tsconfig.json'), renderTsconfig(mayImport));
  await writeFile(resolve(packageDir, 'src', 'index.ts'), renderIndex(packageName, shortName));
  await writeFile(resolve(packageDir, 'src', 'index.test.ts'), renderTest(packageName, shortName));

  const block = renderOwnershipBlock({ shortName, lane, type, layer, mayImport, mayNotImport });
  await writeFile(ownershipPath, insertOwnershipBlock(ownershipText, block));

  console.log(`created ${packageName}`);
  console.log(`  package: ts/packages/${shortName}`);
  console.log(`  ownership: governance/ownership.toml`);
  console.log('next: run `bash harness/ci/check-depgraph.sh` and `pnpm --dir ts --filter ' + packageName + ' test`');
}

main().catch(error => {
  console.error(`new-package failed: ${error instanceof Error ? error.message : String(error)}`);
  console.error(usage());
  process.exit(1);
});
