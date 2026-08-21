/* Prepare the web/API dev stack without corrupting Next dev's .next runtime. */
const fs = require("fs");
const path = require("path");
const { execSync, spawnSync } = require("child_process");

const root = path.resolve(__dirname, "..");
const outIndex = path.join(root, "out", "index.html");
const nextDir = path.join(root, ".next");

function commandLine(pid) {
  try {
    return execSync(`ps -p ${pid} -o command=`, { encoding: "utf8" }).trim();
  } catch {
    return "";
  }
}

function processCwd(pid) {
  try {
    return fs.realpathSync(`/proc/${pid}/cwd`);
  } catch {
    if (process.platform === "darwin") {
      try {
        return fs.realpathSync(execSync(`lsof -a -d cwd -p ${pid} -Fn`, { encoding: "utf8" })
          .split("\n")
          .find((line) => line.startsWith("n"))
          ?.slice(1) || "");
      } catch {
        return "";
      }
    }
    return "";
  }
}

function isRunning(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

function waitForExit(pids) {
  const deadline = Date.now() + 3000;
  while (pids.some(isRunning) && Date.now() < deadline) {
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 100);
  }

  for (const pid of pids) {
    if (!isRunning(pid)) continue;
    console.log(`[dev:stack] force stopping stale process ${pid}`);
    try {
      process.kill(pid, "SIGKILL");
    } catch {
      /* ignore */
    }
  }
}

function stopStaleNextProcesses() {
  if (process.platform === "win32") return;

  let pids = [];
  try {
    const out = execSync("pgrep -f 'next (dev|build)|next-server'", { encoding: "utf8" }).trim();
    pids = out ? out.split(/\n/).filter(Boolean) : [];
  } catch {
    return;
  }

  const stopped = [];
  for (const pidText of pids) {
    const pid = Number(pidText);
    if (!Number.isInteger(pid) || pid === process.pid) continue;

    const cwd = processCwd(pid);
    const command = commandLine(pid);
    const isThisProject =
      cwd === root ||
      command.includes(`${root}${path.sep}`) ||
      command.includes(`cd ${root}`);

    if (!isThisProject) continue;

    console.log(`[dev:stack] stopping stale Next process ${pid}: ${command}`);
    try {
      process.kill(pid, "SIGTERM");
      stopped.push(pid);
    } catch {
      /* ignore */
    }
  }
  waitForExit(stopped);
}

function removeNextDevCache() {
  if (fs.existsSync(nextDir)) {
    console.log("[dev:stack] removing stale .next dev cache");
    fs.rmSync(nextDir, { recursive: true, force: true });
  }
}

function ensureStaticOutExists() {
  if (fs.existsSync(outIndex)) return;

  console.log("[dev:stack] out/ missing - running npm run build once for Rust embedded assets");
  const result = spawnSync("npm", ["run", "build"], {
    cwd: root,
    stdio: "inherit",
    shell: process.platform === "win32",
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }

  removeNextDevCache();
}

stopStaleNextProcesses();
removeNextDevCache();
ensureStaticOutExists();
