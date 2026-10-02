/** Regenerate spec/test-vectors/consent-v1.json (deterministic: fixed key + RFC 6979). */
import { writeFileSync } from "node:fs";
import { buildVector } from "../test/vector.js";

const out = new URL("../../../../spec/test-vectors/consent-v1.json", import.meta.url);
writeFileSync(out, JSON.stringify(buildVector(), null, 2) + "\n");
console.log("wrote", out.pathname);
