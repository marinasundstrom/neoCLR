import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createRavenHighlighter } from './highlight.mjs';

test('Raven highlighting preserves and escapes code across multiline token states', async () => {
  const highlight = await createRavenHighlighter();
  const source = '/* comment\nstill comment */\nlet value = Result<string, Error>.Ok("<script>&")\nWriteLine(42)';
  const html = highlight(source);
  assert.ok(html.includes('syntax-comment'));
  assert.ok(html.includes('syntax-keyword'));
  assert.ok(html.includes('syntax-string'));
  assert.ok(html.includes('syntax-number'));
  assert.ok(!html.includes('<script>'));
  const decoded = html.replace(/<\/?span[^>]*>/g, '').replaceAll('&quot;', '"').replaceAll('&gt;', '>').replaceAll('&lt;', '<').replaceAll('&amp;', '&');
  assert.equal(decoded, source);
  assert.equal(highlight(''), '');
});
