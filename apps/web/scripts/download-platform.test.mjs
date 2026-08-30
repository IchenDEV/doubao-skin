import assert from "node:assert/strict";
import test from "node:test";

import { detectDesktopDownload } from "../src/lib/downloads.ts";

test("desktop detection recommends macOS without guessing on iPad", () => {
  assert.equal(
    detectDesktopDownload({
      platform: "MacIntel",
      userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
    }),
    "macos",
  );
  assert.equal(
    detectDesktopDownload({
      platform: "MacIntel",
      userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15)",
      maxTouchPoints: 5,
    }),
    null,
  );
});

test("desktop detection selects Windows architecture when evidence exists", () => {
  assert.equal(
    detectDesktopDownload({ platform: "Windows", architecture: "arm" }),
    "windows-arm64",
  );
  assert.equal(
    detectDesktopDownload({ platform: "Windows", bitness: "32" }),
    "windows-x86",
  );
  assert.equal(
    detectDesktopDownload({
      platform: "Windows",
      userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
    }),
    "windows-x64",
  );
});

test("desktop detection leaves unsupported systems unselected", () => {
  assert.equal(
    detectDesktopDownload({
      platform: "Linux x86_64",
      userAgent: "Mozilla/5.0 (X11; Linux x86_64)",
    }),
    null,
  );
  assert.equal(
    detectDesktopDownload({
      platform: "Linux armv8l",
      userAgent: "Mozilla/5.0 (Linux; Android 16)",
    }),
    null,
  );
});
