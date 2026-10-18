// Fails when the English and Portuguese mobile catalogs have different keys.
import en from "../src/i18n/locales/en/common.json";
import pt from "../src/i18n/locales/pt/common.json";

const enKeys = new Set(Object.keys(en));
const ptKeys = new Set(Object.keys(pt));
const missing = [...enKeys].filter((key) => !ptKeys.has(key));
const extra = [...ptKeys].filter((key) => !enKeys.has(key));

for (const key of missing) console.error(`missing in pt: ${key}`);
for (const key of extra) console.error(`only in pt: ${key}`);
if (missing.length || extra.length) process.exit(1);
console.log(`i18n ok: ${enKeys.size} keys`);
