// Run inside VS Code with --extensionTestsPath; uses the real Raven extension and tasks.
const vscode = require('vscode');
const fs = require('fs');
const path = require('path');
const assert = require('assert');
const crypto = require('crypto');
const hoverText = result => (result || []).flatMap(h => h.contents.map(c => typeof c === 'string' ? c : c.value)).join('\n');
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
async function until(action, label, timeout = 90000) {
  const deadline = Date.now() + timeout;
  let last;
  while (Date.now() < deadline) {
    try { const result = await action(); if (result) return result; } catch (error) { last = error; }
    await delay(500);
  }
  throw new Error(`Timeout: ${label}; ${last || ''}`);
}
function taskExit(task) {
  return new Promise(async (resolve, reject) => {
    let execution;
    const timer = setTimeout(() => { listener.dispose(); execution?.terminate(); reject(new Error('Task timeout')); }, 120000);
    const listener = vscode.tasks.onDidEndTaskProcess(e => {
      if (e.execution !== execution) return;
      clearTimeout(timer); listener.dispose(); resolve(e.exitCode);
    });
    try { execution = await vscode.tasks.executeTask(task); }
    catch (error) { clearTimeout(timer); listener.dispose(); reject(error); }
  });
}
exports.run = async function () {
  const folder = vscode.workspace.workspaceFolders.find(f => f.name === 'Native') || vscode.workspace.workspaceFolders[0];
  const root = folder.uri.fsPath;
  const config = JSON.parse(fs.readFileSync(path.join(root, 'acceptance.json')));
  const report = { vscode: vscode.version, checks: [], passed: false };
  const record = label => { report.checks.push(label); console.log('PASS ' + label); };
  let probe;
  try {
    await vscode.extensions.getExtension('raven.raven-vscode').activate();
    const uri = vscode.Uri.file(path.join(root, 'Probe.rvn'));
    probe = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(probe);
    const position = probe.positionAt(probe.getText().indexOf('VersionOne') + 2);
    const hover = await until(async () => {
      const result = await vscode.commands.executeCommand('vscode.executeHoverProvider', uri, position);
      report.lastHover = hoverText(result);
      fs.writeFileSync(path.join(root, 'vscode-progress.json'), JSON.stringify(report));
      report.diagnostics = vscode.languages.getDiagnostics(uri).map(d => ({message:d.message, severity:d.severity}));
      return hoverText(result).includes('VersionOne') && result;
    }, 'native hover');
    report.hover = hoverText(hover); record('native imported hover');
    assert(report.hover.includes('documented answer'), 'imported API documentation is visible');
    record('native imported Markdown documentation hover');
    const docsRoot = path.join(root, 'references', 'EditorLibrary.docs');
    const walk = dir => fs.readdirSync(dir, { withFileTypes: true }).flatMap(e =>
      e.isDirectory() ? walk(path.join(dir, e.name)) : [path.join(dir, e.name)]);
    const memberDoc = walk(docsRoot).find(f => f.endsWith('.md') && fs.readFileSync(f, 'utf8').includes('documented answer'));
    assert(memberDoc);
    fs.writeFileSync(memberDoc, fs.readFileSync(memberDoc, 'utf8').replace('documented answer', 'refreshed answer'));
    await until(async () => hoverText(await vscode.commands.executeCommand('vscode.executeHoverProvider', uri, position)).includes('refreshed answer'), 'documentation-only refresh');
    record('native Markdown sidecar edit refreshes hover');
    fs.unlinkSync(memberDoc);
    await until(async () => hoverText(await vscode.commands.executeCommand('vscode.executeHoverProvider', uri, position)).includes('documented answer'), 'XML fallback after Markdown deletion');
    record('native XML fallback after Markdown member deletion');
    const definitions = await until(async () => {
      const result = await vscode.commands.executeCommand('vscode.executeDefinitionProvider', uri, position);
      return result?.length && result;
    }, 'metadata navigation');
    const target = definitions[0].targetUri || definitions[0].uri;
    assert.strictEqual(target.scheme, 'raven-metadata');
    const metadata = await vscode.workspace.openTextDocument(target);
    assert(metadata.getText().includes('VersionOne'));
    assert(metadata.getText().includes('Metadata declarations'));
    record('read-only metadata declaration navigation');
    async function edit(text) {
      const change = new vscode.WorkspaceEdit();
      change.replace(uri, new vscode.Range(probe.positionAt(0), probe.positionAt(probe.getText().length)), text);
      assert(await vscode.workspace.applyEdit(change));
    }
    await edit('func EditorProbe() -> int => EditorApi.');
    const completion = await until(async () => {
      const result = await vscode.commands.executeCommand('vscode.executeCompletionItemProvider', uri, probe.positionAt(probe.getText().length));
      return result?.items?.some(i => JSON.stringify(i.label).includes('VersionOne')) && result;
    }, 'native completion');
    report.completion = completion.items.map(i => i.label); record('native imported completion');
    const documentedCompletion = completion.items.find(i => JSON.stringify(i.label).includes('VersionOne'));
    assert((documentedCompletion.documentation?.value || documentedCompletion.documentation || '').includes('documented answer'));
    record('native imported completion documentation');
    await edit('func EditorProbe() -> int => EditorApi.VersionOne()\n');
    fs.copyFileSync(config.replacementLibrary, path.join(root, 'references/EditorLibrary.dll'));
    await until(() => vscode.languages.getDiagnostics(uri).some(d => d.severity === vscode.DiagnosticSeverity.Error && d.message.includes('VersionOne')), 'library replacement diagnostic');
    record('actual file watcher invalidates replaced native library');
    await edit('func EditorProbe() -> int => EditorApi.VersionTwo()\n');
    await until(async () => {
      const result = await vscode.commands.executeCommand('vscode.executeHoverProvider', uri, probe.positionAt(probe.getText().indexOf('VersionTwo') + 2));
      return hoverText(result).includes('VersionTwo') && !vscode.languages.getDiagnostics(uri).some(d => d.severity === 0);
    }, 'replacement symbol with unsaved buffer');
    record('unsaved buffer survives native workspace reload');
    await probe.save();
    const dependency = path.join(root, 'references/EditorLibrary.dll');
    const dependencyImage = fs.readFileSync(dependency);
    fs.unlinkSync(dependency);
    const projectUri = vscode.Uri.file(path.join(root, 'App.rvnproj'));
    try {
      await until(() => vscode.languages.getDiagnostics(projectUri).some(d => d.source === 'raven-project' && d.message.includes('EditorLibrary')), 'missing reference project diagnostic');
      record('missing native reference produces a project diagnostic');
    } finally { fs.writeFileSync(dependency, dependencyImage); }
    await until(() => !vscode.languages.getDiagnostics(projectUri).some(d => d.source === 'raven-project'), 'dependency recovery');
    record('native dependency recovery clears the project diagnostic');
    const main = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(root, 'Main.rvn')));
    await vscode.window.showTextDocument(main);
    await until(async () => {
      const result = await vscode.commands.executeCommand('vscode.executeHoverProvider', main.uri, main.positionAt(main.getText().indexOf('ArrayList') + 2));
      return hoverText(result).includes('ArrayList');
    }, 'source-built class-library symbols');
    assert(!vscode.languages.getDiagnostics(main.uri).some(d => d.severity === 0));
    record('unchanged broad sample imports source-built class library');
    report.unionHovers = {};
    for (const term of ['Option<Order>', 'Result<Order, SingleError>']) {
      const result = await vscode.commands.executeCommand('vscode.executeHoverProvider', main.uri, main.positionAt(main.getText().indexOf(term) + 2));
      const signature = hoverText(result);
      assert(signature.includes('union struct ' + term), signature);
      report.unionHovers[term] = signature;
    }
    record('native Option and Result type hovers retain union kind and constructed arguments');
    if (config.systemDocumentation) {
      assert(report.unionHovers['Option<Order>'].includes('A nominal union containing Some'));
      assert(report.unionHovers['Result<Order, SingleError>'].includes('A nominal union containing a successful value'));
      assert(!Object.values(report.unionHovers).some(text => text.includes('generated IUnion contract')));
      record('source-built Option and Result display shared authored API documentation');
    }
    const tasks = await vscode.tasks.fetchTasks();
    const build = tasks.find(t => t.name === 'neoCLR: Build');
    assert(build, 'configured native build task');
    assert.strictEqual(await taskExit(build), 0); record('configured VS Code native build task');
    const quote = value => "'" + value.replace(/'/g, "'\\''") + "'";
    const command = ['dotnet', config.compiler, 'neoclr', '--project', path.join(root, 'App.rvnproj'), '--run', config.runtime].map(quote).join(' ')
      + ' > ' + quote(path.join(root, 'run.stdout')) + ' 2> ' + quote(path.join(root, 'run.stderr'));
    const run = new vscode.Task({ type: 'shell' }, folder, 'neoCLR: Captured acceptance run', 'acceptance', new vscode.ShellExecution(command));
    assert.strictEqual(await taskExit(run), 0);
    const stdout = fs.readFileSync(path.join(root, 'run.stdout'), 'utf8');
    assert.strictEqual(stdout.split('\n').slice(1).join('\n'), fs.readFileSync(path.join(root, 'expected.txt'), 'utf8'));
    assert.strictEqual(fs.readFileSync(path.join(root, 'run.stderr'), 'utf8'), '');
    report.stdout = stdout; record('VS Code task executes exact broad-sample output on native runtime');
    const artifact = path.join(root, 'bin/neoclr/App.dll');
    const sha = () => crypto.createHash('sha256').update(fs.readFileSync(artifact)).digest('hex');
    const before = sha();
    await edit('func EditorProbe() -> int => EditorApi.Missing()\n'); await probe.save();
    assert.strictEqual(await taskExit(build), 1);
    assert.strictEqual(sha(), before); record('failed build preserves last successful artifact');
    await edit('func EditorProbe() -> int => EditorApi.VersionTwo()\n'); await probe.save();
    const dotnetFolder = vscode.workspace.workspaceFolders.find(f => f.name === 'DotNet');
    if (dotnetFolder) {
      const doc = await vscode.workspace.openTextDocument(vscode.Uri.joinPath(dotnetFolder.uri, 'Main.rvn'));
      await vscode.window.showTextDocument(doc);
      await until(async () => hoverText(await vscode.commands.executeCommand('vscode.executeHoverProvider', doc.uri,
        doc.positionAt(doc.getText().indexOf('Abs') + 1))).includes('Abs'), '.NET hover');
      assert(!vscode.languages.getDiagnostics(doc.uri).some(d => d.severity === 0));
      record('ordinary .NET editor hover and diagnostics');
    }
    const asyncFolder = vscode.workspace.workspaceFolders.find(f => f.name === 'Async');
    if (asyncFolder) {
      const asyncRoot = asyncFolder.uri.fsPath;
      const doc = await vscode.workspace.openTextDocument(vscode.Uri.joinPath(asyncFolder.uri, 'Main.rvn'));
      await vscode.window.showTextDocument(doc);
      const asyncCommand = ['dotnet', config.compiler, 'neoclr', '--project', path.join(asyncRoot, 'Async.rvnproj'), '--run', config.runtime].map(quote).join(' ')
        + ' > ' + quote(path.join(asyncRoot, 'run.stdout')) + ' 2> ' + quote(path.join(asyncRoot, 'run.stderr'));
      const asyncTask = new vscode.Task({type:'shell'}, asyncFolder, 'neoCLR: Async acceptance run', 'acceptance', new vscode.ShellExecution(asyncCommand));
      assert.strictEqual(await taskExit(asyncTask), 0);
      assert.strictEqual(fs.readFileSync(path.join(asyncRoot, 'run.stdout'), 'utf8').split('\n').slice(1).join('\n'), fs.readFileSync(path.join(asyncRoot, 'expected.txt'), 'utf8'));
      assert.strictEqual(fs.readFileSync(path.join(asyncRoot, 'run.stderr'), 'utf8'), '');
      assert(!vscode.languages.getDiagnostics(doc.uri).some(d => d.severity === 0));
      record('unchanged Tasks/await sample builds and executes from VS Code');
    }
    report.passed = true;
  } catch (error) {
    report.error = error.stack;
    throw error;
  } finally {
    fs.writeFileSync(path.join(root, 'vscode-acceptance.json'), JSON.stringify(report, null, 2));
  }
};
