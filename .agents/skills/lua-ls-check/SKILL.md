---
name: lua-ls-check
description: Use when validating or extending Lua type completion / diagnostics for cworks Lua scripts (src/lib/lua) with LuaLS headless, without VS Code. Covers the nix-provided lua-language-server --check invocation, the three pillars needed for require("gui")/session/env/FSObj resolution (runtime.path, ---@meta stubs under src/lib/lua/types, diagnostics.globals), and a throwaway probe-file technique that proves member resolution. Trigger keywords: lua-language-server, LuaLS, sumneko, .luarc.json, Lua completion, Lua 補完, luadoc, ---@meta stub, undefined-field, missing-require, src/lib/lua/types, rust-analyzer userdata type.
---

# Headless LuaLS check for cworks Lua scripts

## When to use
Use when changing Lua type-completion support (`.luarc.json`, the `Lua.*` block in
`front.code-workspace`, `src/lib/lua/types/*.lua`), when a Rust-side API is exposed to Lua and you
must confirm the language server still resolves it, or to validate Lua edits without opening VS Code.

## Prerequisites
The checker is pulled on demand, so no extension install is needed:
`nix run nixpkgs#lua-language-server` (validated with 3.19.1). Build the project through the Nix
devshell as usual (`nix develop --command ...`).

## Procedure
1. Run the whole-workspace diagnostic check. Logs go to `/tmp`, never into the repo:
   ```bash
   nix run nixpkgs#lua-language-server -- \
     --check=. --configpath=.luarc.json --checklevel=Warning --logpath=/tmp/opencode/luals
   ```
   Success ends with `Diagnosis completed, no problems found`. Findings print as
   `path:line:col [Level] message (rule-id)`.
2. Read `.luarc.json` before diagnosing. Completion needs all three of:
   - `runtime.path` includes `src/lib/lua/usr/lib/?.lua` — without it `require("gui")`,
     `require("cworks")`, `require("path")`, `require("json")` do not resolve.
   - `workspace.library` includes `src/lib/lua/types` — the directory holding the `---@meta` stubs.
   - `diagnostics.globals` lists `session` and `env` — `bootstrap.lua` injects these into the Lua
     state at runtime, so they are globals, not requireable modules.
3. For a new runtime-injected global or a Rust-backed userdata, add a `---@class` block to
   `src/lib/lua/types/cworks-runtime.lua`. Do not try to `require` it. Keep class names aligned with
   the Rust registration names (`LuaSession` -> `session`, `LuaFSObj` -> `FSObj`).
4. Prove member resolution. A clean check does NOT prove completion works, because
   `undefined-field` and `missing-require` are off by default:
   - Write a throwaway probe under `src/lib/lua/usr/bin/` exercising the new API.
   - Temporarily add to `.luarc.json`:
     `"diagnostics.severity": {"undefined-field":"Warning","missing-require":"Warning","undefined-global":"Warning"}`
   - Re-run the check. Zero warnings means every called method actually resolved.
   - Delete the probe and the temporary `diagnostics.severity` entry, then re-run to confirm clean.
   Keep the probe in its own file and remove it afterwards; never leave it in the repo.

## Verification
- `lua-language-server --check` reports `no problems found`.
- The forced-diagnostics probe run reports zero warnings for the new API.
- `python3 -c "import json; json.load(open('.luarc.json'))"` passes (strict JSON).
- `front.code-workspace` is JSONC: trailing commas are legal for VS Code, so a strict JSON parser
  failing on it is not a defect. Do not "fix" pre-existing trailing commas there.

## Pitfalls
- `---@return` with the type on a following `--|` line raises `luadoc-miss-type-name`. Put the whole
  union on the `---@return` line: `---@return "Int"|"String"|nil`.
- `--checklevel` controls log verbosity, not diagnostic filtering: `--checklevel=Error` still prints
  `Warning`s. Always read the full output.
- `runtime.version` is `Lua 5.4` even though `mlua` builds with the `lua55` feature. 5.4 is the
  closest version LuaLS accepts without false diagnostics; keep the two documented together.
- Vendored libraries produce false positives. `src/lib/lua/usr/lib/json.lua` therefore starts with
  `---@diagnostic disable: need-check-nil, cast-local-type`; prefer that over disabling a rule
  globally in `.luarc.json`.
- VS Code editor settings override `.luarc.json`. If the two disagree, keep both
  `front.code-workspace` (`Lua.*`) and `.luarc.json` at the same values.
- A missing `require` in a Lua script (e.g. `json` used without `local json = require("json")`) shows
  up as `undefined-global` in this check. That class of finding is a real runtime bug, not noise.
