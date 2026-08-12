import assert from "node:assert/strict";
import test from "node:test";
import {
  deploymentReceiptFilename,
  deploymentResultToFormState,
  isUnfinishedProgramDeploymentStatus,
  programPlanId,
  programProjectId,
  safeFilename,
  sourceDirProjectName,
} from "./programWorkspace";
import { type ProgramDeploymentResult } from "./appStorage";

const t = (key: string) => key;

test("derives stable workspace identifiers and display names", () => {
  assert.equal(sourceDirProjectName("/tmp/example-program/"), "example-program");
  assert.equal(sourceDirProjectName(""), "program");
  assert.equal(programProjectId("/TMP/Example"), programProjectId("/tmp/example"));
  assert.equal(
    programPlanId("project:abc", "direct-deploy", "devnet", "Program111", "Multi111"),
    programPlanId("project:abc", "direct-deploy", "devnet", "Program111", "Multi111"),
  );
});

test("builds safe deployment filenames and status predicates", () => {
  assert.equal(safeFilename(" bad/name wallet "), "bad-name-wallet");
  assert.equal(deploymentReceiptFilename("ABCDEFGH1234567890"), "deploy-ABCDEFGH-7890.json");
  assert.equal(deploymentReceiptFilename(""), "deploy-program.json");
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
