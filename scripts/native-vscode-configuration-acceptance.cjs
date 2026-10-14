// Exercise only imported configuration invalidation/recovery in the real editor.
const vscode = require('vscode');
const fs = require('fs');
const path = require('path');
const assert = require('assert');
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
exports.run = async () => {
  const root = vscode.workspace.workspaceFolders.find(f => f.name === 'Native').uri.fsPath;
  const config = path.join(root, 'references/NeoCLR.ClassLibrary.props');
  const original = fs.readFileSync(config, 'utf8');
  const project = vscode.Uri.file(path.join(root, 'App.rvnproj'));
  const report = { passed: false, rounds: [] };
  const hasError = () => vscode.languages.getDiagnostics(project).some(d => d.source === 'raven-project');
  async function until(check, label) {
    for (let i = 0; i < 180; i++) { if (await check()) return; await delay(500); }
    throw Error('Timeout: ' + label);
  }
  try {
    await vscode.extensions.getExtension('raven.raven-vscode').activate();
    const doc = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(root, 'Main.rvn')));
    await vscode.window.showTextDocument(doc);
    await until(async () => (await vscode.commands.executeCommand('vscode.executeHoverProvider', doc.uri,
      doc.positionAt(doc.getText().indexOf('ArrayList') + 2)))?.length, 'initial workspace');
    const invalid = original.replace('<RavenNeoClrObjectLibrary>System.Runtime</RavenNeoClrObjectLibrary>',
      '<RavenNeoClrObjectLibrary>Missing.Owner</RavenNeoClrObjectLibrary>');
    assert.notStrictEqual(original, invalid);
    for (let round = 0; round < 3; round++) {
      fs.writeFileSync(config, invalid);
      await until(hasError, 'configuration diagnostic');
      fs.writeFileSync(config, original);
      await until(() => !hasError(), 'configuration recovery');
      report.rounds.push({ round, recovered: true });
    }
    report.passed = true;
  } catch (error) { report.error = String(error.stack || error); throw error; }
  finally {
    fs.writeFileSync(config, original);
    fs.writeFileSync(path.join(root, 'vscode-configuration-acceptance.json'), JSON.stringify(report, null, 2));
  }
};
