#!/usr/bin/env node

import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const [versionArgument, distArgument = "dist"] = process.argv.slice(2);
if (!versionArgument) {
  throw new Error("usage: generate-scoop-manifest.mjs <version> [dist-dir]");
}

const version = versionArgument.replace(/^v/, "");
if (!/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(version)) {
  throw new Error(`invalid semantic version: ${versionArgument}`);
}

const distDirectory = path.resolve(distArgument);
const releaseBase = `https://github.com/IchenDEV/doubao-skin/releases/download/v${version}`;

function asset(architecture, label) {
  const filename = `doubao-skin-cli-Windows-${label}.zip`;
  const checksum = readFileSync(
    path.join(distDirectory, `${filename}.sha256`),
    "utf8",
  ).trim().split(/\s+/)[0];
  if (!/^[a-f0-9]{64}$/.test(checksum)) {
    throw new Error(`invalid checksum for ${filename}`);
  }
  return [architecture, {
    url: `${releaseBase}/${filename}`,
    hash: checksum,
    bin: "doubao-skin.exe",
  }];
}

const manifest = {
  version,
  description: "Theme authoring and automation CLI for Doubao and Doubao Work",
  homepage: "https://doubao-skin.idevlab.dev/guide#cli",
  license: "MIT",
  architecture: Object.fromEntries([
    asset("64bit", "x64"),
    asset("32bit", "x86"),
    asset("arm64", "arm64"),
  ]),
};

const output = path.join(distDirectory, "doubao-skin.json");
writeFileSync(output, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`Generated ${output}`);
