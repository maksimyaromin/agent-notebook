#!/usr/bin/env node
// Measure real CLI reads over imported synthetic records, without changing the working notebook.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, readFile, readdir, writeFile } from 'node:fs/promises';
import { cpus, platform, release } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { performance } from 'node:perf_hooks';
import { fileURLToPath } from 'node:url';

const repository = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const binary = resolve(process.env.ANB_EVAL_BINARY || join(repository, 'target/release/anb'));
const repetitions = Number(process.env.ANB_PERF_REPETITIONS || 15);
assert(Number.isSafeInteger(repetitions) && repetitions >= 5, 'Use at least five repetitions.');
const sizes = [100, 1000, 5000];
const commands = ['status', 'recall', 'ready'];
const fixtureDate = new Date().toISOString().slice(0, 10);
const root = join(repository, '.tmp/evaluation');
await mkdir(root, { recursive: true });
const output = await mkdtemp(join(root, 'performance-'));
const binaryHash = createHash('sha256').update(await readFile(binary)).digest('hex');
const samples = [];

function invoke(program, arguments_, fixture, timeout = 30000) {
  const started = performance.now();
  const result = spawnSync(program, arguments_, {
    cwd: fixture.project, env: fixture.env, encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024, timeout,
  });
  const wallMs = performance.now() - started;
  assert(!result.error, program + ': ' + result.error?.message);
  assert.equal(result.status, 0, arguments_.join(' ') + ': ' + result.stderr);
  return { wallMs, stdout: result.stdout };
}

// Match the public estimator in anb-core/src/tokens.rs, including the output newline.
function estimateTokens(text) {
  let ascii = 0;
  const bytes = Buffer.from(text);
  for (const byte of bytes) if (byte < 128) ascii++;
  return Math.ceil((ascii * 6 + (bytes.length - ascii) * 7) / 21);
}

async function fingerprint(directory) {
  const entries = [];
  async function walk(current) {
    for (const entry of await readdir(current, { withFileTypes: true })) {
      const path = join(current, entry.name);
      if (entry.isDirectory()) await walk(path);
      else entries.push([path.slice(directory.length), createHash('sha256').update(await readFile(path)).digest('hex')]);
    }
  }
  await walk(directory);
  return createHash('sha256').update(JSON.stringify(entries.sort())).digest('hex');
}

const number = index => String(index).padStart(6, '0');
function record(index) {
  const slot = index % 10;
  const type = slot === 0 ? 'decision' : slot < 4 ? 'note' : 'task';
  const id = type + '.scale-' + number(index);
  const active = type === 'task' && slot === 4;
  const lines = [
    '---', 'id: ' + id, 'type: ' + type,
    'state: ' + (type === 'task' ? (active ? 'active' : 'open') : 'active'),
    'title: Synthetic ' + type + ' ' + number(index),
    'by: ' + (type === 'task' ? 'Evaluator' : 'Contributor'), 'tags: scale, context',
  ];
  if (type === 'task') {
    lines.push('taken-by: Evaluator');
    if (slot === 6) lines.push('blocked-by: task.scale-' + number(index - 1));
  } else {
    lines.push('kind: ' + (type === 'decision' ? 'rule' : 'fact'));
    lines.push('from: task.scale-' + number(index - slot + 4));
    lines.push('link: doc https://example.test/synthetic-source');
  }
  lines.push('created: ' + fixtureDate, 'updated: ' + fixtureDate, '---', '');
  lines.push('Synthetic context explains the current constraint, its source and the next useful action. '.repeat(24));
  if (active) lines.push('- ' + fixtureDate + ' Evaluator: Verified the current result. Next: inspect the remaining edge case.');
  return { path: type + 's/' + id + '.md', text: lines.join('\n') + '\n' };
}

async function prepare(size) {
  const directory = join(output, 'records-' + size);
  const project = join(directory, 'project');
  const privateHome = join(directory, 'person');
  const incoming = join(directory, 'incoming');
  await mkdir(directory, { recursive: true });
  await Promise.all([mkdir(project), mkdir(privateHome), mkdir(incoming)]);
  const env = { ...process.env, HOME: privateHome, ANB_BY: 'Evaluator' };
  for (const key of Object.keys(env)) {
    if ((key.startsWith('ANB_') && key !== 'ANB_BY') || key === 'CODEX_THREAD_ID' || key.startsWith('GIT_')) delete env[key];
  }
  env.GIT_CONFIG_NOSYSTEM = '1';
  const fixture = { project, env, notebook: join(project, '.agent-notebook') };
  invoke('git', ['init', '--quiet', project], fixture);
  await Promise.all(['tasks', 'notes', 'decisions', 'questions'].map(type => mkdir(join(incoming, type))));
  for (let index = 0; index < size; index++) {
    const file = record(index);
    await writeFile(join(incoming, file.path), file.text);
  }
  const imported = invoke(binary, ['--notebook', fixture.notebook, 'import', incoming, '--json'], fixture, 300000);
  assert.equal(JSON.parse(imported.stdout).paths.length, size);
  return { ...fixture, importMs: imported.wallMs };
}

function omissions(value, at = '') {
  if (!value || typeof value !== 'object') return {};
  const result = {};
  for (const [key, child] of Object.entries(value)) {
    const location = at + '/' + key;
    if (key === 'omitted' || key.endsWith('-omitted')) result[location] = child;
    else Object.assign(result, omissions(child, location));
  }
  return result;
}

function checkReply(command, value, toon, size) {
  const bytes = Buffer.byteLength(toon), tokens = estimateTokens(toon);
  assert(bytes <= 8192, command + ': default output grew past 8 KiB (' + bytes + ').');
  if (command === 'ready') {
    assert.equal(value.count, size * 0.4, 'The full dependency graph determines readiness.');
    assert.equal(value.ready.length, 20);
    assert.equal(value.omitted, value.count - value.ready.length);
    assert(value.more, 'Omitted ready work needs a follow-up read.');
  } else {
    assert.equal(value.budget.limit, 1500);
    assert.equal(value.budget.spent, tokens, 'TOON measurement matches the reported estimate.');
    assert(tokens <= 1500, command + ': the ordinary synthetic reply exceeded its default budget.');
    const work = command === 'recall' ? value.work : value;
    assert.equal(work.counts.tasks, size * 0.6);
    assert.equal(work.active.count, size * 0.1);
    assert.equal(work.ready.count, size * 0.4);
    for (const name of ['active', 'ready']) assert.equal(work[name].omitted, work[name].count - work[name].rows.length);
    if (command === 'recall') {
      assert.equal(value.count, size * 0.4);
      assert.equal(value.omitted, value.count - value.memories.length);
      assert(value.more, 'Omitted knowledge needs a follow-up read.');
    }
  }
  return { toonBytes: bytes, estimatedTokens: tokens, omissions: omissions(value) };
}

function percentile(values, fraction) {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.max(0, Math.ceil(sorted.length * fraction) - 1)];
}

for (const size of sizes) {
  process.stdout.write('Preparing ' + size + ' synthetic records...\n');
  const fixture = await prepare(size);
  const before = await fingerprint(fixture.notebook);
  const documents = Object.fromEntries(commands.map(command => [command, JSON.parse(invoke(binary, [command, '--json'], fixture).stdout)]));
  for (let warmup = 0; warmup < 2; warmup++) {
    for (const command of commands) invoke(binary, [command], fixture);
  }
  const measured = Object.fromEntries(commands.map(command => [command, []]));
  const outputMetrics = {};
  for (let iteration = 0; iteration < repetitions; iteration++) {
    for (const command of commands) {
      const result = invoke(binary, [command], fixture);
      const metrics = checkReply(command, documents[command], result.stdout, size);
      if (outputMetrics[command]) assert.deepEqual(metrics, outputMetrics[command]);
      outputMetrics[command] = metrics;
      measured[command].push(result.wallMs);
      if (iteration === 0) await writeFile(join(output, size + '-' + command + '.toon'), result.stdout);
    }
  }
  assert.equal(await fingerprint(fixture.notebook), before, 'Measured reads must preserve every notebook file.');
  for (const command of commands) {
    const result = {
      records: size, command, repetitions, importMs: fixture.importMs,
      medianMs: percentile(measured[command], 0.5), p95Ms: percentile(measured[command], 0.95),
      samplesMs: measured[command], ...outputMetrics[command],
    };
    samples.push(result);
    process.stdout.write(size + ' ' + command + ': median ' + result.medianMs.toFixed(1) + ' ms; p95 ' + result.p95Ms.toFixed(1) + ' ms; ' + result.toonBytes + ' bytes\n');
  }
}

const scaling = [];
for (const command of commands) {
  const rows = samples.filter(sample => sample.command === command);
  for (let index = 1; index < rows.length; index++) {
    const before = rows[index - 1], after = rows[index];
    const recordsRatio = after.records / before.records, medianRatio = after.medianMs / before.medianMs;
    scaling.push({
      command, from: before.records, to: after.records, recordsRatio, medianRatio,
      warning: medianRatio > 2 * recordsRatio ? 'Growth exceeds twice the linear ratio; investigate repeated materialization or measurement noise.' : null,
    });
  }
}
const results = {
  measuredAt: new Date().toISOString(), fixtureDate,
  machine: { platform: platform(), release: release(), cpu: cpus()[0]?.model, node: process.version },
  binary: { sha256: binaryHash, version: invoke(binary, ['--version'], { project: repository, env: process.env }).stdout.trim() },
  method: {
    sizes, repetitions, warmups: 2, samples: 'Fresh processes, interleaved commands, warm filesystem cache; startup and stdout capture included.',
    percentile: 'Nearest rank; p95 is a sample estimate, not a service-level guarantee.',
    corpus: '60% Tasks (10% active, 10% blocked, 40% ready), 30% Notes, 10% Decisions; about 2 KiB of prose per record.',
    limitations: 'Machine load, filesystem caching and process startup affect timings. A scaling warning is diagnostic, not proof of asymptotic complexity. This runner does not measure peak memory or cold-cache latency.',
  },
  samples, scaling,
};
await writeFile(join(output, 'results.json'), JSON.stringify(results, null, 2) + '\n');
process.stdout.write('Results: ' + join(output, 'results.json') + '\n');
for (const check of scaling) if (check.warning) process.stdout.write('Warning: ' + check.command + ' ' + check.from + ' to ' + check.to + ': ' + check.warning + '\n');
