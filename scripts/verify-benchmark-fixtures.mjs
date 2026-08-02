import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const directory = path.join(root, 'benchmarks', 'public-pages');
const manifest = JSON.parse(await readFile(path.join(directory, 'manifest.json'), 'utf8'));

if (manifest.schema_version !== 1 || !Array.isArray(manifest.cases) || manifest.cases.length < 4) {
  throw new Error('invalid fixed public-page benchmark manifest');
}

const seen = new Set();
for (const item of manifest.cases) {
  if (!item.id || seen.has(item.id)) throw new Error(`duplicate or empty benchmark id: ${item.id}`);
  seen.add(item.id);
  if (!/^https:\/\//.test(item.source_url)) throw new Error(`${item.id}: source_url must use https`);
  if (!/^\d{4}-\d{2}-\d{2}$/.test(item.captured_at)) throw new Error(`${item.id}: invalid captured_at`);
  if (!/^[a-f0-9]{64}$/.test(item.sha256)) throw new Error(`${item.id}: invalid sha256`);
  const bytes = await readFile(path.join(directory, item.fixture));
  const actual = createHash('sha256').update(bytes).digest('hex');
  if (actual !== item.sha256) throw new Error(`${item.id}: fixture checksum mismatch`);
}

console.log(`Fixed public-page fixtures verified: ${manifest.cases.length} cases.`);
