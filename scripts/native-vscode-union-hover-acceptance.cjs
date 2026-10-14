const vscode = require('vscode');
const fs = require('fs');
const path = require('path');
const assert = require('assert');
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));

// Run against a copy of the bundled JSON sample, using an isolated editor profile.
exports.run = async () => {
  const root = vscode.workspace.workspaceFolders[0].uri.fsPath;
  const report = { passed: false, samples: [] };
  let document, original;
  const alternatives = ['StreamError', 'JsonError', 'MappingError'];
  const union = alternatives.join(' | ');
  async function hover(offset) {
    const position = document.positionAt(offset);
    const result = await vscode.commands.executeCommand('vscode.executeHoverProvider', document.uri, position);
    const text = (result || []).flatMap(h => h.contents.map(c => typeof c === 'string' ? c : c.value)).join('\n');
    report.samples.push({ position, text, ranges: (result || []).map(h => h.range) });
    return { result, text, signature: /```[^\n]*\n([\s\S]*?)```/.exec(text)?.[1].trim() };
  }
  try {
    await vscode.extensions.getExtension('raven.raven-vscode').activate();
    document = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(root, 'Main.rvn')));
    original = document.getText();
    await vscode.window.showTextDocument(document);
    const anchor = 'Task<Result<unit, ' + union + '>>';
    const start = original.indexOf(anchor);
    assert(start >= 0, 'Expected the bundled JSON RoundTrip return annotation');
    const unionStart = start + 'Task<Result<unit, '.length;
    let ready = false;
    for (let i = 0; i < 180; i++) {
      if ((await hover(unionStart + 1)).text) { ready = true; break; }
      await delay(500);
    }
    assert(ready, 'Union hover never became available');
    for (let round = 0; round < 2; round++) {
      for (const name of alternatives) {
        const offset = unionStart + union.indexOf(name) + 1;
        const actual = await hover(offset);
        assert(actual.signature?.startsWith('union struct ' + name), actual.text);
        for (const other of alternatives.filter(n => n !== name)) {
          assert(!actual.signature.includes(other), actual.signature);
          const otherPosition = document.positionAt(unionStart + union.indexOf(other) + 1);
          for (const h of actual.result) assert(!h.range?.contains(otherPosition), 'Alternative hover covers its neighbor');
        }
      }
      for (let i = 0; i < union.length; i++) {
        if (union[i] !== '|') continue;
        const actual = await hover(unionStart + i);
        assert.strictEqual(actual.signature, union);
      }
      if (round === 0) {
        const edit = new vscode.WorkspaceEdit();
        edit.insert(document.uri, document.positionAt(document.getText().length), '\n// Union hover edit\n');
        assert(await vscode.workspace.applyEdit(edit));
      }
    }
    report.passed = true;
  } catch (error) {
    report.error = String(error.stack || error);
    throw error;
  } finally {
    if (document && original !== undefined) {
      const restore = new vscode.WorkspaceEdit();
      restore.replace(document.uri, new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), original);
      await vscode.workspace.applyEdit(restore);
    }
    fs.writeFileSync(path.join(root, 'vscode-union-hover-acceptance.json'), JSON.stringify(report, null, 2));
  }
};
