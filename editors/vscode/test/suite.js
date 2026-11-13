// Opens the fixture Dockerfile, checks the diagnostics and applies a quick fix.
const assert = require("assert");
const vscode = require("vscode");

async function waitFor(fn, what, ms = 20000) {
  const end = Date.now() + ms;
  for (;;) {
    const v = fn();
    if (v) return v;
    if (Date.now() > end) throw new Error(`timed out waiting for ${what}`);
    await new Promise((r) => setTimeout(r, 100));
  }
}

const codes = (uri) => vscode.languages.getDiagnostics(uri).map((d) => d.code.value).sort();

exports.run = async function () {
  const [folder] = vscode.workspace.workspaceFolders;
  const uri = vscode.Uri.joinPath(folder.uri, "Dockerfile");
  const doc = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(doc);
  assert.strictEqual(doc.languageId, "dockerfile");

  await waitFor(() => codes(uri).length === 3, "3 diagnostics");
  assert.deepStrictEqual(codes(uri), ["DL3003", "DL3006", "SC2086"]);
  const sc = vscode.languages.getDiagnostics(uri).find((d) => d.code.value === "SC2086");
  assert.strictEqual(sc.source, "radolint");
  assert.strictEqual(sc.range.start.line, 1);
  assert.strictEqual(String(sc.code.target), "https://www.shellcheck.net/wiki/SC2086");

  // Quick fix: ignore DL3003 for the RUN instruction.
  const actions = await vscode.commands.executeCommand("vscode.executeCodeActionProvider", uri, sc.range);
  const fix = actions.find((a) => a.title === "Ignore DL3003 for this instruction");
  assert.ok(fix, `no quick fix in ${actions.map((a) => a.title)}`);
  await vscode.workspace.applyEdit(fix.edit);
  assert.strictEqual(doc.lineAt(1).text, "# hadolint ignore=DL3003");
  await waitFor(() => codes(uri).join() === "DL3006,SC2086", "DL3003 to go away after the fix");

  // A second fix on the same instruction extends the pragma.
  const sc2 = vscode.languages.getDiagnostics(uri).find((d) => d.code.value === "SC2086");
  const actions2 = await vscode.commands.executeCommand("vscode.executeCodeActionProvider", uri, sc2.range);
  await vscode.workspace.applyEdit(actions2.find((a) => a.title === "Ignore SC2086 for this instruction").edit);
  assert.strictEqual(doc.lineAt(1).text, "# hadolint ignore=DL3003,SC2086");

  // File-wide ignore goes above the first instruction.
  const dl = vscode.languages.getDiagnostics(uri).find((d) => d.code.value === "DL3006")
    ?? (await waitFor(() => vscode.languages.getDiagnostics(uri).find((d) => d.code.value === "DL3006"), "DL3006"));
  const actions3 = await vscode.commands.executeCommand("vscode.executeCodeActionProvider", uri, dl.range);
  await vscode.workspace.applyEdit(actions3.find((a) => a.title === "Ignore DL3006 in this file").edit);
  assert.strictEqual(doc.lineAt(0).text, "# hadolint global ignore=DL3006");
  await waitFor(() => codes(uri).length === 0, "all diagnostics to clear");

  // Parse errors are reported as DL1000 errors.
  const edit = new vscode.WorkspaceEdit();
  edit.insert(uri, new vscode.Position(doc.lineCount, 0), "BOGUS line\n");
  await vscode.workspace.applyEdit(edit);
  const parse = await waitFor(() => vscode.languages.getDiagnostics(uri).find((d) => d.code.value === "DL1000"), "DL1000");
  assert.strictEqual(parse.severity, vscode.DiagnosticSeverity.Error);
  console.log("radolint extension test: all checks passed");
};
