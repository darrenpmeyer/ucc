import csv
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / 'extract_printable.py'


class ExtractPrintableTests(unittest.TestCase):
    def test_categories_names_ranges_and_aliases(self):
        xml = '''<ucd xmlns="http://www.unicode.org/ns/2003/ucd/1.0">
        <repertoire>
          <char cp="0000" na="" gc="Cc"/>
          <char cp="0020" na="SPACE" gc="Zs"/>
          <char cp="0041" na="LATIN CAPITAL LETTER A" gc="Lu" na1="OLD A">
            <name-alias alias="OLD A" type="alternate"/>
            <name-alias alias="A, LETTER" type="abbreviation"/>
          </char>
          <char cp="0301" na="COMBINING ACUTE ACCENT" gc="Mn"/>
          <char cp="00A0" na="NO-BREAK SPACE" gc="Zs"/>
          <char cp="200D" na="ZERO WIDTH JOINER" gc="Cf"/>
          <char cp="2028" na="LINE SEPARATOR" gc="Zl"/>
          <char cp="2029" na="PARAGRAPH SEPARATOR" gc="Zp"/>
          <char cp="D800" na="" gc="Cs"/>
          <char cp="E000" na="" gc="Co"/>
          <reserved cp="0378" gc="Cn"/>
          <char first-cp="3400" last-cp="3401" na="CJK UNIFIED IDEOGRAPH-#" gc="Lo"/>
          <char cp="1F600" na="GRINNING FACE" gc="So"/>
          <char cp="10FFFF" na="" gc="Cn"/>
        </repertoire></ucd>'''
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / 'input.xml'
            output = Path(directory) / 'output.csv'
            source.write_text(xml, encoding='utf-8')
            result = subprocess.run(
                [sys.executable, str(SCRIPT), str(source), str(output)],
                capture_output=True, text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            with output.open(newline='', encoding='utf-8') as stream:
                rows = list(csv.DictReader(stream))
        self.assertEqual([r['cp'] for r in rows],
                         ['0020', '0041', '0301', '3400', '3401', '1F600'])
        self.assertEqual(rows[3]['na'], 'CJK UNIFIED IDEOGRAPH-3400')
        self.assertEqual(rows[1]['na1'], 'OLD A')
        self.assertEqual(json.loads(rows[1]['aliases']), ['OLD A', 'A, LETTER'])
        self.assertEqual(json.loads(rows[0]['aliases']), [])

    def test_refuses_to_overwrite_input(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / 'input.xml'
            source.write_text('<ucd/>', encoding='utf-8')
            result = subprocess.run(
                [sys.executable, str(SCRIPT), str(source), str(source)],
                capture_output=True, text=True,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(source.read_text(), '<ucd/>')


if __name__ == '__main__':
    unittest.main()
