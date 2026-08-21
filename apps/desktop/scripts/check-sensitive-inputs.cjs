const fs = require("fs");
const path = require("path");

const ROOT = path.resolve(__dirname, "..");
const SRC = path.join(ROOT, "src");
const SENSITIVE_FIELDS = new Set([
  "password",
  "current_password",
  "new_password",
  "master_password",
  "private_key",
  "secret_key",
  "keystore_json",
  "program_keypair_json",
  "programKeypairJson",
  "security_answer",
  "totp_code",
]);

function walk(dir) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  return entries.flatMap((entry) => {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) return walk(fullPath);
    if (/\.(tsx|jsx|ts|js)$/.test(entry.name)) return [fullPath];
    return [];
  });
}

function lineNumber(source, index) {
  return source.slice(0, index).split("\n").length;
}

function readInputTag(source, startIndex) {
  let quote = null;
  let braceDepth = 0;

  for (let i = startIndex; i < source.length; i += 1) {
    const char = source[i];
    const next = source[i + 1];

    if (quote) {
      if (char === "\\" && (quote === '"' || quote === "'")) {
        i += 1;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }

    if (char === '"' || char === "'" || char === "`") {
      quote = char;
      continue;
    }

    if (char === "{") braceDepth += 1;
    if (char === "}") braceDepth = Math.max(0, braceDepth - 1);
    if (char === ">" && braceDepth === 0) {
      return source.slice(startIndex, i + 1);
    }

    if (char === "/" && next === ">") {
      return source.slice(startIndex, i + 2);
    }
  }

  return source.slice(startIndex);
}

const failures = [];

for (const file of walk(SRC)) {
  const source = fs.readFileSync(file, "utf8");
  const forbiddenProgramKeypairState = [
    /\bformData\.(?:program_keypair_json|programKeypairJson)\b/g,
    /handleFormChange\(\s*["'](?:program_keypair_json|programKeypairJson)["']/g,
    /\bprogramKeypairJson\b/g,
  ];
  for (const pattern of forbiddenProgramKeypairState) {
    let secretMatch;
    while ((secretMatch = pattern.exec(source))) {
      failures.push(
        `${path.relative(ROOT, file)}:${lineNumber(source, secretMatch.index)} program keypair JSON must not be stored in FormState`,
      );
    }
  }

  const storageMatch = source.match(
    /(?:localStorage|sessionStorage)\.setItem\([^)]*program[_-]?keypair/i,
  );
  if (storageMatch?.index !== undefined) {
    failures.push(
      `${path.relative(ROOT, file)}:${lineNumber(source, storageMatch.index)} program keypair JSON must not be persisted in browser storage`,
    );
  }

  const inputRegex = /<input\b/g;
  let match;

  while ((match = inputRegex.exec(source))) {
    const tag = readInputTag(source, match.index);
    const fieldMatch = tag.match(/handleFormChange\(\s*["']([^"']+)["']/);
    const valueMatch = tag.match(/value=\{\s*formData\.([A-Za-z0-9_]+)/);
    const annotatedFieldMatch = tag.match(/data-sensitive-field=\s*["']([^"']+)["']/);
    const fieldName = fieldMatch?.[1] || valueMatch?.[1] || annotatedFieldMatch?.[1];

    if (!fieldName || !SENSITIVE_FIELDS.has(fieldName)) continue;
    if (/\btype=\s*["']password["']/.test(tag)) continue;

    failures.push(
      `${path.relative(ROOT, file)}:${lineNumber(source, match.index)} sensitive field "${fieldName}" must use type="password"`,
    );
  }
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}
