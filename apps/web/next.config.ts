import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Repository-wide agent guidance lives at the root; avoid generated duplicates.
  agentRules: false,
  // The SQLite database is read at build time; include it in traces so any
  // future server-rendered route keeps working on Vercel.
  outputFileTracingIncludes: {
    "/*": ["./data/themes.db"],
  },
};

export default nextConfig;
