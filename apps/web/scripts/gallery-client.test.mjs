import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const source = readFileSync(
  new URL("../src/components/GalleryClient.tsx", import.meta.url),
  "utf8",
);

test("the gallery search icon uses explicit SVG path commands", () => {
  assert.match(source, /d="M20 20 L16 16"/);
});
