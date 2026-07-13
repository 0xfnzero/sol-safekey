/**
 * Free DEFAULT_API_PORT (3841) before `cargo run`, so `desktop:dev` does not fail with
 * EADDRINUSE when a previous sol-safekey-ui is still running.
 *
 * macOS / Linux: lsof. Windows: no-op (run Task Manager or `netstat` manually if needed).
 */
const { execSync } = require("child_process");

const port = process.env.SOL_SAFEKEY_API_PORT || "3841";

if (process.platform === "win32") {
  process.exit(0);
}

try {
  const out = execSync(
    `lsof -nP -iTCP:${port} -sTCP:LISTEN -t 2>/dev/null`,
    { encoding: "utf8" },
  ).trim();
  if (!out) process.exit(0);

  const pids = [...new Set(out.split(/\n/).filter(Boolean))];
  console.log(`[dev:stack] port ${port} is in use — stopping PID(s): ${pids.join(", ")}`);
  for (const pid of pids) {
    try {
      process.kill(Number(pid), "SIGKILL");
    } catch (_) {
      /* ignore */
    }
  }
} catch {
  /* no listener */
}
