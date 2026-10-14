const vscode = require('vscode');
const fs = require('fs');
const path = require('path');
const assert = require('assert');
exports.run = async () => {
  const report = { passed: false, samples: [] };
  const kit = path.dirname(vscode.workspace.workspaceFile.fsPath);
  try {
    await vscode.extensions.getExtension('raven.raven-vscode').activate();
    for (const name of ['Unions', 'Intersections', 'ExternalBounds']) {
      const doc = await vscode.workspace.openTextDocument(path.join(kit, name, 'Main.rvn'));
      await vscode.window.showTextDocument(doc);
      const offset = doc.getText().indexOf('Main()');
      assert(offset >= 0);
      let hover;
      for (let n = 0; n < 180; n++) {
        hover = await vscode.commands.executeCommand('vscode.executeHoverProvider', doc.uri, doc.positionAt(offset + 1));
        if (hover?.length) break;
        await new Promise(resolve => setTimeout(resolve, 500));
      }
      assert(hover?.length, `${name}: no Main hover`);
      await new Promise(resolve => setTimeout(resolve, 1000));
      const errors = vscode.languages.getDiagnostics(doc.uri).filter(d => d.severity === vscode.DiagnosticSeverity.Error).map(d => d.message);
      report.samples.push({name, hover: hover.flatMap(h => h.contents.map(c => c.value || c)), errors});
      assert.equal(errors.length, 0, `${name}: ${errors.join('\n')}`);
    }
    report.passed = true;
  } finally {
    fs.writeFileSync(path.join(kit, 'editor-check.json'), JSON.stringify(report, null, 2));
  }
};
