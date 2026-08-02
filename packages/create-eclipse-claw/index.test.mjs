import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  addToClaudeDesktop,
  addToCodex,
  httpsUrl,
  parseExpectedChecksum,
  validateArchiveEntries,
  verifyChecksum,
} from "./index.mjs";

test("checksum manifest matches the exact release asset", () => {
  const digest = "a".repeat(64);
  const manifest = `${digest}  eclipse-claw-v0.4.2-x86_64-unknown-linux-gnu.tar.gz\n`;

  assert.equal(
    parseExpectedChecksum(
      manifest,
      "eclipse-claw-v0.4.2-x86_64-unknown-linux-gnu.tar.gz",
    ),
    digest,
  );
  assert.throws(
    () =>
      parseExpectedChecksum(
        manifest,
        "eclipse-claw-v0.4.2-aarch64-apple-darwin.tar.gz",
      ),
    /does not contain/,
  );
});

test("download URLs never downgrade from HTTPS", () => {
  assert.equal(
    httpsUrl("/asset", "https://github.com/release").protocol,
    "https:",
  );
  assert.throws(() => httpsUrl("http://example.com/archive"), /non-HTTPS/);
});

test("archive paths stay inside the expected release directory", () => {
  const root = "eclipse-claw-v0.4.2-x86_64-unknown-linux-gnu";
  assert.equal(
    validateArchiveEntries([root, `${root}/eclipse-claw-mcp`], root),
    true,
  );
  assert.equal(validateArchiveEntries([`${root}/../outside`], root), false);
  assert.equal(validateArchiveEntries(["/absolute/path"], root), false);
  assert.equal(validateArchiveEntries(["other/eclipsed-claw-mcp"], root), false);
});

test("archive checksum comparison is fail-closed", () => {
  const directory = mkdtempSync(join(tmpdir(), "eclipse-claw-checksum-"));
  const archive = join(directory, "archive.tar.gz");
  try {
    writeFileSync(archive, "verified bytes");
    verifyChecksum(
      archive,
      "186287b2d987891f027b4bc8baaf621a3e5a4a73ec78e04b0f65dc309b1ccc03",
    );
    assert.throws(
      () => verifyChecksum(archive, "0".repeat(64)),
      /verification failed/,
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("JSON MCP config uses the literal hyphenated server key", () => {
  const directory = mkdtempSync(join(tmpdir(), "eclipse-claw-json-config-"));
  const configPath = join(directory, "config.json");
  try {
    writeFileSync(configPath, "{}\n");
    addToClaudeDesktop(configPath, null);
    const config = JSON.parse(readFileSync(configPath, "utf-8"));
    assert.deepEqual(Object.keys(config.mcpServers), ["eclipse-claw"]);
    assert.equal(
      readFileSync(`${configPath}.eclipse-claw.bak`, "utf-8"),
      "{}\n",
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("invalid existing JSON is never overwritten", () => {
  const directory = mkdtempSync(join(tmpdir(), "eclipse-claw-invalid-config-"));
  const configPath = join(directory, "config.json");
  try {
    writeFileSync(configPath, "{ invalid json\n");
    assert.throws(() => addToClaudeDesktop(configPath, null), /Refusing to overwrite/);
    assert.equal(readFileSync(configPath, "utf-8"), "{ invalid json\n");
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("Codex TOML config escapes user-provided values", () => {
  const directory = mkdtempSync(join(tmpdir(), "eclipse-claw-toml-config-"));
  const configPath = join(directory, "config.toml");
  const injected = 'token"\n[mcp_servers.attacker]\ncommand = "bad"';
  try {
    addToCodex(configPath, injected);
    addToCodex(configPath, "replacement-token");
    const config = readFileSync(configPath, "utf-8");
    assert.equal((config.match(/^\[mcp_servers\./gm) || []).length, 1);
    assert.match(config, /replacement-token/);
    assert.doesNotMatch(config, /mcp_servers\.attacker/);
    assert.doesNotMatch(config, /^\[mcp_servers\.attacker\]$/m);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
