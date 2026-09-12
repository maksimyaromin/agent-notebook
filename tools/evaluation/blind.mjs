#!/usr/bin/env node
// Discoverability and plain-file usefulness, with an otherwise identical no-memory control.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, mkdir, readFile, readdir, realpath, symlink, writeFile } from 'node:fs/promises';
import { tmpdir, homedir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn, spawnSync } from 'node:child_process';

const repository = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const binary = process.env.ANB_EVAL_BINARY || join(repository, 'target/debug/anb');
const models = (process.env.ANB_EVAL_MODELS || 'gpt-5.6-luna,gpt-6-astra').split(',');
const breadcrumb = process.env.ANB_EVAL_BREADCRUMB === '1';
const output = resolve(process.env.ANB_EVAL_OUTPUT || join(repository, `.tmp/evaluation/blind${breadcrumb ? '-breadcrumb' : ''}`));
const credentialHome = process.env.CODEX_HOME || join(homedir(), '.codex');
const commandPath = name => {
  const found = spawnSync('/bin/sh', ['-c', 'command -v "$1"', 'find-command', name], { encoding: 'utf8' });
  assert.equal(found.status, 0, `${name} is unavailable`);
  return found.stdout.trim();
};
const codex = commandPath('codex');
const prompt = 'Before we implement campaign cancellation, explain the intended behavior for scheduled and already-published deliveries, including reserved credits. Base the answer on established product behavior and identify what remains undecided. Do not change files or contact external services.';
const source = 'https://product.example.test/policies/campaign-cancellation-37';
const files = {
  'README.md': '# Relay\n\nRelay is a small service for campaign delivery. The current module exposes campaign lookup and the cancellation entry point.\n\n## Development\n\nRun `node --test` from the repository root. No installation or network connection is needed.\n',
  'package.json': '{"name":"relay","private":true,"type":"module","scripts":{"test":"node --test"}}\n',
  'src/campaigns.mjs': '/** @typedef {{id: string, campaignId: string, channelId: string, state: "scheduled"|"published"|"cancelled", reservedCredits: number}} Delivery */\n\nexport function findCampaign(campaigns, id) {\n  return campaigns.find(campaign => campaign.id === id) ?? null;\n}\n\nexport function requestCampaignCancellation(campaignId) {\n  throw new Error(`Cancellation is not implemented for ${campaignId}`);\n}\n',
  'test/campaigns.test.mjs': 'import { test } from "node:test";\nimport assert from "node:assert/strict";\nimport { findCampaign } from "../src/campaigns.mjs";\n\ntest("campaign lookup does not mutate the collection", () => {\n  const campaigns = [{id: "summer", title: "Summer launch"}];\n  assert.equal(findCampaign(campaigns, "summer"), campaigns[0]);\n  assert.equal(findCampaign(campaigns, "missing"), null);\n  assert.equal(campaigns.length, 1);\n});\n',
};
if (breadcrumb) files['README.md'] += '\n## Project references\n\n[Project decisions and working notes](.agent-notebook/) are plain Markdown.\n';

async function fingerprint(directory) {
  const entries = [];
  async function walk(at) {
    for (const entry of await readdir(at, { withFileTypes: true })) {
      const path = join(at, entry.name);
      if (entry.isDirectory()) await walk(path);
      else entries.push([path.slice(directory.length), createHash('sha256').update(await readFile(path)).digest('hex')]);
    }
  }
  await walk(directory);
  return JSON.stringify(entries.sort());
}

async function prepare(model, memory) {
  const directory = await mkdtemp(join(tmpdir(), 'relay-discovery-'));
  const project = join(directory, 'project');
  const host = await mkdtemp(join(tmpdir(), 'relay-agent-host-'));
  const person = join(directory, 'person');
  const commands = join(host, 'bin');
  await Promise.all([mkdir(project), mkdir(person), mkdir(commands)]);
  await symlink(join(credentialHome, 'auth.json'), join(host, 'auth.json'));
  for (const tool of ['node', 'rg']) await symlink(await realpath(commandPath(tool)), join(commands, tool));
  const env = { ...process.env, HOME: person, CODEX_HOME: host, PATH: `${commands}:/usr/bin:/bin:/usr/sbin:/sbin` };
  for (const name of Object.keys(env)) if (name.startsWith('ANB_') || name === 'CODEX_THREAD_ID' || name === 'CLAUDE_ENV_FILE') delete env[name];
  const noCli = spawnSync('/bin/zsh', ['-lc', 'command -v anb'], { env, encoding: 'utf8' });
  assert.notEqual(noCli.status, 0, 'The blind agent can resolve anb from its shell');
  for (const [name, body] of Object.entries(files)) {
    const path = join(project, name);
    await mkdir(dirname(path), { recursive: true });
    await writeFile(path, body);
  }
  const ordinary = await fingerprint(project);
  assert(!Object.values(files).join('').includes('37'));
  assert(!Object.values(files).join('').includes(source));
  const recordPaths = [];
  if (memory) {
    const anb = args => {
      const result = spawnSync(binary, ['--json', ...args], { cwd: project, env: { ...env, ANB_BY: 'Product team' }, encoding: 'utf8' });
      assert.equal(result.status, 0, result.stderr);
      return JSON.parse(result.stdout);
    };
    const add = (type, title, body, extra) => {
      const record = anb(['add', type, title, '--body', body, ...extra]);
      recordPaths.push(record.path);
      return record.id;
    };
    add('note', 'Campaign and Publication boundaries', '# Campaign and Publication boundaries\n\nA Campaign groups Publications; it is not itself delivered to a channel. A Publication is one tenant-scoped delivery to one channel. The same Campaign may have several Publications. The code calls these delivery rows.\n', ['--kind', 'model']);
    const old = add('decision', 'Campaign cancellation credits', '# Campaign cancellation credits\n\nThe original pilot rule cancelled every delivery in a Campaign, including already-published deliveries, and released reserved credits immediately. This rule covered the pilot only.\n', ['--kind', 'rule']);
    add('decision', 'Campaign cancellation credits', '# Campaign cancellation credits\n\nApproved current behavior: cancelling a Campaign cancels only scheduled Publications. Already-published Publications remain published and are not deleted or recalled. Release the cancelled scheduled Publications\' reserved credits exactly 37 minutes after cancellation, not immediately, so settlement reconciliation can finish. Do not release credits for an already-published Publication.\n\nThis replaces the pilot rule. The cancellation behavior for a Publication already being dispatched remains undecided and requires a product decision.\n\nCanonical source: ' + source + '\n', ['--kind', 'rule', '--supersedes', old, '--link', 'doc ' + source]);
    anb(['archive', old]);
    assert.equal(anb(['check', '--all']).count, 0);
  }
  return { model, memory, directory, project, env, ordinary, recordPaths, before: await fingerprint(project) };
}

function execute(fixture, answerPath) {
  const args = ['exec', '--ephemeral', '--ignore-user-config', '--disable', 'hooks', '--disable', 'memories', '--skip-git-repo-check', '--sandbox', 'read-only', '-C', fixture.project, '--model', fixture.model, '-c', 'model_reasoning_effort="low"', '--json', '-o', answerPath, prompt];
  return new Promise((resolveResult, reject) => {
    const child = spawn(codex, args, { env: fixture.env, stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '', stderr = '';
    const timer = setTimeout(() => child.kill('SIGTERM'), 240_000);
    const hardStop = setTimeout(() => child.kill('SIGKILL'), 245_000);
    child.stdout.on('data', chunk => { stdout += chunk; });
    child.stderr.on('data', chunk => { stderr += chunk; });
    child.on('error', error => { clearTimeout(timer); clearTimeout(hardStop); reject(error); });
    child.on('close', code => { clearTimeout(timer); clearTimeout(hardStop); resolveResult({ code, stdout, stderr }); });
  });
}

async function trial(fixture) {
  const name = `${fixture.model}-${fixture.memory ? 'memory' : 'control'}`;
  const began = Date.now();
  const answerPath = join(output, name + '.answer.md');
  const result = await execute(fixture, answerPath);
  await writeFile(join(output, name + '.jsonl'), result.stdout);
  await writeFile(join(output, name + '.stderr.txt'), result.stderr);
  const events = result.stdout.split('\n').filter(Boolean).map(JSON.parse);
  const commands = events.filter(event => event.type === 'item.completed' && event.item?.type === 'command_execution').map(event => event.item);
  const reads = commands.map(item => ({ command: item.command, output: item.aggregated_output }));
  await writeFile(join(output, name + '.reads.json'), JSON.stringify(reads, null, 2) + '\n');
  const answer = await readFile(answerPath, 'utf8').catch(() => '');
  const observations = {
    readMemoryFact: commands.some(item => String(item.aggregated_output).includes('37 minutes') && String(item.aggregated_output).includes(source)),
    attemptedCli: commands.some(item => /(?:^|[\s/])anb(?:\s|$)|cargo\s+run|npx\s/.test(item.command)),
    readSkill: commands.some(item => /SKILL\.md|\.agents\/skills|\.claude\/skills/.test(item.command)),
    externalTool: events.some(event => event.item && /web_search|mcp_tool_call/.test(event.item.type)),
    currentRule: /37\s+minutes/i.test(answer),
    source: answer.includes(source),
    publishedPreserved: /(?:already[- ]published|published).{0,220}(?:remain|keep|preserv|not|unchanged|stay)|(?:do not|don't|not).{0,90}(?:delete|recall|cancel).{0,90}published/is.test(answer),
    scheduledCancelled: /scheduled.{0,150}cancel|cancel.{0,150}scheduled/is.test(answer),
    undecided: /dispatch.{0,150}(?:undecided|unknown|unresolved|decision|unspecified)|(?:undecided|unknown|unresolved|decision|unspecified).{0,150}dispatch/is.test(answer),
    controlUncertain: /unknown|undocumented|unclear|undecided|not yet a confirmed product decision|not (?:defined|specified|established|implemented)|need(?:s)?[^.]{0,100}(?:confirm|decid|clarif)|cannot|can't|no (?:defined|documented|established)/i.test(answer),
    unchanged: await fingerprint(fixture.project) === fixture.before,
  };
  let error = null;
  try {
    assert.equal(result.code, 0, result.stderr);
    assert(observations.unchanged, 'The reader changed fixture files');
    assert(!observations.attemptedCli && !observations.readSkill && !observations.externalTool, 'The reader used excluded assistance');
    if (fixture.memory) {
      for (const key of ['readMemoryFact', 'currentRule', 'publishedPreserved', 'scheduledCancelled', 'undecided']) assert(observations[key], `Missing acceptance: ${key}`);
    } else {
      assert(!observations.currentRule && !observations.source, 'The control invented the missing rule or authority');
      assert(observations.controlUncertain, 'The control did not acknowledge missing product policy');
    }
  } catch (failure) { error = failure.message; }
  const dimensions = {
    discoveredMemory: fixture.memory ? observations.readMemoryFact : null,
    correctMemoryUse: fixture.memory ? ['readMemoryFact', 'currentRule', 'publishedPreserved', 'scheduledCancelled', 'undecided'].every(key => observations[key]) : null,
    sourceCitation: observations.source ? 'canonical' : /\.agent-notebook\/(?:archive\/)?(?:decisions|notes|tasks|questions)\/[^\s)]+\.md/.test(answer) ? 'local-record' : 'none',
    groundedControl: fixture.memory ? null : observations.controlUncertain && !observations.currentRule && !observations.source,
  };
  const summary = { model: fixture.model, memory: fixture.memory, breadcrumb, passed: error === null, durationMs: Date.now() - began, fixture: fixture.project, observations, dimensions, error };
  await writeFile(join(output, name + '.result.json'), JSON.stringify(summary, null, 2) + '\n');
  console.log(JSON.stringify(summary));
  return summary;
}

await mkdir(output, { recursive: true });
await writeFile(join(output, 'prompt.txt'), prompt + '\n');
const fixtures = await Promise.all(models.flatMap(model => [true, false].map(memory => prepare(model, memory))));
assert.equal(new Set(fixtures.map(fixture => fixture.ordinary)).size, 1, 'Ordinary code and documentation differ between treatments');
await writeFile(join(output, 'fixtures.json'), JSON.stringify(fixtures.map(({ model, memory, project, ordinary, recordPaths }) => ({ model, memory, project, ordinary, recordPaths })), null, 2) + '\n');
console.log(JSON.stringify({ output, models, breadcrumb, prompt }));
const results = await Promise.all(fixtures.map(trial));
await writeFile(join(output, 'results.json'), JSON.stringify(results, null, 2) + '\n');
process.exitCode = results.every(result => result.passed) ? 0 : 1;
