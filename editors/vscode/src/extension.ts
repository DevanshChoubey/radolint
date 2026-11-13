// SPDX-License-Identifier: GPL-3.0-only
// VS Code client for radolint: runs `radolint -f json -` on each Dockerfile
// buffer and shows the findings as diagnostics, with quick fixes that insert
// hadolint ignore pragmas.

import * as vscode from "vscode";
import { spawn, execFile } from "child_process";
import { createHash } from "crypto";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

const SOURCE = "radolint";
const RELEASE_VERSION = "0.1.1";
const REPO = "DevanshChoubey/radolint";
const DEBOUNCE_MS = 300;

interface Finding {
  code: string;
  column: number;
  file: string;
  level: "error" | "warning" | "info" | "style";
  line: number;
  message: string;
}

let diagnostics: vscode.DiagnosticCollection;
let output: vscode.OutputChannel;
let storageDir: string;
const timers = new Map<string, NodeJS.Timeout>();
const running = new Map<string, number>();
let resolvedBinary: Promise<string | undefined> | undefined;
let warnedMissing = false;

export function activate(context: vscode.ExtensionContext): void {
  diagnostics = vscode.languages.createDiagnosticCollection(SOURCE);
  output = vscode.window.createOutputChannel("radolint");
  storageDir = context.globalStorageUri.fsPath;
  context.subscriptions.push(diagnostics, output);

  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((d) => schedule(d, 0)),
    vscode.workspace.onDidSaveTextDocument((d) => schedule(d, 0)),
    vscode.workspace.onDidChangeTextDocument((e) => {
      if (setting<string>(e.document, "run") === "onType") {
        schedule(e.document, DEBOUNCE_MS);
      }
    }),
    vscode.workspace.onDidCloseTextDocument((d) => diagnostics.delete(d.uri)),
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration("radolint")) {
        resolvedBinary = undefined;
        warnedMissing = false;
        lintAll();
      }
    }),
    vscode.languages.registerCodeActionsProvider(
      { language: "dockerfile" },
      new IgnoreActions(),
      { providedCodeActionKinds: [vscode.CodeActionKind.QuickFix] },
    ),
    vscode.commands.registerCommand("radolint.lintWorkspace", lintAll),
    vscode.commands.registerCommand("radolint.showOutput", () => output.show()),
    vscode.commands.registerCommand("radolint.download", async () => {
      const bin = await download();
      if (bin) {
        resolvedBinary = Promise.resolve(bin);
        lintAll();
      }
    }),
  );
  lintAll();
}

export function deactivate(): void {
  for (const t of timers.values()) clearTimeout(t);
}

function setting<T>(doc: vscode.TextDocument | undefined, key: string): T | undefined {
  return vscode.workspace.getConfiguration("radolint", doc?.uri).get<T>(key);
}

function isDockerfile(doc: vscode.TextDocument): boolean {
  return doc.languageId === "dockerfile" && (doc.uri.scheme === "file" || doc.uri.scheme === "untitled");
}

function lintAll(): void {
  for (const doc of vscode.workspace.textDocuments) schedule(doc, 0);
}

function schedule(doc: vscode.TextDocument, delay: number): void {
  if (!isDockerfile(doc)) return;
  const key = doc.uri.toString();
  clearTimeout(timers.get(key));
  timers.set(key, setTimeout(() => {
    timers.delete(key);
    lint(doc).catch((err) => output.appendLine(`lint failed: ${err}`));
  }, delay));
}

async function lint(doc: vscode.TextDocument): Promise<void> {
  const bin = await binary();
  if (!bin) return;
  const key = doc.uri.toString();
  const version = doc.version;
  running.set(key, version);

  const args = ["--no-color", "--format", "json"];
  const config = setting<string>(doc, "config");
  if (config) args.push("--config", config);
  for (const code of setting<string[]>(doc, "ignore") ?? []) args.push("--ignore", code);
  for (const reg of setting<string[]>(doc, "trustedRegistries") ?? []) args.push("--trusted-registry", reg);
  args.push(...(setting<string[]>(doc, "extraArgs") ?? []), "-");

  // hadolint looks for .hadolint.yaml in the working directory.
  const folder = vscode.workspace.getWorkspaceFolder(doc.uri);
  const cwd = folder?.uri.fsPath ?? (doc.uri.scheme === "file" ? path.dirname(doc.uri.fsPath) : os.homedir());

  const { stdout, stderr, code } = await run(bin, args, cwd, doc.getText());
  // A newer edit started another run; drop this stale result.
  if (running.get(key) !== version || doc.isClosed) return;
  running.delete(key);

  let findings: Finding[];
  try {
    findings = stdout.trim() === "" ? [] : JSON.parse(stdout);
  } catch {
    output.appendLine(`radolint exited with ${code}: ${stderr || stdout}`);
    return;
  }
  if (stderr.trim()) output.appendLine(stderr.trim());
  diagnostics.set(doc.uri, findings.map((f) => toDiagnostic(doc, f)));
}

function toDiagnostic(doc: vscode.TextDocument, f: Finding): vscode.Diagnostic {
  const line = Math.min(Math.max(f.line - 1, 0), Math.max(doc.lineCount - 1, 0));
  const text = doc.lineAt(line);
  const range = f.code === "DL1000"
    ? new vscode.Range(line, Math.max(f.column - 1, 0), line, text.range.end.character)
    : new vscode.Range(line, text.firstNonWhitespaceCharacterIndex, line, text.range.end.character);
  const severity = {
    error: vscode.DiagnosticSeverity.Error,
    warning: vscode.DiagnosticSeverity.Warning,
    info: vscode.DiagnosticSeverity.Information,
    style: vscode.DiagnosticSeverity.Hint,
  }[f.level] ?? vscode.DiagnosticSeverity.Warning;
  const d = new vscode.Diagnostic(range, f.message, severity);
  d.source = SOURCE;
  d.code = { value: f.code, target: vscode.Uri.parse(ruleUrl(f.code)) };
  return d;
}

function ruleUrl(code: string): string {
  return code.startsWith("SC")
    ? `https://www.shellcheck.net/wiki/${code}`
    : `https://github.com/hadolint/hadolint/wiki/${code}`;
}

function run(bin: string, args: string[], cwd: string, input: string):
  Promise<{ stdout: string; stderr: string; code: number | null }> {
  return new Promise((resolve) => {
    const child = spawn(bin, args, { cwd, windowsHide: true });
    let stdout = "";
    let stderr = "";
    child.stdout.setEncoding("utf8").on("data", (c) => (stdout += c));
    child.stderr.setEncoding("utf8").on("data", (c) => (stderr += c));
    child.on("error", (err) => resolve({ stdout: "", stderr: String(err), code: null }));
    child.on("close", (code) => resolve({ stdout, stderr, code }));
    child.stdin.on("error", () => undefined);
    child.stdin.end(input);
  });
}

// ---- finding the binary ---------------------------------------------------

function binary(): Promise<string | undefined> {
  if (!resolvedBinary) resolvedBinary = findBinary();
  return resolvedBinary;
}

async function findBinary(): Promise<string | undefined> {
  const configured = setting<string>(undefined, "path");
  if (configured) {
    if (await works(configured)) return configured;
    vscode.window.showErrorMessage(`radolint: cannot run "${configured}" (radolint.path).`);
    return undefined;
  }
  if (await works("radolint")) return "radolint";
  const downloaded = downloadedPath();
  if (fs.existsSync(downloaded) && await works(downloaded)) return downloaded;
  if (warnedMissing) return undefined;
  warnedMissing = true;
  const choice = await vscode.window.showInformationMessage(
    `radolint was not found on PATH. Download radolint ${RELEASE_VERSION} from GitHub releases?`,
    "Download", "Set Path",
  );
  if (choice === "Download") return download();
  if (choice === "Set Path") {
    vscode.commands.executeCommand("workbench.action.openSettings", "radolint.path");
  }
  return undefined;
}

function works(bin: string): Promise<boolean> {
  return new Promise((resolve) => {
    execFile(bin, ["--version"], { timeout: 10000, windowsHide: true }, (err) => resolve(!err));
  });
}

function target(): { target: string; ext: string } | undefined {
  const key = `${process.platform}/${process.arch}`;
  const targets: Record<string, string> = {
    "linux/x64": "x86_64-unknown-linux-musl",
    "linux/arm64": "aarch64-unknown-linux-musl",
    "darwin/x64": "x86_64-apple-darwin",
    "darwin/arm64": "aarch64-apple-darwin",
    "win32/x64": "x86_64-pc-windows-msvc",
  };
  const t = targets[key];
  return t ? { target: t, ext: process.platform === "win32" ? "zip" : "tar.gz" } : undefined;
}

function downloadedPath(): string {
  const exe = process.platform === "win32" ? "radolint.exe" : "radolint";
  return path.join(storageDir, `v${RELEASE_VERSION}`, exe);
}

async function download(): Promise<string | undefined> {
  const t = target();
  if (!t) {
    vscode.window.showErrorMessage(`radolint: no release binary for ${process.platform}/${process.arch}. Build it with cargo and set radolint.path.`);
    return undefined;
  }
  const asset = `radolint-v${RELEASE_VERSION}-${t.target}.${t.ext}`;
  const base = `https://github.com/${REPO}/releases/download/v${RELEASE_VERSION}`;
  const dir = path.dirname(downloadedPath());
  try {
    return await vscode.window.withProgress(
      { location: vscode.ProgressLocation.Notification, title: `Downloading ${asset}` },
      async () => {
        const [archive, sums] = await Promise.all([fetchBytes(`${base}/${asset}`), fetchBytes(`${base}/SHA256SUMS`)]);
        const expected = sums.toString("utf8").split("\n")
          .map((l) => l.trim().split(/\s+/))
          .find((p) => p[1] === asset)?.[0];
        const actual = createHash("sha256").update(archive).digest("hex");
        if (!expected || expected !== actual) throw new Error(`checksum mismatch for ${asset}`);
        fs.rmSync(dir, { recursive: true, force: true });
        fs.mkdirSync(dir, { recursive: true });
        const file = path.join(dir, asset);
        fs.writeFileSync(file, archive);
        // tar ships with macOS, Linux and Windows 10+, and bsdtar reads zip too.
        await new Promise<void>((resolve, reject) =>
          execFile("tar", ["-xf", file, "-C", dir, "--strip-components=1"], { windowsHide: true },
            (err) => (err ? reject(err) : resolve())));
        fs.rmSync(file, { force: true });
        const bin = downloadedPath();
        if (process.platform !== "win32") fs.chmodSync(bin, 0o755);
        if (!(await works(bin))) throw new Error("downloaded binary does not run");
        output.appendLine(`Downloaded ${asset} to ${bin}`);
        return bin;
      },
    );
  } catch (err) {
    vscode.window.showErrorMessage(`radolint: download failed: ${err}`);
    return undefined;
  }
}

async function fetchBytes(url: string): Promise<Buffer> {
  const res = await fetch(url, { redirect: "follow" });
  if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
  return Buffer.from(await res.arrayBuffer());
}

// ---- quick fixes ----------------------------------------------------------

const PRAGMA = /^(\s*)#\s*hadolint\s+ignore\s*=\s*([A-Z0-9, ]+?)\s*$/;
const GLOBAL_PRAGMA = /^(\s*)#\s*hadolint\s+global\s+ignore\s*=\s*([A-Z0-9, ]+?)\s*$/;

class IgnoreActions implements vscode.CodeActionProvider {
  provideCodeActions(doc: vscode.TextDocument, _range: vscode.Range, ctx: vscode.CodeActionContext): vscode.CodeAction[] {
    const actions: vscode.CodeAction[] = [];
    for (const d of ctx.diagnostics) {
      if (d.source !== SOURCE || typeof d.code !== "object") continue;
      const code = String(d.code.value);
      if (code === "DL1000") continue;
      actions.push(
        this.ignore(doc, d, code, d.range.start.line, PRAGMA, "ignore", `Ignore ${code} for this instruction`),
        this.ignore(doc, d, code, firstInstructionLine(doc), GLOBAL_PRAGMA, "global ignore", `Ignore ${code} in this file`),
      );
    }
    return actions;
  }

  private ignore(doc: vscode.TextDocument, d: vscode.Diagnostic, code: string, line: number,
    pattern: RegExp, kind: string, title: string): vscode.CodeAction {
    const action = new vscode.CodeAction(title, vscode.CodeActionKind.QuickFix);
    action.diagnostics = [d];
    action.edit = new vscode.WorkspaceEdit();
    const above = line > 0 ? doc.lineAt(line - 1) : undefined;
    const m = above && pattern.exec(above.text);
    if (above && m) {
      const codes = m[2].split(/[\s,]+/).filter(Boolean);
      if (!codes.includes(code)) codes.push(code);
      action.edit.replace(doc.uri, above.range, `${m[1]}# hadolint ${kind}=${codes.join(",")}`);
    } else {
      const indent = doc.lineAt(line).text.match(/^\s*/)?.[0] ?? "";
      action.edit.insert(doc.uri, new vscode.Position(line, 0), `${indent}# hadolint ${kind}=${code}${eol(doc)}`);
    }
    return action;
  }
}

// Global pragmas may sit anywhere; put new ones above the first instruction,
// after any leading comments such as `# syntax=` parser directives.
function firstInstructionLine(doc: vscode.TextDocument): number {
  for (let i = 0; i < doc.lineCount; i++) {
    const t = doc.lineAt(i).text.trim();
    if (t !== "" && !t.startsWith("#")) return i;
    if (GLOBAL_PRAGMA.test(doc.lineAt(i).text) && i + 1 < doc.lineCount && !GLOBAL_PRAGMA.test(doc.lineAt(i + 1).text)) {
      return i + 1;
    }
  }
  return 0;
}

function eol(doc: vscode.TextDocument): string {
  return doc.eol === vscode.EndOfLine.CRLF ? "\r\n" : "\n";
}
