#!/usr/bin/env python3
import re

with open('README.md', 'r') as fh:
    orig = fh.read()

matches = list(re.finditer(r'(?P<open>```\w+\s*\{source=(?P<source>.+?)\}?\n)(.*?)```', orig, re.DOTALL))
matches.reverse() # iterate in reverse order to not mess up indexes while we edit

edited = orig

for match in matches:
    start, end = match.span()
    parts = match.groupdict()

    with open(parts['source'], 'r') as fh:
        file_contents = fh.read()

    print(f"inlined {parts['source']}")

    edited = edited[:start] + parts["open"] + file_contents + "\n```" + edited[end:]

with open('README.md', 'w') as fh:
    fh.write(edited)
