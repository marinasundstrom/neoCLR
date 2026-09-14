import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import oniguruma from 'vscode-oniguruma';
import textmate from 'vscode-textmate';

const escape = text => text.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');

export async function createRavenHighlighter() {
  await oniguruma.loadWASM(await readFile(new URL('./node_modules/vscode-oniguruma/release/onig.wasm', import.meta.url)));
  const definition = JSON.parse(await readFile(new URL('./syntaxes/raven.tmLanguage.json', import.meta.url), 'utf8'));
  const registry = new textmate.Registry({
    onigLib: Promise.resolve({
      createOnigScanner: patterns => new oniguruma.OnigScanner(patterns),
      createOnigString: text => new oniguruma.OnigString(text),
    }),
    loadGrammar: async scope => scope === 'source.raven' ? definition : null,
  });
  const grammar = await registry.loadGrammar('source.raven');
  if (!grammar) throw new Error('Raven grammar could not be loaded');
  return source => {
    let stack = textmate.INITIAL;
    return source.split('\n').map(line => {
      const result = grammar.tokenizeLine(line, stack);
      stack = result.ruleStack;
      return result.tokens.map(token => {
        const scopes = token.scopes.join(' ');
        const type = /comment/.test(scopes) ? 'comment'
          : /string/.test(scopes) ? 'string'
          : /constant.numeric/.test(scopes) ? 'number'
          : /keyword|storage\.|constant.language/.test(scopes) ? 'keyword'
          : /entity.name.type|support.type/.test(scopes) ? 'type'
          : /entity.name.function|support.function/.test(scopes) ? 'function'
          : /variable.parameter/.test(scopes) ? 'parameter' : null;
        const text = escape(line.slice(token.startIndex, token.endIndex));
        return type ? `<span class="syntax-${type}">${text}</span>` : text;
      }).join('');
    }).join('\n');
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  let input = '';
  for await (const chunk of process.stdin) input += chunk;
  const highlight = await createRavenHighlighter();
  process.stdout.write(JSON.stringify(JSON.parse(input).map(highlight)));
}
