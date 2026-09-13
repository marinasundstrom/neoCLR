#!/usr/bin/env python3
"""Build the dependency-free project site using excerpts from executable samples."""
from html import escape
from html.parser import HTMLParser
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / 'website'
OUTPUT = ROOT / 'target/website'


def excerpt(path, start, end):
    text = (ROOT / path).read_text(encoding='utf-8')
    first = text.index(start)
    last = text.index(end, first) + len(end)
    return escape(text[first:last])


class PageCheck(HTMLParser):
    def __init__(self):
        super().__init__()
        self.ids = set()
        self.links = []

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if 'id' in attrs:
            if attrs['id'] in self.ids:
                raise ValueError('Duplicate anchor: ' + attrs['id'])
            self.ids.add(attrs['id'])
        for attr in ('href', 'src'):
            if attr in attrs:
                self.links.append(attrs[attr])

    def check(self):
        for link in self.links:
            if link.startswith('#'):
                if link[1:] not in self.ids:
                    raise ValueError('Missing anchor: ' + link)
            elif '://' not in link and not (OUTPUT / link).exists():
                raise ValueError('Missing local asset: ' + link)


def main():
    if OUTPUT.exists():
        shutil.rmtree(OUTPUT)
    OUTPUT.mkdir(parents=True)
    for name in ('style.css', 'mark.svg'):
        shutil.copyfile(SOURCE / name, OUTPUT / name)
    page = (SOURCE / 'index.html').read_text(encoding='utf-8')
    page = page.replace('{{RAVEN_SAMPLE}}', excerpt(
        'docs/experiments/raven-target/samples/library-propagation.rvn',
        'func Normalize', '\n}'))
    page = page.replace('{{IL_SAMPLE}}', excerpt(
        'examples/preview/result-void.neoil', '.function Complete', '.end'))
    if '{{' in page:
        raise ValueError('Unexpanded website placeholder')
    (OUTPUT / 'index.html').write_text(page, encoding='utf-8')
    (OUTPUT / '.nojekyll').touch()
    check = PageCheck()
    check.feed(page)
    check.check()
    print('Built and checked:', OUTPUT)


if __name__ == '__main__':
    main()
