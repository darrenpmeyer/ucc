#!/usr/bin/env python3
"""Stream a flat UCD XML file into a CSV of printable character definitions.

Printable means general categories L, M, N, P, S, plus U+0020 SPACE
(the same category rule as str.isprintable(), using the input UCD version).
Combining marks are included even when they need a base character to display.
The aliases column is a JSON array containing na1 and all name-alias values,
deduplicated in source order. Only the Python standard library is required.
"""

import argparse
import csv
import json
from pathlib import Path
import sys
import xml.etree.ElementTree as ET


def printable_rows(source):
    """Yield CSV rows from a flat UCD XML stream with bounded tree memory."""
    stack = []
    for event, element in ET.iterparse(source, events=('start', 'end')):
        if event == 'start':
            stack.append(element)
            continue
        tag = element.tag.rsplit('}', 1)[-1]
        if tag == 'char':
            attributes = element.attrib
            category = attributes.get('gc', '')
            first = attributes.get('cp', attributes.get('first-cp'))
            if first is None:
                raise ValueError('Character definition has no cp or first-cp')
            start = int(first, 16)
            end = int(attributes.get('last-cp', first), 16)
            if not 0 <= start <= end <= 0x10FFFF:
                raise ValueError(f'Invalid codepoint range: {first}..{end:X}')
            na1 = attributes.get('na1', '')
            aliases = [na1] if na1 else []
            aliases.extend(
                child.attrib['alias'] for child in element
                if child.tag.rsplit('}', 1)[-1] == 'name-alias'
                and child.attrib.get('alias')
            )
            encoded_aliases = json.dumps(list(dict.fromkeys(aliases)), ensure_ascii=False)
            for codepoint in range(start, end + 1):
                if category[:1] in ('L', 'M', 'N', 'P', 'S') or codepoint == 0x20:
                    cp = f'{codepoint:04X}'
                    yield (attributes.get('na', '').replace('#', cp), cp,
                           na1, encoded_aliases)
        # Keep alias children until their enclosing character has been read.
        parent = stack[-2] if len(stack) > 1 else None
        if parent is None or parent.tag.rsplit('}', 1)[-1] != 'char':
            element.clear()
            if parent is not None:
                parent.remove(element)
        stack.pop()


def main():
    """Parse CLI arguments and write the CSV."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', nargs='?', type=Path, default=Path('ucd.all.flat.xml'))
    parser.add_argument('output', nargs='?', type=Path, default=Path('printable.csv'))
    args = parser.parse_args()
    if args.input.resolve() == args.output.resolve() or (
        args.output.exists() and args.input.exists()
        and args.input.samefile(args.output)
    ):
        parser.error('Input and output must be different files')
    try:
        with args.input.open('rb') as source, args.output.open(
            'w', encoding='utf-8', newline=''
        ) as output:
            writer = csv.writer(output)
            writer.writerow(('na', 'cp', 'na1', 'aliases'))
            count = 0
            for row in printable_rows(source):
                writer.writerow(row)
                count += 1
    except (OSError, ET.ParseError, ValueError) as error:
        parser.exit(1, f'{parser.prog}: {error}\n')
    print(f'Wrote {count:,} characters to {args.output}', file=sys.stderr)


if __name__ == '__main__':
    main()
