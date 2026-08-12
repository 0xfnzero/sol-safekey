import { invoke } from "@tauri-apps/api/core";
import { isSafeUrlHostname } from "@/lib/appStorage";

function isTauriWebview(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Open a URL in the system default browser (Tauri) or a new tab (web). */
export async function openExternalUrl(url: string): Promise<void> {
  const target = url.trim();
  let parsed: URL;
  try {
    parsed = new URL(target);
  } catch {
    throw new Error("invalid external URL");
  }
  if (
    parsed.protocol !== "https:" ||
    !isSafeUrlHostname(parsed.hostname) ||
    parsed.username ||
    parsed.password
  ) {
    throw new Error("only https URLs can be opened externally");
  }

  if (isTauriWebview()) {
    await invoke("open_external_url", { url: target });
    return;
  }

  const opened = window.open(target, "_blank", "noopener,noreferrer");
  if (!opened) {
    throw new Error("failed to open external browser");
  }
}
