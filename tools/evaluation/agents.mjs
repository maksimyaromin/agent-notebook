#!/usr/bin/env node
// Isolated, observable agent trials. Expectations are specified before a model runs.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, mkdir, readFile, readdir, realpath, symlink, writeFile } from 'node:fs/promises';
import { homedir, tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn, spawnSync } from 'node:child_process';

const repository = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const binary = process.env.ANB_EVAL_BINARY || join(repository, 'target/debug/anb');
const models = (process.env.ANB_EVAL_MODELS || 'gpt-5.6-luna,gpt-6-astra').split(',');
const scenarios = (process.env.ANB_EVAL_SCENARIOS || 'continuation,intents,plain-reader,bounded-read').split(',');
const output = process.env.ANB_EVAL_OUTPUT || await mkdtemp(join(tmpdir(), 'anb-agent-evidence-'));
await mkdir(output, { recursive: true });
const codeHome = process.env.CODEX_HOME || join(homedir(), '.codex');

function anb(fixture, args, overrides = {}) {
  const result = spawnSync(binary, ['--json', ...args], { cwd: fixture.project, env: { ...fixture.env, ...overrides }, encoding: 'utf8' });
  assert.equal(result.status, 0, `${args.join(' ')}: ${result.stderr}`);
  return JSON.parse(result.stdout);
}
const fields = record => Object.fromEntries(record.fields.rows);
const body = record => (record.body.head || '') + (record.body.tail || '');
const evidence = record => body(record) + JSON.stringify(record.links || record.fields?.rows || []);

async function fingerprint(directory) {
  const files = [];
  async function walk(path) {
    for (const entry of await readdir(path, { withFileTypes: true })) {
      const child = join(path, entry.name);
      if (entry.isDirectory()) await walk(child);
      else files.push([child.slice(directory.length), createHash('sha256').update(await readFile(child)).digest('hex')]);
    }
  }
  await walk(directory);
  return JSON.stringify(files.sort());
}

async function prepare(scenario, model) {
  const directory = await mkdtemp(join(tmpdir(), `anb-${scenario}-`));
  const project = join(directory, 'project');
  const privateHome = join(directory, 'person');
  await Promise.all([mkdir(project), mkdir(privateHome)]);
  const env = { ...process.env, HOME: privateHome, CODEX_HOME: codeHome, ANB_BY: 'Ada', ANB_SESSION: 'current', PATH: `${dirname(binary)}:${process.env.PATH}` };
  delete env.ANB_NOTEBOOK;
  delete env.CODEX_THREAD_ID;
  const fixture = { directory, project, env, scenario, model };
  const add = (type, id, title, text, extra = []) => anb(fixture, ['add', type, title, '--id', id, '--body', text, ...extra]);
  fixture.native = scenario.startsWith('native-');
  if (fixture.native) {
    assert.equal(spawnSync('git', ['init', '--quiet'], { cwd: project }).status, 0);
    // Native integration uses its own trusted host configuration, outside the writable fixture.
    const nativeHome = await mkdtemp(join(tmpdir(), 'anb-native-config-'));
    await symlink(join(codeHome, 'auth.json'), join(nativeHome, 'auth.json'));
    const trustedPaths = [...new Set([project, await realpath(project)])];
    await writeFile(join(nativeHome, 'config.toml'), trustedPaths.map(path => `[projects.${JSON.stringify(path)}]\ntrust_level = "trusted"\n`).join('\n'), { mode: 0o600 });
    env.CODEX_HOME = nativeHome;
  }
  if (scenario !== 'plain-reader') anb(fixture, ['setup', '--agent', fixture.native ? 'codex' : 'agents-md']);
  add('note', 'note.publication', 'Publication vocabulary', 'A Publication is a tenant-scoped delivery to one channel. The canonical product source is https://example.test/product/publication. A Campaign groups Publications; it is not itself published.', ['--kind', 'model', '--by', 'Grace', '--link', 'doc https://example.test/product/publication']);
  if (scenario === 'native-probe') {
    add('task', 'task.hook-probe', 'Resume the indexed conversation', 'The host should deliver this focus without a manual read.');
    anb(fixture, ['start', 'task.hook-probe']);
    fixture.prompt = 'Without using any tools or reading files, name the current Task id and the title of the shared Note supplied in the SessionStart working-memory context. If no such context was supplied, say unavailable. Do not infer the answer from filenames or instructions.';
    fixture.verify = async answer => {
      assert.match(answer, /task\.hook-probe/);
      assert.match(answer, /Publication vocabulary/);
    };
  } else if (scenario === 'continuation' || scenario === 'native-continuation') {
    await writeFile(join(project, 'message.json'), '{"message":"Hello worlt","preview":"unchanged"}\n');
    await writeFile(join(project, 'verify.mjs'), "import assert from 'node:assert/strict'; import fs from 'node:fs'; const data=JSON.parse(fs.readFileSync('message.json')); assert.equal(data.message,'Hello world'); assert.equal(data.preview,'unchanged'); console.log('message and preview verified');\n");
    add('task', 'task.correction', 'Correct the greeting', 'Fix the message in message.json to Hello world. Preserve preview. Run node verify.mjs. A concise recorded outcome is sufficient.');
    add('task', 'task.preview', 'Separate preview work', 'Another session owns this work. Leave its state and the preview value unchanged.');
    anb(fixture, ['start', 'task.preview'], { ANB_SESSION: 'other' });
    anb(fixture, ['start', 'task.correction']);
    anb(fixture, ['comment', 'task.correction', '--body', 'Located the typo in message.json. No source changes yet. Next: correct message, run node verify.mjs and record the result.']);
    fixture.prompt = 'Continue where this session stopped. Finish and verify the small correction. Leave the other session and its work alone.';
    fixture.verify = async () => {
      assert.deepEqual(JSON.parse(await readFile(join(project, 'message.json'), 'utf8')), { message: 'Hello world', preview: 'unchanged' });
      const corrected = anb(fixture, ['show', 'task.correction', '--all']);
      assert.equal(fields(corrected).state, 'closed');
      assert.match(body(corrected), /verif|test|pass/i);
      assert.equal(fields(anb(fixture, ['show', 'task.preview', '--all'])).state, 'active');
      assert.equal(anb(fixture, ['list', '--all', '--archive', '--team']).count, 3);
      assert.equal(anb(fixture, ['recall', '--all'], { ANB_SESSION: 'other' }).focus.id, 'task.preview');
    };
  } else if (scenario === 'intents') {
    add('task', 'task.review', 'Review publication wording', 'Review wording with the product team.');
    fixture.prompt = 'Update our shared Publication definition: it is a tenant-scoped delivery to exactly one channel, and the same Campaign can have several Publications. Preserve the canonical source and the distinction from Campaign. Reassign the pending review to Grace. Remember for me in this project: ask before running the slow integration suite. Across all my projects: answer me in concise English. Do not change external systems.';
    fixture.verify = async () => {
      const model = anb(fixture, ['show', 'note.publication', '--all']);
      assert.match(body(model), /exactly one channel/i);
      assert.match(body(model), /(?:several|multiple|many) Publications/i);
      assert.match(body(model), /not (?:itself )?published/i);
      assert.match(evidence(model), /https:\/\/example\.test\/product\/publication/);
      assert.equal(fields(anb(fixture, ['show', 'task.review', '--all']))['taken-by'], 'Grace');
      const recall = anb(fixture, ['recall', '--all']);
      assert(recall.memories.some(item => item.scope === 'personal' && /slow integration/i.test(body(item))));
      assert(recall.memories.some(item => item.scope === 'global' && /concise English/i.test(body(item))));
      assert.equal(anb(fixture, ['list', '--all', '--archive', '--team']).count, 2);
    };
  } else if (['named-result', 'named-part', 'occupied-result'].includes(scenario)) {
    add('note', 'note.export-idea', 'Export generation cleanup', 'Separate formatting from delivery so CSV and PDF exports can evolve independently. This is the approved scope of the export refactor.', ['--kind', 'idea']);
    add('task', 'task.export-result', 'Restructure export generation', 'Deliver the export formatting cleanup described by note.export-idea. CSV and PDF work are independent parts; the shared timezone migration is an external prerequisite, not part of this result.', ['--from', 'note.export-idea']);
    add('task', 'task.csv', 'Separate the CSV formatter', 'Extract the CSV formatter; do not change output bytes.', ['--from', 'task.export-result']);
    add('task', 'task.pdf', 'Separate the PDF formatter', 'Extract the PDF formatter once the timezone migration is complete.', ['--from', 'task.export-result']);
    add('task', 'task.timezones', 'Shared timezone migration', 'A separate project-wide migration. It is not part of export cleanup.', ['--priority', '0']);
    anb(fixture, ['block', 'task.export-result', 'task.csv']);
    anb(fixture, ['block', 'task.export-result', 'task.pdf']);
    anb(fixture, ['block', 'task.pdf', 'task.timezones']);
    if (scenario === 'occupied-result') anb(fixture, ['edit', 'task.pdf', '--taken-by', 'Grace']);
    fixture.prompt = scenario === 'named-part'
      ? 'Take just the CSV-formatting part of that export cleanup for me, not the overall delivery or the PDF part. Begin that Task, but do not implement or close anything yet.'
      : 'Let us work on that cleanup of export generation. Take responsibility for the overall result and begin its first available piece. Do not implement or close anything yet.';
    fixture.verify = async answer => {
      const result = fields(anb(fixture, ['show', 'task.export-result', '--all']));
      const csv = fields(anb(fixture, ['show', 'task.csv', '--all']));
      const pdf = fields(anb(fixture, ['show', 'task.pdf', '--all']));
      const migration = fields(anb(fixture, ['show', 'task.timezones', '--all']));
      assert.equal(migration.state, 'open'); assert.equal(migration['taken-by'], undefined);
      assert.equal(pdf.state, 'open');
      if (scenario === 'occupied-result') {
        assert.match(answer, /Grace/);
        assert.match(answer, /\?|confirm|clarif|coordinate|approval|permission/i);
        assert.equal(result.state, 'open'); assert.equal(result['taken-by'], undefined);
        assert.equal(csv.state, 'open'); assert.equal(csv['taken-by'], undefined);
        assert.equal(pdf['taken-by'], 'Grace');
      } else {
        assert.equal(csv.state, 'active'); assert.equal(csv['taken-by'], 'Ada');
        assert.equal(pdf['taken-by'], undefined);
        assert.equal(result.state, scenario === 'named-result' ? 'active' : 'open');
        assert.equal(result['taken-by'], scenario === 'named-result' ? 'Ada' : undefined);
        assert.equal(anb(fixture, ['recall', '--all']).focus.id, 'task.csv');
      }
      if (scenario !== 'occupied-result') assert.equal(anb(fixture, ['list', '--all', '--archive', '--team']).count, 6);
    };
  } else if (scenario === 'parallel-next') {
    add('task', 'task.release', 'Prepare the release', 'This hub completes only after both independent deliveries.');
    for (const side of ['left', 'right']) {
      await writeFile(join(project, `${side}.txt`), 'pending\n');
      add('task', `task.${side}`, `Prepare ${side}`, `Change only ${side}.txt to the single word ready, verify its contents, and close this Task with a concise outcome. Leave other delivery files alone.`, ['--from', 'task.release']);
      anb(fixture, ['block', 'task.release', `task.${side}`]);
    }
    add('task', 'task.foreign', 'Grace has separate work', 'Not part of this release.', ['--taken-by', 'Grace']);
    fixture.prompt = 'Take exactly one next ready task for task.release, carry it out, verify and close it. Other agent sessions are working here at the same time. Do not close the hub, claim another task, or change the other agent\'s files.';
    fixture.verify = async () => {
      const focus = anb(fixture, ['recall', '--all']).focus;
      assert(['task.left', 'task.right'].includes(focus.id));
      assert.equal(fields(focus).state, 'closed');
      assert.equal((await readFile(join(project, `${focus.id.slice(5)}.txt`), 'utf8')).trim(), 'ready');
    };
  } else if (scenario === 'consolidation') {
    add('decision', 'decision.retention-old', 'Export retention', 'Keep export manifests for 30 days because the original support window was 30 days.', ['--kind', 'rule']);
    add('question', 'question.retention', 'Which retention window applies?', 'Legal is reviewing the new support window.');
    add('task', 'task.probe', 'Probe the export endpoint', 'An exploratory probe, not a change to product policy.');
    fixture.prompt = 'We settled question.retention: the approved shared rule is now to retain export manifests for 90 days, because the support window was extended. The authority is https://example.test/policy/retention-90. Replace the old ruling while preserving its reasoning and history, and close the question with its resolution. Also record on task.probe that one request timed out today; this single attempt does not establish a general reliability rule. Do not visit or change external systems.';
    fixture.verify = async () => {
      const old = anb(fixture, ['show', 'decision.retention-old', '--all']);
      assert.equal(fields(old).state, 'superseded');
      assert.match(body(old), /original support window was 30 days/);
      const knowledge = anb(fixture, ['recall', '--all'], { ANB_BY: 'Grace', ANB_SESSION: 'grace' }).memories;
      const replacement = knowledge.find(item => item.id.startsWith('decision.') && /90 days/.test(body(item)));
      assert(replacement, 'The new ruling is not recalled for another person');
      assert.match(evidence(replacement), /https:\/\/example\.test\/policy\/retention-90/);
      assert(!knowledge.some(item => item.id === 'decision.retention-old'));
      const question = fields(anb(fixture, ['show', 'question.retention', '--all']));
      assert.equal(question.state, 'closed'); assert.equal(question['resolved-by'], replacement.id);
      assert.match(body(anb(fixture, ['show', 'task.probe', '--all'])), /timed? out|timeout/i);
      assert.equal(anb(fixture, ['list', '--all', '--archive', '--team']).count, 5);
    };
  } else if (scenario === 'plain-reader') {
    fixture.prompt = 'Without running anb or using a notebook skill, read the plain project files and explain what a Publication is, how it relates to Campaign, and where the canonical definition is maintained. Do not modify files or visit the external source.';
    const before = await fingerprint(project);
    fixture.verify = async answer => {
      assert.match(answer, /tenant/i); assert.match(answer, /channel/i); assert.match(answer, /Campaign/);
      assert.match(answer, /https:\/\/example\.test\/product\/publication/);
      assert.equal(await fingerprint(project), before);
    };
  } else if (scenario === 'bounded-read') {
    add('task', 'task.policy', 'Confirm the export exception', 'Find the exact export retention exception in note.export-policy, including its source. Do not change records.');
    const padding = 'Ordinary exports follow the documented tenant policy.\n'.repeat(55);
    add('note', 'note.export-policy', 'Export retention policy', `${padding}\nThe legal-hold exception retains export manifests for exactly 73 days. Its authority is https://example.test/policy/legal-hold-73.\n${padding}`, ['--from', 'task.policy']);
    for (let index = 0; index < 18; index++) add('note', `note.noise-${index}`, `Other component ${index}`, 'Unrelated component guidance. '.repeat(40));
    anb(fixture, ['start', 'task.policy']);
    assert(!JSON.stringify(anb(fixture, ['recall'])).includes('73 days'), 'Fixture exposes the answer without expanding the bounded read');
    fixture.prompt = 'What is the exact export retention exception we need for this work, and its source? Read enough to be certain. Do not modify files or access external services.';
    const before = await fingerprint(project);
    fixture.verify = async answer => {
      assert.match(answer, /73 days/); assert.match(answer, /https:\/\/example\.test\/policy\/legal-hold-73/);
      assert.equal(await fingerprint(project), before);
    };
  } else throw new Error(`Unknown scenario: ${scenario}`);
  return fixture;
}

async function trial(fixture) {
  const name = `${fixture.model}-${fixture.scenario}`;
  const started = Date.now();
  const prompt = `Work only inside this isolated evaluation fixture. No commits, pushes, internet, or external services. ${fixture.scenario === 'plain-reader' ? '' : 'The installed anb command is available; use the project instructions and skill.'}\n\n${fixture.prompt}`;
  const args = ['exec', '--ephemeral', ...(fixture.native ? [] : ['--ignore-user-config']), '--skip-git-repo-check', '--sandbox', 'workspace-write', '-C', fixture.project, '--add-dir', fixture.directory, '--model', fixture.model, '-c', 'model_reasoning_effort="low"', '--json', '-o', join(output, `${name}.answer.md`), ...(fixture.native ? ['--enable', 'hooks', '--dangerously-bypass-hook-trust'] : []), prompt];
  const result = await new Promise((resolveResult, reject) => {
    const child = spawn('codex', args, { env: fixture.env, stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '', stderr = '';
    const timer = setTimeout(() => child.kill('SIGTERM'), 600_000);
    child.stdout.on('data', chunk => { stdout += chunk; });
    child.stderr.on('data', chunk => { stderr += chunk; });
    child.on('error', reject);
    child.on('close', code => { clearTimeout(timer); resolveResult({ code, stdout, stderr }); });
  });
  await writeFile(join(output, `${name}.jsonl`), result.stdout);
  await writeFile(join(output, `${name}.stderr.txt`), result.stderr);
  let error = null;
  try {
    assert.equal(result.code, 0, result.stderr || result.stdout);
    await fixture.verify(await readFile(join(output, `${name}.answer.md`), 'utf8'));
    if (fixture.scenario === 'native-probe') {
      assert(!result.stdout.split('\n').filter(Boolean).map(JSON.parse).some(event => event.item?.type === 'command_execution'), 'Native context probe used a command to obtain the answer');
    }
    if (fixture.scenario === 'plain-reader') {
      const commands = result.stdout.split('\n').filter(Boolean).map(line => JSON.parse(line)).filter(event => event.type === 'item.completed' && event.item?.type === 'command_execution').map(event => event.item.command);
      assert(!commands.some(command => /\banb\s+(recall|show|list|graph|status)\b/.test(command)), 'Plain reader invoked the CLI');
      assert(!commands.some(command => /\b(curl|wget)\s|\bopen\s+https?:/.test(command)), 'Plain reader fetched an external source');
    }
    if (fixture.scenario !== 'parallel-next') {
      const checked = anb(fixture, ['check', '--all']);
      assert.equal(checked.count, 0, JSON.stringify(checked));
    }
  } catch (failure) { error = failure.message; }
  const summary = { scenario: fixture.scenario, model: fixture.model, passed: error === null, durationMs: Date.now() - started, fixture: fixture.directory, error };
  await writeFile(join(output, `${name}.result.json`), JSON.stringify(summary, null, 2) + '\n');
  console.log(JSON.stringify(summary));
  return summary;
}

console.log(JSON.stringify({ output, binary, models, scenarios }));
const results = [];
for (const scenario of scenarios) {
  if (scenario === 'parallel-next') {
    assert(models.length === 2, 'The parallel scenario uses two models/sessions');
    const fixture = await prepare(scenario, models[0]);
    const peers = models.map((model, index) => {
      const peer = { ...fixture, model, env: { ...fixture.env, ANB_SESSION: `peer-${index}` } };
      peer.verify = async () => {
        const focus = anb(peer, ['recall', '--all']).focus;
        assert(['task.left', 'task.right'].includes(focus.id));
        assert.equal(fields(focus).state, 'closed');
        assert.equal((await readFile(join(peer.project, `${focus.id.slice(5)}.txt`), 'utf8')).trim(), 'ready');
      };
      return peer;
    });
    results.push(...await Promise.all(peers.map(trial)));
    const focuses = peers.map(peer => anb(peer, ['recall', '--all']).focus.id);
    assert.equal(new Set(focuses).size, 2, 'Parallel sessions chose the same task');
    assert.equal(fields(anb(fixture, ['show', 'task.release', '--all'])).state, 'open');
    assert.equal(fields(anb(fixture, ['show', 'task.foreign', '--all']))['taken-by'], 'Grace');
    assert.equal(anb(fixture, ['list', '--all', '--archive', '--team']).count, 5);
    assert.equal(anb(fixture, ['check', '--all']).count, 0);
  } else {
    results.push(...await Promise.all(models.map(async model => trial(await prepare(scenario, model)))));
  }
  await writeFile(join(output, 'results.json'), JSON.stringify(results, null, 2) + '\n');
}
process.exitCode = results.every(result => result.passed) ? 0 : 1;
