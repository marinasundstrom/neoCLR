const vscode = require('vscode');
const fs = require('fs');
const path = require('path');
const assert = require('assert');
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
exports.run = async () => {
  const root = vscode.workspace.workspaceFolders.find(f => f.name === 'Native').uri.fsPath;
  const report = { passed: false, samples: [] };
  let document, original;
  async function hover() {
    const offset = document.getText().indexOf('index.Find(');
    assert(offset >= 0, 'Find invocation exists');
    const started = Date.now();
    const result = await vscode.commands.executeCommand('vscode.executeHoverProvider', document.uri, document.positionAt(offset + 7));
    const text = (result || []).flatMap(h => h.contents.map(c => typeof c === 'string' ? c : c.value)).join('\n');
    report.samples.push({ elapsedMs: Date.now() - started, text });
    return text;
  }
  try {
    await vscode.extensions.getExtension('raven.raven-vscode').activate();
    document = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(root, 'Main.rvn')));
    original = document.getText();
    await vscode.window.showTextDocument(document);
    let ready = false;
    for (let i = 0; i < 180; i++) {
      if ((await hover()).includes('Find')) { ready = true; break; }
      await delay(500);
    }
    assert(ready, 'Find hover never became available');
    for (let i = 0; i < 4; i++) {
      const edit = new vscode.WorkspaceEdit();
      edit.insert(document.uri, document.positionAt(document.getText().length), '\n// Find hover edit\n');
      assert(await vscode.workspace.applyEdit(edit));
      const text = await hover();
      assert(text.includes('Find') && text.includes('Option'), 'Find lost its method contract after an edit');
    }
    report.passed = true;
  } catch (error) { report.error = String(error.stack || error); throw error; }
  finally {
    if (document && original !== undefined) {
      const restore = new vscode.WorkspaceEdit();
      restore.replace(document.uri, new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), original);
      await vscode.workspace.applyEdit(restore);
    }
    fs.writeFileSync(path.join(root, 'vscode-member-hover-acceptance.json'), JSON.stringify(report, null, 2));
  }
};
