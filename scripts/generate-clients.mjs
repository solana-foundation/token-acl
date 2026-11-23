#!/usr/bin/env zx
import 'zx/globals';
import { createFromRoot } from 'codama';
import { renderVisitor as renderJavaScriptVisitor } from '@codama/renderers-js';
import { renderVisitor as renderRustVisitor } from "@codama/renderers-rust";
import { workingDirectory } from './utils.mjs';

const idl = require(path.join(workingDirectory, 'program', 'idl.json'));

// Instanciate Codama.
const codama = createFromRoot(idl);

// Render JavaScript.
const jsClient = path.join(__dirname, '..', 'clients', 'js');
codama.accept(
  renderJavaScriptVisitor(path.join(jsClient, 'src', 'generated'), {
    prettier: require(path.join(jsClient, '.prettierrc.json')),
  })
);

// Render Rust.
//
// The permissionless thaw/freeze instructions take an optional trailing `data`
// argument (remainder-sized bytes forwarded to the gating program). The Codama
// Rust renderer maps remainder bytes to `kaigan::types::RemainderVec<u8>` (kaigan
// is not a dependency of this crate) and renders the empty default as an
// uncompilable `unwrap_or([])`. It would also change the signature of the
// generated `instruction()` / `Cpi::new` methods. To keep the Rust client source compatible, the argument is omitted
// from the Rust render and exposed instead through hand-written helpers in
// `clients/rust/src/permissionless_data.rs` (`instruction_with_data`).
const PERMISSIONLESS_DATA_IXS = new Set([
  'thawPermissionless',
  'thawPermissionlessIdempotent',
  'freezePermissionless',
  'freezePermissionlessIdempotent',
]);
const rustIdl = structuredClone(idl);
for (const ix of rustIdl.program.instructions) {
  if (PERMISSIONLESS_DATA_IXS.has(ix.name)) {
    ix.arguments = ix.arguments.filter((arg) => arg.name !== 'data');
  }
}

const rustClient = path.join(__dirname, "..", "clients", "rust");
createFromRoot(rustIdl).accept(
  renderRustVisitor(path.join(rustClient, "src", "generated"), {
    formatCode: true,
    crateFolder: rustClient,
  })
);
