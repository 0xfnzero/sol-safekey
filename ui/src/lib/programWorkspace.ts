import {
  type AppNetwork,
  type ProgramDeploymentPlanKind,
  type ProgramDeploymentPlanStatus,
  type ProgramDeploymentResult,
} from "./appStorage";

type ProgramWorkspaceFormState = Record<string, string | number | undefined>;

export function safeFilename(value: string): string {
  return value.trim().replace(/[^a-zA-Z0-9._-]+/g, "-").replace(/^-+|-+$/g, "") || "wallet";
}

export function stableLocalId(value: string): string {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(36);
}

export function sourceDirProjectName(sourceDir: string): string {
  const normalized = sourceDir.trim().replace(/[\\/]+$/, "");
  return normalized.split(/[\\/]/).pop() || normalized || "program";
}

export function programProjectId(sourceDir: string): string {
  return `project:${stableLocalId(sourceDir.trim().toLowerCase())}`;
}

export function programPlanId(
  projectId: string,
  kind: ProgramDeploymentPlanKind,
  network: AppNetwork,
  programId?: string,
  multisig?: string,
): string {
  return [
    "plan",
    stableLocalId(projectId),
    kind,
    network,
    stableLocalId(String(programId || "")),
    stableLocalId(String(multisig || "")),
  ].join(":");
}

export function isUnfinishedProgramDeploymentStatus(status: ProgramDeploymentPlanStatus | undefined): boolean {
  return status === "running" || status === "buffer-ready" || status === "failed";
}

export function deploymentReceiptFilename(programId: unknown): string {
  const normalized = String(programId || "program").trim();
  const shortId =
    normalized.length > 12
      ? `${normalized.slice(0, 8)}-${normalized.slice(-4)}`
      : normalized || "program";
  return `deploy-${safeFilename(shortId)}.json`;
}

export function deploymentResultToFormState(
  result: ProgramDeploymentResult | undefined,
  t: (key: string, vars?: Record<string, string | number>) => string,
): ProgramWorkspaceFormState {
  if (!result) return {};
  return {
    programId: result.programId,
    programdataAddress: result.programdataAddress,
    bufferAddress: result.bufferAddress,
    authority: result.authority,
    signature: result.deploySignature || undefined,
    writeCount: result.writeCount === undefined ? undefined : String(result.writeCount),
    skippedWriteCount: result.skippedWriteCount === undefined ? undefined : String(result.skippedWriteCount),
    rentLamports: result.rentLamports === undefined ? undefined : String(result.rentLamports),
    estimatedFeesLamports: result.estimatedFeesLamports === undefined ? undefined : String(result.estimatedFeesLamports),
    feeRateReserveLamports: result.feeRateReserveLamports === undefined ? undefined : String(result.feeRateReserveLamports),
    recoveryWriteReserveLamports: result.recoveryWriteReserveLamports === undefined ? undefined : String(result.recoveryWriteReserveLamports),
    totalFeeBudgetLamports: result.totalFeeBudgetLamports === undefined ? undefined : String(result.totalFeeBudgetLamports),
    estimatedRequiredBalanceLamports: result.estimatedRequiredBalanceLamports === undefined ? undefined : String(result.estimatedRequiredBalanceLamports),
    createBufferSignature: result.createBufferSignature || undefined,
    programBytes: result.programBytes === undefined ? undefined : String(result.programBytes),
    programSha256: result.programSha256,
    genesisHash: result.genesisHash,
    deployedSlot: result.deployedSlot === undefined ? undefined : String(result.deployedSlot),
    finalizedSlot: result.finalizedSlot === undefined ? undefined : String(result.finalizedSlot),
    readbackVerified: result.readbackVerified === undefined
      ? undefined
      : result.readbackVerified
        ? t("features.program-deploy.readbackPassed")
        : t("features.program-deploy.readbackFailed"),
    deploymentReceiptJson: result.receiptJson,
    network: result.network,
  };
}
