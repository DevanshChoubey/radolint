// Runs the extension inside a downloaded VS Code with a throwaway profile.
//   RADOLINT_BIN=/path/to/radolint npm test
const path = require("path");
const fs = require("fs");
const os = require("os");
const { runTests } = require("@vscode/test-electron");

async function main() {
  const bin = process.env.RADOLINT_BIN;
  if (!bin) throw new Error("set RADOLINT_BIN to a radolint binary");
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "radolint-ws-"));
  fs.cpSync(path.join(__dirname, "fixture"), workspace, { recursive: true });
  fs.writeFileSync(path.join(workspace, ".vscode", "settings.json"),
    JSON.stringify({ "radolint.path": path.resolve(bin) }));
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), "radolint-profile-"));
  await runTests({
    extensionDevelopmentPath: path.join(__dirname, ".."),
    extensionTestsPath: path.join(__dirname, "suite.js"),
    launchArgs: [workspace, "--disable-extensions", "--user-data-dir", path.join(profile, "user"),
      "--extensions-dir", path.join(profile, "ext"), "--skip-welcome", "--disable-workspace-trust"],
  });
}

main().catch((err) => { console.error(err); process.exit(1); });
