#!/usr/bin/env node

import {
  copyFileSync,
  constants,
  createWriteStream,
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "fs";
import { createInterface } from "readline";
import { homedir, platform, arch } from "os";
import { join, dirname } from "path";
import { pathToFileURL } from "url";
import { execFileSync } from "child_process";
import { createHash, timingSafeEqual } from "crypto";
import { chmod } from "fs/promises";
import https from "https";

// ── Constants ──

const REPO = "PavelHopson/eclipse-claw";
const BINARY_NAME =
  platform() === "win32" ? "eclipse-claw-mcp.exe" : "eclipse-claw-mcp";
const INSTALL_DIR = join(homedir(), ".eclipse-claw");
const BINARY_PATH = join(INSTALL_DIR, BINARY_NAME);
const MAX_REDIRECTS = 5;
const MAX_METADATA_BYTES = 2 * 1024 * 1024;
const MAX_ARCHIVE_BYTES = 256 * 1024 * 1024;

const COLORS = {
  reset: "\x1b[0m",
  bold: "\x1b[1m",
  dim: "\x1b[2m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  cyan: "\x1b[36m",
  red: "\x1b[31m",
};

const c = (color, text) => `${COLORS[color]}${text}${COLORS.reset}`;

// ── AI Tool Detection ──

const AI_TOOLS = [
  {
    id: "claude-desktop",
    name: "Claude Desktop",
    detect: () => {
      if (platform() === "darwin")
        return existsSync(
          join(
            homedir(),
            "Library/Application Support/Claude/claude_desktop_config.json",
          ),
        );
      if (platform() === "win32")
        return existsSync(
          join(process.env.APPDATA || "", "Claude/claude_desktop_config.json"),
        );
      return false;
    },
    configPath: () => {
      if (platform() === "darwin")
        return join(
          homedir(),
          "Library/Application Support/Claude/claude_desktop_config.json",
        );
      if (platform() === "win32")
        return join(
          process.env.APPDATA || "",
          "Claude/claude_desktop_config.json",
        );
      return null;
    },
  },
  {
    id: "claude-code",
    name: "Claude Code",
    detect: () => existsSync(join(homedir(), ".claude.json")),
    configPath: () => join(homedir(), ".claude.json"),
  },
  {
    id: "cursor",
    name: "Cursor",
    detect: () => {
      // Check for .cursor directory in home or current project
      return (
        existsSync(join(homedir(), ".cursor")) ||
        existsSync(join(process.cwd(), ".cursor"))
      );
    },
    configPath: () => {
      const projectPath = join(process.cwd(), ".cursor", "mcp.json");
      const globalPath = join(homedir(), ".cursor", "mcp.json");
      return existsSync(join(process.cwd(), ".cursor"))
        ? projectPath
        : globalPath;
    },
  },
  {
    id: "windsurf",
    name: "Windsurf",
    detect: () => {
      return (
        existsSync(join(homedir(), ".codeium")) ||
        existsSync(join(homedir(), ".windsurf"))
      );
    },
    configPath: () =>
      join(homedir(), ".codeium", "windsurf", "mcp_config.json"),
  },
  {
    id: "vscode-continue",
    name: "VS Code (Continue)",
    detect: () => existsSync(join(homedir(), ".continue")),
    configPath: () => join(homedir(), ".continue", "config.json"),
  },
  {
    id: "opencode",
    name: "OpenCode",
    detect: () => {
      return (
        existsSync(join(homedir(), ".config", "opencode", "opencode.json")) ||
        existsSync(join(process.cwd(), "opencode.json"))
      );
    },
    configPath: () => {
      const projectPath = join(process.cwd(), "opencode.json");
      const globalPath = join(
        homedir(),
        ".config",
        "opencode",
        "opencode.json",
      );
      return existsSync(projectPath) ? projectPath : globalPath;
    },
  },
  {
    id: "antigravity",
    name: "Antigravity",
    detect: () => {
      return (
        existsSync(join(homedir(), ".antigravity")) ||
        existsSync(join(homedir(), ".config", "antigravity"))
      );
    },
    configPath: () => {
      const configDir = existsSync(join(homedir(), ".config", "antigravity"))
        ? join(homedir(), ".config", "antigravity")
        : join(homedir(), ".antigravity");
      return join(configDir, "mcp.json");
    },
  },
  {
    id: "codex",
    name: "Codex (CLI + App)",
    detect: () => existsSync(join(homedir(), ".codex")),
    configPath: () => join(homedir(), ".codex", "config.toml"),
  },
];

// ── Helpers ──

function ask(question) {
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
  });
  return new Promise((resolve) => {
    rl.question(question, (answer) => {
      rl.close();
      resolve(answer.trim());
    });
  });
}

function httpsUrl(url, base) {
  const parsed = new URL(url, base);
  if (parsed.protocol !== "https:") {
    throw new Error(`Refusing non-HTTPS download: ${parsed.href}`);
  }
  return parsed;
}

function download(url, redirects = 0) {
  return new Promise((resolve, reject) => {
    if (redirects > MAX_REDIRECTS) {
      reject(new Error("Too many download redirects"));
      return;
    }

    const parsed = httpsUrl(url);
    https
      .get(
        parsed,
        { headers: { "User-Agent": "create-eclipse-claw" } },
        (res) => {
          // Follow redirects
          if (
            res.statusCode >= 300 &&
            res.statusCode < 400 &&
            res.headers.location
          ) {
            res.resume();
            const next = httpsUrl(res.headers.location, parsed).href;
            return download(next, redirects + 1).then(resolve).catch(reject);
          }
          if (res.statusCode !== 200) {
            res.resume();
            return reject(new Error(`HTTP ${res.statusCode}`));
          }
          const chunks = [];
          let received = 0;
          res.on("data", (chunk) => {
            received += chunk.length;
            if (received > MAX_METADATA_BYTES) {
              res.destroy(new Error("Release metadata exceeds the size limit"));
              return;
            }
            chunks.push(chunk);
          });
          res.on("end", () => resolve(Buffer.concat(chunks)));
          res.on("error", reject);
        },
      )
      .on("error", reject);
  });
}

async function downloadFile(url, dest, redirects = 0) {
  return new Promise((resolve, reject) => {
    if (redirects > MAX_REDIRECTS) {
      reject(new Error("Too many archive redirects"));
      return;
    }

    const parsed = httpsUrl(url);
    https
      .get(
        parsed,
        { headers: { "User-Agent": "create-eclipse-claw" } },
        (res) => {
          if (
            res.statusCode >= 300 &&
            res.statusCode < 400 &&
            res.headers.location
          ) {
            res.resume();
            const next = httpsUrl(res.headers.location, parsed).href;
            return downloadFile(next, dest, redirects + 1)
              .then(resolve)
              .catch(reject);
          }
          if (res.statusCode !== 200) {
            res.resume();
            return reject(new Error(`HTTP ${res.statusCode}`));
          }

          const file = createWriteStream(dest);
          let received = 0;
          const fail = (error) => {
            file.destroy();
            rmSync(dest, { force: true });
            reject(error);
          };

          res.on("data", (chunk) => {
            received += chunk.length;
            if (received > MAX_ARCHIVE_BYTES) {
              res.destroy(new Error("Release archive exceeds the size limit"));
            }
          });
          res.pipe(file);
          file.on("finish", () => {
            file.close(resolve);
          });
          file.on("error", fail);
          res.on("error", fail);
        },
      )
      .on("error", reject);
  });
}

function getReleaseTarget() {
  const os = platform();
  const a = arch();

  if (os === "darwin" && a === "arm64") return "aarch64-apple-darwin";
  if (os === "darwin" && a === "x64") return "x86_64-apple-darwin";
  if (os === "linux" && a === "x64") return "x86_64-unknown-linux-gnu";
  if (os === "linux" && a === "arm64") return "aarch64-unknown-linux-gnu";

  return null;
}

function parseExpectedChecksum(manifest, assetName) {
  for (const line of manifest.split(/\r?\n/)) {
    const match = line.match(/^([a-fA-F0-9]{64})\s+\*?(.+)$/);
    if (match && match[2].trim() === assetName) {
      return match[1].toLowerCase();
    }
  }
  throw new Error(`SHA256SUMS does not contain ${assetName}`);
}

function verifyChecksum(path, expectedHex) {
  const actual = createHash("sha256").update(readFileSync(path)).digest();
  const expected = Buffer.from(expectedHex, "hex");
  if (expected.length !== actual.length || !timingSafeEqual(actual, expected)) {
    throw new Error("Release archive SHA-256 verification failed");
  }
}

function validateArchiveEntries(entries, root) {
  return (
    entries.length > 0 &&
    entries.every((entry) => {
      const segments = entry.split("/");
      return (
        !entry.startsWith("/") &&
        !entry.includes("\\") &&
        !segments.includes("..") &&
        (entry === root || entry.startsWith(`${root}/`))
      );
    })
  );
}

function extractVerifiedArchive(path, releaseTag, target) {
  const root = `eclipse-claw-${releaseTag}-${target}`;
  const binaryEntry = `${root}/eclipse-claw-mcp`;
  const entries = execFileSync("tar", ["tzf", path], {
    encoding: "utf-8",
    maxBuffer: MAX_ARCHIVE_BYTES,
  })
    .split(/\r?\n/)
    .filter(Boolean);

  if (!validateArchiveEntries(entries, root)) {
    throw new Error("Release archive contains an unsafe path");
  }

  if (entries.filter((entry) => entry === binaryEntry).length !== 1) {
    throw new Error(
      "Verified release archive must contain one eclipse-claw-mcp binary",
    );
  }

  const binary = execFileSync("tar", ["xOzf", path, binaryEntry], {
    maxBuffer: MAX_ARCHIVE_BYTES,
  });
  if (binary.length === 0) {
    throw new Error("Verified eclipse-claw-mcp binary is empty");
  }
  writeFileSync(BINARY_PATH, binary, { mode: 0o755 });
}

function readJsonFile(path) {
  if (!existsSync(path)) return {};

  try {
    const text = readFileSync(path, "utf-8");
    return text.trim() ? JSON.parse(text) : {};
  } catch (error) {
    throw new Error(
      `Refusing to overwrite invalid JSON config ${path}: ${error.message}`,
    );
  }
}

function writeTextFileAtomic(path, text) {
  const dir = dirname(path);
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });

  const backup = `${path}.eclipse-claw.bak`;
  if (existsSync(path) && !existsSync(backup)) {
    copyFileSync(path, backup, constants.COPYFILE_EXCL);
  }

  const temp = `${path}.eclipse-claw-${process.pid}.tmp`;
  const mode = existsSync(path) ? statSync(path).mode & 0o777 : 0o600;
  try {
    writeFileSync(temp, text, { mode });
    renameSync(temp, path);
  } finally {
    rmSync(temp, { force: true });
  }
}

function writeJsonFile(path, data) {
  writeTextFileAtomic(path, JSON.stringify(data, null, 2) + "\n");
}

function buildMcpEntry(apiKey) {
  const entry = {
    command: BINARY_PATH,
  };
  if (apiKey) {
    entry.env = { ECLIPSE_CLAW_API_KEY: apiKey };
  }
  return entry;
}

// ── MCP Config Writers ──

function addToClaudeDesktop(configPath, apiKey) {
  const config = readJsonFile(configPath);
  if (!config.mcpServers) config.mcpServers = {};
  config.mcpServers["eclipse-claw"] = buildMcpEntry(apiKey);
  writeJsonFile(configPath, config);
}

function addToClaudeCode(configPath, apiKey) {
  const config = readJsonFile(configPath);
  if (!config.mcpServers) config.mcpServers = {};
  config.mcpServers["eclipse-claw"] = buildMcpEntry(apiKey);
  writeJsonFile(configPath, config);
}

function addToCursor(configPath, apiKey) {
  const config = readJsonFile(configPath);
  if (!config.mcpServers) config.mcpServers = {};
  config.mcpServers["eclipse-claw"] = {
    command: BINARY_PATH,
    ...(apiKey ? { env: { ECLIPSE_CLAW_API_KEY: apiKey } } : {}),
  };
  writeJsonFile(configPath, config);
}

function addToWindsurf(configPath, apiKey) {
  const config = readJsonFile(configPath);
  if (!config.mcpServers) config.mcpServers = {};
  config.mcpServers["eclipse-claw"] = buildMcpEntry(apiKey);
  writeJsonFile(configPath, config);
}

function addToVSCodeContinue(configPath, apiKey) {
  const config = readJsonFile(configPath);
  if (!config.mcpServers) config.mcpServers = [];
  // Continue uses array format
  const existing = config.mcpServers.findIndex?.((s) => s.name === "eclipse-claw");
  const entry = {
    name: "eclipse-claw",
    command: BINARY_PATH,
    ...(apiKey ? { env: { ECLIPSE_CLAW_API_KEY: apiKey } } : {}),
  };
  if (existing >= 0) {
    config.mcpServers[existing] = entry;
  } else if (Array.isArray(config.mcpServers)) {
    config.mcpServers.push(entry);
  }
  writeJsonFile(configPath, config);
}

function addToOpenCode(configPath, apiKey) {
  const config = readJsonFile(configPath);
  if (!config.mcp) config.mcp = {};
  config.mcp["eclipse-claw"] = {
    type: "local",
    command: [BINARY_PATH],
    enabled: true,
  };
  if (apiKey) {
    config.mcp["eclipse-claw"].environment = { ECLIPSE_CLAW_API_KEY: apiKey };
  }
  writeJsonFile(configPath, config);
}

function addToAntigravity(configPath, apiKey) {
  const config = readJsonFile(configPath);
  if (!config.mcpServers) config.mcpServers = {};
  config.mcpServers["eclipse-claw"] = buildMcpEntry(apiKey);
  writeJsonFile(configPath, config);
}

function addToCodex(configPath, apiKey) {
  // Codex uses TOML format, not JSON. Append MCP server config section.
  const dir = dirname(configPath);
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });

  let existing = "";
  try {
    existing = readFileSync(configPath, "utf-8");
  } catch {
    // File doesn't exist yet
  }

  // Remove any existing eclipse-claw MCP section
  existing = existing.replace(
    /^\[mcp_servers\.eclipse-claw\]\r?\n(?:^(?!\[).*(?:\r?\n|$))*/gm,
    "",
  );

  let section = `\n[mcp_servers.eclipse-claw]\ncommand = ${JSON.stringify(BINARY_PATH)}\nargs = []\nenabled = true\n`;
  if (apiKey) {
    section += `env = { ECLIPSE_CLAW_API_KEY = ${JSON.stringify(apiKey)} }\n`;
  }

  writeTextFileAtomic(configPath, existing.trimEnd() + "\n" + section);
}

const CONFIG_WRITERS = {
  "claude-desktop": addToClaudeDesktop,
  "claude-code": addToClaudeCode,
  cursor: addToCursor,
  windsurf: addToWindsurf,
  "vscode-continue": addToVSCodeContinue,
  opencode: addToOpenCode,
  antigravity: addToAntigravity,
  codex: addToCodex,
};

// ── Main ──

async function main() {
  console.log();
  console.log(c("bold", "  ┌─────────────────────────────────────┐"));
  console.log(
    c("bold", "  │") +
      c("cyan", "  eclipse-claw") +
      c("dim", " — MCP setup for AI agents") +
      c("bold", "  │"),
  );
  console.log(c("bold", "  └─────────────────────────────────────┘"));
  console.log();

  // 1. Detect installed AI tools
  console.log(c("bold", "  Detecting AI tools..."));
  console.log();

  const detected = AI_TOOLS.filter((tool) => {
    try {
      return tool.detect();
    } catch {
      return false;
    }
  });

  if (detected.length === 0) {
    console.log(c("yellow", "  No supported AI tools detected."));
    console.log();
    console.log(c("dim", "  Supported tools:"));
    for (const tool of AI_TOOLS) {
      console.log(c("dim", `    • ${tool.name}`));
    }
    console.log();
    console.log(
      c("dim", "  Install one of these tools and run this command again."),
    );
    console.log(c("dim", "  Or use --manual to configure manually."));
    console.log();

    if (process.argv.includes("--manual")) {
      // Continue anyway for manual setup
    } else {
      process.exit(0);
    }
  }

  for (const tool of detected) {
    console.log(c("green", `  ✓ ${tool.name}`));
  }
  console.log();

  if (detected.length > 0) {
    const confirm = await ask(
      c("bold", "  Configure these tools? ") + c("dim", "[Y/n]: "),
    );
    if (["n", "no"].includes(confirm.toLowerCase())) {
      console.log(c("yellow", "\n  No configuration files were changed.\n"));
      process.exit(0);
    }
    console.log();
  }

  // 2. Ask for API key
  console.log(c("dim", "  An API key enables cloud features."));
  console.log(
    c("dim", "  Without one, eclipse-claw runs locally (free, no account needed)."),
  );
  console.log(
    c("dim", "  If entered, the key is stored in each selected local MCP config."),
  );
  console.log();

  const apiKey = await ask(
    c("bold", "  API key ") +
      c("dim", "(press Enter to skip for local-only): "),
  );
  console.log();

  // 3. Download binary
  console.log(c("bold", "  Downloading eclipse-claw-mcp..."));

  if (!existsSync(INSTALL_DIR)) {
    mkdirSync(INSTALL_DIR, { recursive: true });
  }

  let downloaded = false;
  let release;

  try {
    const releaseData = await download(
      `https://api.github.com/repos/${REPO}/releases/latest`,
    );
    release = JSON.parse(releaseData.toString());
  } catch (e) {
    throw new Error(`Could not read the latest GitHub release: ${e.message}`);
  }

  if (!/^v\d+\.\d+\.\d+$/.test(release.tag_name || "")) {
    throw new Error("Latest GitHub release has an invalid version tag");
  }

  const target = getReleaseTarget();
  if (target) {
    const assetName = `eclipse-claw-${release.tag_name}-${target}.tar.gz`;
    const asset = release.assets?.find((item) => item.name === assetName);
    const checksums = release.assets?.find((item) => item.name === "SHA256SUMS");

    if (asset && checksums) {
      const archivePath = join(INSTALL_DIR, assetName);
      try {
        const manifest = await download(checksums.browser_download_url);
        const expected = parseExpectedChecksum(
          manifest.toString("utf-8"),
          assetName,
        );
        await downloadFile(asset.browser_download_url, archivePath);
        verifyChecksum(archivePath, expected);
        extractVerifiedArchive(archivePath, release.tag_name, target);
        await chmod(BINARY_PATH, 0o755);
      } finally {
        rmSync(archivePath, { force: true });
      }

      console.log(
        c("green", `  ✓ Installed verified ${release.tag_name} to ${BINARY_PATH}`),
      );
      downloaded = true;
    }
  }

  if (!downloaded) {
    console.log(
      c(
        "yellow",
        `  No verified pre-built binary for ${platform()}-${arch()}. Trying Cargo...`,
      ),
    );
    try {
      execFileSync(
        "cargo",
        [
          "install",
          "--git",
          `https://github.com/${REPO}`,
          "--tag",
          release.tag_name,
          "--locked",
          "eclipse-claw-mcp",
          "--root",
          INSTALL_DIR,
        ],
        { stdio: "inherit" },
      );
      // cargo install puts binary in INSTALL_DIR/bin/
      const cargoPath = join(INSTALL_DIR, "bin", BINARY_NAME);
      if (existsSync(cargoPath)) {
        copyFileSync(cargoPath, BINARY_PATH);
        await chmod(BINARY_PATH, 0o755);
        console.log(c("green", `  ✓ Built and installed to ${BINARY_PATH}`));
        downloaded = true;
      }
    } catch {
      console.log(
        c("red", "  Failed to install. Make sure Rust is installed:"),
      );
      console.log(
        c(
          "dim",
          `  Install Rust, then retry; Cargo will build the locked ${release.tag_name} source tag.`,
        ),
      );
      process.exit(1);
    }
  }

  console.log();

  // 4. Configure each detected tool
  console.log(c("bold", "  Configuring MCP servers..."));
  console.log();

  for (const tool of detected) {
    const configPath = tool.configPath();
    if (!configPath) continue;

    const writer = CONFIG_WRITERS[tool.id];
    if (!writer) continue;

    try {
      writer(configPath, apiKey || null);
      console.log(
        c("green", `  ✓ ${tool.name}`) + c("dim", ` → ${configPath}`),
      );
    } catch (e) {
      console.log(c("red", `  ✗ ${tool.name}: ${e.message}`));
    }
  }

  console.log();

  // 5. Verify
  if (downloaded) {
    try {
      const version = execFileSync(BINARY_PATH, ["--version"], {
        encoding: "utf-8",
      }).trim();
      console.log(c("green", `  ✓ ${version}`));
    } catch {
      console.log(c("green", `  ✓ eclipse-claw-mcp installed`));
    }
  }

  // 6. Summary
  console.log();
  console.log(c("bold", "  Done! eclipse-claw is ready."));
  console.log();
  console.log(c("dim", "  Your AI agent now has these tools:"));
  console.log(c("dim", "    • scrape — extract content from any URL"));
  console.log(c("dim", "    • crawl  — recursively crawl a website"));
  console.log(c("dim", "    • search — web search + parallel scrape"));
  console.log(c("dim", "    • map    — discover URLs from sitemaps"));
  console.log(c("dim", "    • batch  — extract multiple URLs in parallel"));
  console.log();

  if (!apiKey) {
    console.log(c("yellow", "  Running in local-only mode (no API key)."));
    console.log(
      c(
        "dim",
        "  Get an API key at https://webclaw.io/dashboard for cloud features.",
      ),
    );
    console.log();
  }

  console.log(c("dim", "  Restart your AI tool to activate the MCP server."));
  console.log();
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((e) => {
    console.error(c("red", `\n  Error: ${e.message}\n`));
    process.exit(1);
  });
}

export {
  addToClaudeDesktop,
  addToCodex,
  httpsUrl,
  parseExpectedChecksum,
  validateArchiveEntries,
  verifyChecksum,
};
