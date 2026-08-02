#!/usr/bin/env node

import { readFileSync } from "node:fs";

const dockerfiles = ["Dockerfile", "Dockerfile.ci"];
const digestPattern = /@sha256:[a-f0-9]{64}$/;
const allowedDynamicStages = new Set(["${RUNTIME_VARIANT}"]);
const failures = [];

for (const path of dockerfiles) {
  const stages = new Set();
  const lines = readFileSync(path, "utf-8").split(/\r?\n/);

  for (const [index, raw] of lines.entries()) {
    const line = raw.trim();
    if (!line.startsWith("FROM ")) continue;

    const match = line.match(/^FROM\s+(?:--platform=\S+\s+)?(\S+)(?:\s+AS\s+(\S+))?$/i);
    if (!match) {
      failures.push(`${path}:${index + 1}: cannot parse FROM instruction`);
      continue;
    }

    const [, image, stage] = match;
    const internal = stages.has(image) || allowedDynamicStages.has(image);
    if (!internal && !digestPattern.test(image)) {
      failures.push(`${path}:${index + 1}: external image is not pinned by sha256: ${image}`);
    }

    if (stage) stages.add(stage);
  }
}

const compose = readFileSync("docker-compose.yml", "utf-8");
if (!compose.includes("image: ${OLLAMA_IMAGE:?Set OLLAMA_IMAGE to a pinned ollama/ollama@sha256 digest}")) {
  failures.push("docker-compose.yml: OLLAMA_IMAGE must remain an explicit required digest");
}

if (failures.length > 0) {
  console.error("Container supply-chain contract failed:\n" + failures.map((item) => `- ${item}`).join("\n"));
  process.exit(1);
}

console.log("Container supply-chain contract passed: all external base images require immutable digests.");
