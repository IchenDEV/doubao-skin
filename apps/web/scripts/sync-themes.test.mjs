import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const source = readFileSync(new URL("./sync-themes.mjs", import.meta.url), "utf8");

test("theme sync preserves an author-provided real-window preview", () => {
  const preserveGuard = source.indexOf(
    "if (fs.existsSync(destination)) return destination;",
  );
  const fallbackRenderer = source.indexOf("const rendered = await sharp(baseLayer)");

  assert.notEqual(preserveGuard, -1, "sync must keep an existing preview image");
  assert.ok(
    preserveGuard < fallbackRenderer,
    "the existing-preview guard must run before the synthetic fallback renderer",
  );
});

test("theme sync preserves a package when only ZIP metadata would change", () => {
  const contentComparison = source.indexOf("archiveContentsMatch(destination, candidate)");
  const nonEmptySignature = source.indexOf("leftSignature.length > 0");
  const candidateRemoval = source.indexOf("fs.rmSync(candidate, { force: true });", contentComparison);
  const destinationRemoval = source.indexOf("fs.rmSync(destination, { force: true });", contentComparison);

  assert.notEqual(contentComparison, -1, "sync must compare logical archive contents");
  assert.notEqual(nonEmptySignature, -1, "an empty or unparseable ZIP listing must not compare equal");
  assert.ok(
    candidateRemoval > contentComparison && candidateRemoval < destinationRemoval,
    "sync must keep the checked-in package when the candidate differs only in ZIP metadata",
  );
});
