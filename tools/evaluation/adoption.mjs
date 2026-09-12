#!/usr/bin/env node
// Exercise migration on a disposable copy, never on the supplied source notebook.
import assert from 'node:assert/strict';
import { cp, mkdtemp, readFile, readdir } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

const source = process.argv[2];
assert(source, 'Usage: node tools/evaluation/adoption.mjs <existing-notebook>');
const binary = resolve(process.env.ANB_EVAL_BINARY || 'target/debug/anb');
const scratch = await mkdtemp(join(tmpdir(), 'anb-adoption-'));
const notebook = join(scratch, '.agent-notebook');
await cp(resolve(source), notebook, { recursive: true, dereference: false });
const env = { ...process.env, ANB_BY: 'Migration evaluator' };
delete env.ANB_SESSION; delete env.CODEX_THREAD_ID; delete env.ANB_NOTEBOOK;
function run(args) {
  const result = spawnSync(binary, ['--json', '--notebook', notebook, ...args], { cwd: scratch, env, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr || result.stdout);
  return JSON.parse(result.stdout);
}
async function originals(path, prefix = '') {
  const found = new Map();
  for (const entry of await readdir(path, { withFileTypes: true })) {
    if (entry.name.startsWith('.')) continue;
    const relative = join(prefix, entry.name);
    if (entry.isDirectory()) for (const pair of await originals(join(path, entry.name), relative)) found.set(...pair);
    else if (entry.name.endsWith('.md')) found.set(relative, await readFile(join(path, entry.name), 'utf8'));
  }
  return found;
}
const before = await originals(notebook);
const records = run(['list', '--all', '--archive', '--team']).records;
const semantics = new Map(records.map(record => [record.id, run(['show', record.id, '--all'])]));
const preview = run(['migrate', '--check']);
assert.deepEqual(await originals(notebook), before, 'Preview changed record bytes');
const applied = run(['migrate']);
for (const [id, original] of semantics) {
  const actual = run(['show', id, '--all']);
  assert.deepEqual(actual, original, `Migration changed the meaning or body of ${id}`);
}
const repeated = run(['migrate']);
assert.deepEqual(repeated.paths, [], 'Repeated migration still changes records');
assert.equal(repeated.unchanged, before.size, 'Repeated migration did not account for every record');
const check = run(['check', '--all']);
assert.equal(check.count, 0, 'Migrated notebook has validation findings');
console.log(JSON.stringify({ sourceRecords: before.size, scratch, preview, applied, repeated, check, semanticReadsPreserved: semantics.size }, null, 2));
