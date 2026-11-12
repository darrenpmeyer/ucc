use super::*;
use std::io::{Cursor, Read, SeekFrom};

struct Counted {
    input: Cursor<Vec<u8>>,
    rewinds: usize,
    bytes: usize,
}

impl Read for Counted {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let count = self.input.read(buf)?;
        self.bytes += count;
        Ok(count)
    }
}

impl BufRead for Counted {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.input.fill_buf()
    }
    fn consume(&mut self, count: usize) {
        self.bytes += count;
        self.input.consume(count);
    }
}

impl Seek for Counted {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.rewinds += 1;
        self.input.seek(pos)
    }
}

#[test]
fn batched_search_scans_once_for_exact_and_twice_for_fuzzy() {
    let data = b"na,cp,na1,aliases\nFIRST LETTER,0041,,[]\nSECOND LETTER,0042,,[]\nOTHER,0043,FIRST LETTER,[]\n";
    for (queries, passes) in [
        (vec!["FIRST LETTER", "SECOND LETTER"], 1),
        (vec!["FIRST LETTER", "SECOND LET"], 2),
        (vec!["NO MATCH", "ALSO MISSING"], 2),
    ] {
        let mut reader = Counted {
            input: Cursor::new(data.to_vec()),
            rewinds: 0,
            bytes: 0,
        };
        let pending = queries
            .iter()
            .enumerate()
            .map(|(index, query)| (query.to_string(), index))
            .collect();
        let mut matches = vec![BTreeMap::new(); queries.len()];
        search_map(&mut reader, &pending, &mut matches, &BTreeMap::new()).unwrap();
        assert_eq!(reader.rewinds, passes - 1);
        assert_eq!(reader.bytes, data.len() * passes);
        if queries[0] == "FIRST LETTER" {
            // A match late in the file must still introduce ambiguity.
            assert_eq!(matches[0].keys().copied().collect::<Vec<_>>(), ['A', 'C']);
            assert_eq!(matches[1].keys().copied().collect::<Vec<_>>(), ['B']);
        } else {
            assert!(matches.iter().all(BTreeMap::is_empty));
        }
    }
}

#[test]
fn streaming_csv_handles_multiline_fields_bom_and_line_endings() {
    let data = "\u{feff}name,value\r\n\"a,\"\"b\"\"\nnext\",\"x\r\ny\"\r\nlast,z\rfinal,q";
    let mut reader = BufReader::with_capacity(1, Cursor::new(data));
    let mut rows = Vec::new();
    csv(&mut reader, |row| {
        rows.push(row.to_vec());
        Ok(())
    })
    .unwrap();
    assert_eq!(
        rows,
        vec![
            vec!["name", "value"],
            vec!["a,\"b\"\nnext", "x\r\ny"],
            vec!["last", "z"],
            vec!["final", "q"],
        ]
    );
    for invalid in ["\"unterminated\n", "\"closed\"extra\n", "bad\"quote\n"] {
        assert!(csv(&mut invalid.as_bytes(), |_| Ok(())).is_err());
    }
}
