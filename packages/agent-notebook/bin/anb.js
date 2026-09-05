#!/usr/bin/env node
'use strict'

// The npm package is a thin shim: the binary for this machine arrives as an
// optional dependency, one package per platform, and this file finds it and
// hands the whole command line over. Nothing is downloaded and nothing runs
// at install time.
const { spawnSync } = require('node:child_process')

const PACKAGES = {
  'darwin-arm64': '@supolka/agent-notebook-darwin-arm64',
  'darwin-x64': '@supolka/agent-notebook-darwin-x64',
  'linux-x64': '@supolka/agent-notebook-linux-x64',
  'linux-arm64': '@supolka/agent-notebook-linux-arm64',
  'win32-x64': '@supolka/agent-notebook-win32-x64',
}

const platform = `${process.platform}-${process.arch}`
const packageName = PACKAGES[platform]

if (packageName === undefined) {
  console.error(
    `anb: no binary is published for ${platform}; build from source with ` +
      'cargo install --git https://github.com/maksimyaromin/agent-notebook anb',
  )
  process.exit(1)
}

let binary
try {
  binary = require.resolve(`${packageName}/bin/${process.platform === 'win32' ? 'anb.exe' : 'anb'}`)
} catch {
  console.error(
    `anb: ${packageName} is not installed; it is an optional dependency of @supolka/agent-notebook, ` +
      'so reinstall with optional dependencies enabled (npm install --include=optional)',
  )
  process.exit(1)
}

const run = spawnSync(binary, process.argv.slice(2), { stdio: 'inherit', windowsHide: true })

if (run.error) {
  console.error(`anb: ${run.error.message}`)
  process.exit(1)
}
if (run.signal) {
  process.kill(process.pid, run.signal)
}
process.exit(run.status ?? 1)
