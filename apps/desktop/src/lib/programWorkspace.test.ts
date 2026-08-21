import assert from "node:assert/strict";
import test from "node:test";
import {
  deploymentReceiptFilename,
  deploymentResultToFormState,
  isUnfinishedProgramDeploymentStatus,
  programDeploymentHistoryFilename,
  programDeploymentHistoryId,
  programDeploymentHistoryToJson,
  programProjectDeploymentHistoryToJson,
  programPlanId,
  programProjectId,
  safeFilename,
  sourceDirProjectName,
} from "./programWorkspace";
import { type ProgramDeploymentHistoryItem, type ProgramDeploymentResult, type ProgramProject } from "./appStorage";

const t = (key: string) => key;

test("derives stable workspace identifiers and display names", () => {
  assert.equal(sourceDirProjectName("/tmp/example-program/"), "example-program");
  assert.equal(sourceDirProjectName(""), "program");
  assert.equal(programProjectId("/TMP/Example"), programProjectId("/tmp/example"));
  assert.notEqual(
    programProjectId("/tmp/example", "Wallet111"),
    programProjectId("/tmp/example", "Wallet222"),
  );
  assert.equal(
    programProjectId("/TMP/Example", "Wallet111"),
    programProjectId("/tmp/example", "wallet111"),
  );
  assert.equal(
    programPlanId("project:abc", "direct-deploy", "devnet", "Program111", "Multi111"),
    programPlanId("project:abc", "direct-deploy", "devnet", "Program111", "Multi111"),
  );
});

test("builds safe deployment filenames and status predicates", () => {
  assert.equal(safeFilename(" bad/name wallet "), "bad-name-wallet");
  assert.equal(deploymentReceiptFilename("ABCDEFGH1234567890"), "deploy-ABCDEFGH-7890.json");
  assert.equal(deploymentReceiptFilename(""), "deploy-program.json");
  assert.equal(isUnfinishedProgramDeploymentStatus("ready"), true);
  assert.equal(isUnfinishedProgramDeploymentStatus("running"), true);
  assert.equal(isUnfinishedProgramDeploymentStatus("buffer-ready"), true);
  assert.equal(isUnfinishedProgramDeploymentStatus("failed"), true);
  assert.equal(isUnfinishedProgramDeploymentStatus("finalized"), false);
});

test("converts deployment result to form state without leaking optional nulls", () => {
  const result: ProgramDeploymentResult = {
    programId: "Program1111111111111111111111111111111111",
    programdataAddress: "ProgramData111111111111111111111111111",
    bufferAddress: "Buffer111111111111111111111111111111111",
    authority: "Authority1111111111111111111111111111111",
    deploySignature: null,
    createBufferSignature: "CreateSig",
    writeCount: 3,
    skippedWriteCount: 1,
    rentLamports: 10,
    estimatedFeesLamports: 20,
    feeRateReserveLamports: 30,
    recoveryWriteReserveLamports: 40,
    totalFeeBudgetLamports: 50,
    estimatedRequiredBalanceLamports: 60,
    programBytes: 70,
    programSha256: "ab".repeat(32),
    genesisHash: "GenesisHash",
    deployedSlot: 80,
    finalizedSlot: 90,
    readbackVerified: true,
    receiptJson: "{}",
    network: "devnet",
    completedAt: 100,
  };

  const formState = deploymentResultToFormState(result, t);
  assert.equal(formState.programId, result.programId);
  assert.equal(formState.signature, undefined);
  assert.equal(formState.writeCount, "3");
  assert.equal(formState.readbackVerified, "features.program-deploy.readbackPassed");
  assert.equal(formState.network, "devnet");
});

test("exports generic deployment history records", () => {
  const projectId = programProjectId("/tmp/example-program");
  const history: ProgramDeploymentHistoryItem = {
    id: programDeploymentHistoryId(
      projectId,
      "direct-deploy",
      "devnet",
      1_700_000_000_000,
      "Program1111111111111111111111111111111111",
      "Signature111",
    ),
    projectId,
    kind: "direct-deploy",
    status: "finalized",
    network: "devnet",
    sourceDir: "/tmp/example-program",
    programId: "Program1111111111111111111111111111111111",
    programSha256: "cd".repeat(32),
    programBytes: 1024,
    deploySignature: "Signature111",
    createdAt: 1_700_000_000_000,
    completedAt: 1_700_000_000_500,
  };
  const filename = programDeploymentHistoryFilename(history);
  assert.equal(filename, "deploy-Program1-1111-20231114-221320.json");

  const record = JSON.parse(programDeploymentHistoryToJson(history));
  assert.equal(record.schema, "fnzero-safe_program_deployment_history");
  assert.equal(record.record.kind, "direct-deploy");
  assert.equal(record.record.programSha256, history.programSha256);

  const project: ProgramProject = {
    id: projectId,
    name: "example-program",
    sourceDir: "/tmp/example-program",
    network: "devnet",
    programId: history.programId,
    updatedAt: 1_700_000_000_500,
    plans: [],
    history: [history],
  };
  const exportedProject = JSON.parse(programProjectDeploymentHistoryToJson(project));
  assert.equal(exportedProject.schema, "fnzero-safe_program_project_history");
  assert.equal(exportedProject.history.length, 1);
  assert.equal(exportedProject.project.programId, history.programId);
});
