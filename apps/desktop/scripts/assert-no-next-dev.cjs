const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");

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
        const cwd = execSync(`lsof -a -d cwd -p ${pid} -Fn`, { encoding: "utf8" })
          .split("\n")
          .find((line) => line.startsWith("n"))
          ?.slice(1);
        return cwd ? fs.realpathSync(cwd) : "";
      } catch {
        return "";
      }
    }
    return "";
  }
}

function isThisProject(pid, command) {
  const cwd = processCwd(pid);
  return cwd === root || command.includes(`${root}${path.sep}`) || command.includes(`cd ${root}`);
}

if (process.platform === "win32") {
  process.exit(0);
}

let pids = [];
try {
  const out = execSync("pgrep -f 'next dev|next-server'", { encoding: "utf8" }).trim();
  pids = out ? out.split(/\n/).filter(Boolean) : [];
} catch {
  process.exit(0);
}

const matches = pids
  .map((pidText) => Number(pidText))
  .filter((pid) => Number.isInteger(pid) && pid !== process.pid)
  .map((pid) => ({ pid, command: commandLine(pid) }))
  .filter(({ pid, command }) => command && isThisProject(pid, command));

if (matches.length === 0) {
  process.exit(0);
}

console.error(
  [
    "[build] next dev is already running for this project.",
    "Stop the dev stack before running npm run build; running them at the same time corrupts Next's dev runtime and causes 500 errors.",
    ...matches.map(({ pid, command }) => `- PID ${pid}: ${command}`),
  ].join("\n"),
);
process.exit(1);
