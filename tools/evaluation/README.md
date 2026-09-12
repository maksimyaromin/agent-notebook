# Reproducible evaluations

These checks supplement the repository's ordinary test gate. Run them from the repository root. They use scratch projects or explicitly selected research checkouts, not the project's working notebook.

## Agent behavior

The [agent runner](./agents.mjs) gives fresh agents ordinary requests and checks the resulting files and records. It requires an authenticated Codex CLI and access to the selected models. Runs consume model usage. The default four scenarios cover continuation, editing and assignment, reading Markdown without the CLI or skill, and retrieving a fact omitted from the initial context. Run the complete non-native set with:

```sh
cargo build --locked -p anb
ANB_EVAL_SCENARIOS=continuation,intents,plain-reader,bounded-read,consolidation,parallel-next,named-result,named-part,occupied-result node tools/evaluation/agents.mjs
```

Consolidation distinguishes a settled ruling from an isolated observation. Parallel next asks two simultaneous sessions of one person to complete different ready Tasks under the same hub. The named-work scenarios resolve an ordinary description into the whole result or one requested part, leave external prerequisites outside that assignment, and ask how to coordinate when a constituent already belongs to another person. These scenarios verify preserved sources, explicit supersession, private audiences, independent focus and unchanged work owned by another person. The defaults are `gpt-5.6-luna` and `gpt-6-astra` with low reasoning effort. Set `ANB_EVAL_MODELS` to change the comma-separated model list; `parallel-next` requires exactly two models.

Native startup is a separate check, because an agent can recover through a manual read even if the host never ran its hook:

```sh
ANB_EVAL_SCENARIOS=native-probe,native-continuation node tools/evaluation/agents.mjs
```

These scenarios install the real Codex integration. The context probe must name the focused Task and shared Note without any tool call. The continuation scenario must complete the focused work without disturbing another session. The runner creates an isolated trusted host configuration and links the existing file-based `auth.json`; it does not edit the user's configuration or copy credential contents. It bypasses hook review only for the integration it just generated inside the disposable fixture. Other scenarios ignore user configuration.

Fixtures receive a temporary home directory and project. They are instructed not to commit, push or contact external services. Raw trajectories, answers, assertion results and timings go to the temporary output directory printed at startup. `ANB_EVAL_OUTPUT` selects another output directory, and `ANB_EVAL_BINARY` selects another binary. Keep outputs private: they can contain local paths and host diagnostics. A passing trial establishes the tested behavior on those models and fixtures, not success for every possible natural-language request.

## Reading without the tool

The [plain-file runner](./blind.mjs) asks the same product question in matched repositories with and without relevant memory. It supplies no CLI, notebook skill or record-specific prompt. The ordinary source tree does not contain the distinctive business rule; the memory contains its current Decision and a contradictory superseded one. The reader must apply the current rule and preserve uncertainty without changing files or using external services. Controls must not invent the missing rule.

```sh
ANB_EVAL_BREADCRUMB=1 node tools/evaluation/blind.mjs
```

The breadcrumb is a generic README link to the memory folder, not a hint about which record or answer to use. Omitting the variable explores discovery without that link as a separate diagnostic, not a requirement that every agent find an unadvertised hidden directory. Results separate discovery, correct use, source attribution and grounded uncertainty. Quoting an exact upstream URL is not required when the user did not ask for it. This runner also consumes model usage and keeps its evidence private.

## Existing notebooks

The [adoption runner](./adoption.mjs) copies a selected notebook into a temporary directory before doing any work:

```sh
node tools/evaluation/adoption.mjs /path/to/existing/.agent-notebook
```

It checks that migration preview changes no record bytes, that migration preserves every complete record view and body, and that a repeated migration leaves no further normalization work. It prints the migration and validation results with the scratch path. The supplied notebook is only read; its files and Git state remain untouched. Review the returned `check` findings before applying migration to a real project.

## Default-read scalability

The [performance runner](./performance.mjs) measures the real CLI over 100, 1,000 and 5,000 synthetic live records. Each fresh notebook receives its complete fixture through one import. The fixture contains Tasks, Notes and Decisions, short dependency edges, source links and about 2 KiB of prose per record. It uses an isolated home directory and Git repository; it does not change the project's working notebook.

```sh
cargo build --release --locked -p anb
node tools/evaluation/performance.mjs
```

The runner performs two warmups and 15 interleaved samples per command, preserving the default limits on `status`, `recall` and `ready`. It records every sample, median and nearest-rank p95 wall time, TOON byte length, estimated tokens and omission fields. Process startup and stdout capture are included; import time is recorded separately. Corresponding JSON reads check total counts, readiness, omissions and the reported budget outside the timed samples. A fingerprint verifies that the measured reads change no notebook bytes.

Raw results, first-sample replies and generated fixtures stay under the ignored `.tmp/evaluation/performance-*` directory printed at completion. Set `ANB_EVAL_BINARY` to measure another binary or `ANB_PERF_REPETITIONS` to change the sample count, with a minimum of five. The result records the binary hash and machine details. It fails if these ordinary fixtures exceed their 1,500-token Status/Recall budget, grow a default reply past 8 KiB, omit counts or lose the follow-up read. A latency increase exceeding twice the corpus-size ratio produces a diagnostic warning for repeated materialization or measurement noise.

This is a warm-cache scaling check, not a benchmark against another tool. It does not prove asymptotic complexity, measure peak memory or establish a latency guarantee; machine load, filesystem behavior and the small sample affect the results. The byte ceiling guards this fixture's ordinary output, not every possible record title or minimum useful reply under a tiny requested budget.

## Official TOON fixtures

The [conformance runner](../../crates/anb/tests/toon_conformance.rs) executes every case from the official TOON v4.1 fixture tree at revision `d6db4b04303bdea132351ce45aed612311c850b2`: 23 files and 538 cases, with none skipped. It checks exact encoded strings, decoded JSON values and required errors, applying each fixture's delimiter, indentation and strictness options. The runner refuses a different revision, modified fixtures or unhandled options. The [upstream fixture instructions](https://github.com/toon-format/spec/blob/d6db4b04303bdea132351ce45aed612311c850b2/tests/README.md) define the fixture contract.

```sh
fixture_dir="$(mktemp -d)"
git clone https://github.com/toon-format/spec.git "$fixture_dir/spec"
git -C "$fixture_dir/spec" checkout --detach d6db4b04303bdea132351ce45aed612311c850b2
ANB_TOON_SPEC="$fixture_dir/spec" cargo test --locked -p anb --test toon_conformance -- --ignored --nocapture
```

An existing clean checkout works too: set `ANB_TOON_SPEC` to its directory. The runner only reads it. The test is ignored in the ordinary offline gate because it requires that external checkout; invoking it explicitly without the checkout fails rather than silently skipping cases.

The runner also checks four application-shaped round trips through the production encoder: C0 control characters and Unicode, table cells with punctuation and quotes, unsigned 64-bit integer boundaries, and strings that resemble headings, list items or terminal escapes. Encoding is deterministic, decoding preserves the exact JSON values, and literal controls other than output newlines do not reach the terminal. The ordinary [TOON reply tests](../../crates/anb/tests/toon.rs) separately cover the shared JSON projection, output budgets, counts, omitted rows, action qualification and Markdown skill output.

Verified result with the pinned codec: **538 official cases and four adversarial round trips pass**. A run against a different Git revision was also verified to fail at the pin check. This is fixture and application-contract evidence, not a claim about every possible host-language value; the fixture format cannot represent non-JSON values such as dates or non-finite numbers.

The fixtures remain in their upstream checkout. No fixture or competing implementation code is vendored here. Their [MIT license](https://github.com/toon-format/spec/blob/d6db4b04303bdea132351ce45aed612311c850b2/LICENSE) remains with that checkout.

## Codec selection and supply

The application uses the published `reddb-io-toon` crate pinned to **exactly `0.29.8`** in [Cargo.toml](../../crates/anb/Cargo.toml). [Cargo.lock](../../Cargo.lock) records the registry archive checksum `972efbbd42db652a44aa1a6dbd8cd8a76dd2b7a1c5f3b0028411c9e7d0f79b86`; the inspected archive's SHA-256 matched it. Its packaged VCS metadata points to upstream revision `615cc6060ff9e5ef970cd1e77d7133d029bfa027`, directory `crates/toon`.

The normalized package manifest declares Rust 1.75, MIT licensing and no build script. Its only direct runtime dependency is `serde_json`, already used by agent-notebook, with `preserve_order` and `float_roundtrip`. The upstream [crate manifest](https://github.com/reddb-io/toon/blob/615cc6060ff9e5ef970cd1e77d7133d029bfa027/crates/toon/Cargo.toml) and [license](https://github.com/reddb-io/toon/blob/615cc6060ff9e5ef970cd1e77d7133d029bfa027/LICENSE) support those metadata checks. Inspect the resolved dependency graph with:

```sh
cargo tree --locked -p reddb-io-toon --edges normal
cargo metadata --locked --format-version 1
```

The production path calls the canonical encoder with its defaults. In the inspected [encoder options](https://github.com/reddb-io/toon/blob/615cc6060ff9e5ef970cd1e77d7133d029bfa027/crates/toon/src/lib_parts/encode.rs), the three RedDB extension switches are false. agent-notebook does not use TOONL, custom array columns, cyclic encoding or a local format fork. The version pin and executable fixtures make a codec upgrade a reviewable change. These checks establish provenance and behavior at this version; they are not an independent security audit of the dependency.

### Alternatives examined

| Candidate | Evidence | Decision |
|---|---|---|
| Official `toon-format` Rust checkout `2136cb1a35f3ed63be733bb74d36b89d3b4592dd`, package version `0.5.0` | Its [README](https://github.com/toon-format/toon-rust/blob/2136cb1a35f3ed63be733bb74d36b89d3b4592dd/README.md) targets spec v3.0. The string encoder emits a bare leading `#`, leaves ESC literal and permits an unquoted Unicode key. These three v4 incompatibilities were reproduced through `encode_default`; the relevant code is [string.rs](https://github.com/toon-format/toon-rust/blob/2136cb1a35f3ed63be733bb74d36b89d3b4592dd/src/utils/string.rs). | Not used for a v4.1 output contract. |
| Official Rust [v4.1 pull request #79](https://github.com/toon-format/toon-rust/pull/79) | An implementation proposal was available but unmerged when reviewed. Its reported upstream test results were not counted as agent-notebook's own evidence. | No application dependency on an unmerged branch. Reconsider after a suitable release. |
| Published `reddb-io-toon =0.29.8` | The pinned official suite and four production-encoder round trips above pass, without extension flags or local patches. | Selected as the codec behind the shared reply document. |

The three official Rust probe inputs and v4 expectations were `"# heading"` → `"\"# heading\""`, `"\u001b[31m"` → `"\"\\u001b[31m\""`, and `{"雪":"value"}` → `"\"雪\": value"`. These are JSON-escaped strings describing the expected TOON bytes. They explain the rejection without importing that candidate into the application's dependency graph.
