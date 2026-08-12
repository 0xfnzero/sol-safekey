const { spawnSync } = require("node:child_process");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const uiRoot = path.resolve(__dirname, "..");
const genericBoundaryFiles = [
  path.join("src", "app", "[locale]", "page.tsx"),
  path.join("src", "components", "FieldHelp.tsx"),
  path.join("src", "lib", "apiPath.ts"),
  path.join("src", "lib", "apiPath.test.ts"),
  path.join("src", "lib", "appStorage.ts"),
  path.join("src", "lib", "appStorage.test.ts"),
  path.join("src", "lib", "programDeploy.ts"),
  path.join("src", "lib", "programDeploy.test.ts"),
  path.join("src", "lib", "programWorkspace.ts"),
  path.join("src", "lib", "programWorkspace.test.ts"),
  path.join("src", "messages", "en.json"),
  path.join("src", "messages", "zh.json"),
];
const genericBoundarySources = genericBoundaryFiles.map((relativePath) => [
  relativePath,
  fs.readFileSync(path.join(uiRoot, relativePath), "utf8"),
]);
const deploymentPage = genericBoundarySources[0][1];
const fieldHelpComponent = genericBoundarySources[1][1];
const projectSpecificName = ["fn", "zero"].join("");
const projectSpecificNamePattern = new RegExp(projectSpecificName, "i");

for (const [relativePath, source] of genericBoundarySources) {
  assert.doesNotMatch(
    source,
    projectSpecificNamePattern,
    `${relativePath} must remain project-agnostic`,
  );
}

assert.match(deploymentPage, /apiFetch\("program\/deploy"/);
assert.match(deploymentPage, /buildProgramDeploymentReceiptJson\(/);
assert.match(deploymentPage, /const programSoBase64 = btoa\(binary\);/);
assert.match(deploymentPage, /programSoBase64,\s*programSoName:/);
assert.doesNotMatch(deploymentPage, /programSoBase64:\s*btoa\(binary\)/);
assert.doesNotMatch(deploymentPage, /devnet\/deploy/i);
assert.match(fieldHelpComponent, /CircleHelp/);
assert.match(fieldHelpComponent, /Tooltip\.Trigger/);
assert.match(fieldHelpComponent, /Tooltip\.Portal/);
assert.match(fieldHelpComponent, /closeOnClick=\{false\}/);
assert.match(fieldHelpComponent, /data-field-help/);
assert.match(fieldHelpComponent, /aria-describedby=\{tooltipId\}/);
assert.match(fieldHelpComponent, /role="tooltip"/);

const tooltipKeys = [
  "approvedProgramSha256Tooltip",
  "deploymentWalletTooltip",
  "derivedProgramIdTooltip",
  "expectedGenesisHashTooltip",
  "expectedProgramIdTooltip",
  "expectedUpgradeAuthorityTooltip",
  "journalTooltip",
  "maxDataLenTooltip",
  "programFileTooltip",
  "programKeypairFileTooltip",
  "resumeBufferAddressTooltip",
  "uploadedProgramSha256Tooltip",
  "walletPasswordTooltip",
];
for (const locale of ["en", "zh"]) {
  const messages = JSON.parse(
    fs.readFileSync(path.join(uiRoot, "src", "messages", `${locale}.json`), "utf8"),
  );
  const deploymentMessages = messages.features?.["program-deploy"];
  assert.equal(typeof deploymentMessages?.helpAriaLabel, "string");
  assert.ok(deploymentMessages.helpAriaLabel.includes("{field}"));
  for (const key of tooltipKeys) {
    assert.equal(typeof deploymentMessages?.[key], "string", `${locale}.${key} is required`);
    assert.ok(deploymentMessages[key].trim().length > 0, `${locale}.${key} must not be empty`);
  }
}
for (const key of tooltipKeys) {
  assert.ok(
    deploymentPage.includes(`features.program-deploy.${key}`),
    `deployment UI must render ${key}`,
  );
}

const outputDirectory = fs.mkdtempSync(
  path.join(os.tmpdir(), "sol-safekey-program-deploy-"),
);

try {
  const compiler = path.join(uiRoot, "node_modules", "typescript", "bin", "tsc");
  const libDirectory = path.join(uiRoot, "src", "lib");
  const testSources = fs
    .readdirSync(libDirectory)
    .filter((name) => name.endsWith(".test.ts"))
    .map((name) => path.join(libDirectory, name));
  const sources = [
    ...new Set([
      path.join(libDirectory, "appStorage.ts"),
      path.join(libDirectory, "programDeploy.ts"),
      ...testSources,
    ]),
  ];
  const compile = spawnSync(
    process.execPath,
    [
      compiler,
      ...sources,
      "--outDir",
      outputDirectory,
      "--module",
      "commonjs",
      "--moduleResolution",
      "node",
      "--target",
      "ES2020",
      "--esModuleInterop",
      "--skipLibCheck",
    ],
    { cwd: uiRoot, encoding: "utf8" },
  );
  if (compile.status !== 0) {
    process.stderr.write(compile.stdout || "");
    process.stderr.write(compile.stderr || "");
    process.exitCode = compile.status || 1;
  } else {
    const compiledTests = testSources.map((source) =>
      path.join(outputDirectory, path.basename(source, ".ts") + ".js"),
    );
    const run = spawnSync(process.execPath, ["--test", ...compiledTests], {
      cwd: uiRoot,
      encoding: "utf8",
      stdio: "inherit",
    });
    process.exitCode = run.status ?? 1;
  }
} finally {
  fs.rmSync(outputDirectory, { recursive: true, force: true });
}
