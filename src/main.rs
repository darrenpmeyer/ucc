use std::{
    collections::{BTreeMap, HashMap},
    env, fs,
    io::{self, BufRead, BufReader, Seek, Write},
    path::PathBuf,
    process,
};

type Result<T> = std::result::Result<T, String>;

// Parse CSV a physical line at a time, retaining at most one logical record.
// Quoted fields may contain commas, escaped quotes, CRLF, and newlines.
fn csv(reader: &mut impl BufRead, mut visit: impl FnMut(&[String]) -> Result<()>) -> Result<()> {
    let mut row = Vec::new();
    let mut field = String::new();
    let mut buffer = Vec::new();
    let mut quoted = false;
    let mut closed = false;
    let mut first = true;
    loop {
        buffer.clear();
        // Recognize CR as well as LF so CR-only CSVs also stay streaming.
        loop {
            let available = reader.fill_buf().map_err(|e| e.to_string())?;
            if available.is_empty() {
                break;
            }
            let end = available.iter().position(|b| matches!(b, b'\r' | b'\n'));
            let count = end.map_or(available.len(), |index| index + 1);
            buffer.extend_from_slice(&available[..count]);
            reader.consume(count);
            if end.is_some() {
                break;
            }
        }
        if buffer.is_empty() {
            break;
        }
        let line = std::str::from_utf8(&buffer).map_err(|e| e.to_string())?;
        let input = if first {
            line.trim_start_matches('\u{feff}')
        } else {
            line
        };
        first = false;
        let mut chars = input.chars().peekable();
        while let Some(c) = chars.next() {
            if quoted {
                if c == '"' {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        field.push('"');
                    } else {
                        quoted = false;
                        closed = true;
                    }
                } else {
                    field.push(c);
                }
            } else {
                match c {
                    '"' if field.is_empty() && !closed => quoted = true,
                    ',' => {
                        row.push(std::mem::take(&mut field));
                        closed = false;
                    }
                    '\n' | '\r' => {
                        if c == '\r' && chars.peek() == Some(&'\n') {
                            chars.next();
                        }
                        row.push(std::mem::take(&mut field));
                        if row.iter().any(|s| !s.is_empty()) {
                            visit(&row)?;
                        }
                        row.clear();
                        closed = false;
                    }
                    _ if closed || c == '"' => return Err("invalid CSV quoting".into()),
                    _ => field.push(c),
                }
            }
        }
    }
    if quoted {
        return Err("unterminated CSV quote".into());
    }
    if !field.is_empty() || !row.is_empty() || closed {
        row.push(field);
        visit(&row)?;
    }
    Ok(())
}

fn codepoint(s: &str) -> Option<char> {
    let s = s.trim();
    let hex = s
        .strip_prefix("U+")
        .or_else(|| s.strip_prefix("u+"))
        .or_else(|| s.strip_prefix("0x"))
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    if hex.is_empty() || hex.len() > 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
}

// Validate every record, even after all requested exact names have been found:
// later records can introduce ambiguous names or invalid map data.
fn scan_map(
    reader: &mut impl BufRead,
    mut visit: impl FnMut(char, &str, &[String]) -> Result<()>,
) -> Result<()> {
    let mut columns: Option<[usize; 4]> = None;
    let mut record = 0;
    csv(reader, |row| {
        record += 1;
        let Some([na, cp, na1, aliases]) = columns else {
            let column = |name: &str| {
                row.iter()
                    .position(|s| s == name)
                    .ok_or_else(|| format!("missing character map column: {name}"))
            };
            columns = Some([
                column("na")?,
                column("cp")?,
                column("na1")?,
                column("aliases")?,
            ]);
            return Ok(());
        };
        let get = |n| {
            row.get(n)
                .ok_or_else(|| format!("character map row {record} is incomplete"))
        };
        let name = get(na)?;
        let ch = codepoint(get(cp)?).ok_or_else(|| format!("invalid codepoint at row {record}"))?;
        let mut names: Vec<String> = serde_json::from_str(get(aliases)?)
            .map_err(|e| format!("invalid aliases at row {record}: {e}"))?;
        names.push(name.clone());
        names.push(get(na1)?.clone());
        names.retain(|s| !s.is_empty());
        for name in &mut names {
            *name = name.to_uppercase();
        }
        visit(ch, name, &names)
    })?;
    if columns.is_none() {
        return Err("empty character map".into());
    }
    Ok(())
}

// Preserve substring lookup and allow ordered word abbreviations, skipping words.
// Callers supply uppercase strings for case-insensitive matching.
fn fuzzy_matches(name: &str, query: &str, words: &[&str]) -> bool {
    if name.contains(query) {
        return true;
    }
    let mut candidates = name.split_whitespace();
    !words.is_empty()
        && words
            .iter()
            .all(|word| candidates.any(|candidate| candidate.starts_with(word)))
}

type Matches = BTreeMap<char, String>;

// One exact pass for every pending query; a second pass only for exact misses.
// This is a tradeoff: makes exact matches faster and lower-RAM usage, in
// exchange for slower fuzzy match searches
fn search_map(
    reader: &mut (impl BufRead + Seek),
    pending: &HashMap<String, usize>,
    matches: &mut [Matches],
    aliases: &BTreeMap<String, char>,
) -> Result<()> {
    scan_map(reader, |ch, display, names| {
        for name in names {
            if let Some(&index) = pending.get(name) {
                matches[index].insert(ch, display.to_owned());
            }
        }
        Ok(())
    })?;
    let fuzzy: Vec<_> = pending
        .iter()
        .filter(|(_, index)| matches[**index].is_empty())
        .map(|(key, &index)| {
            (
                key.as_str(),
                index,
                key.split_whitespace().collect::<Vec<_>>(),
            )
        })
        .collect();
    if fuzzy.is_empty() {
        return Ok(());
    }
    reader.rewind().map_err(|e| e.to_string())?;
    scan_map(reader, |ch, display, names| {
        for (key, index, words) in &fuzzy {
            if names.iter().any(|name| fuzzy_matches(name, key, words)) {
                matches[*index].insert(ch, display.to_owned());
            }
        }
        Ok(())
    })?;
    for (key, index, words) in fuzzy {
        for (name, ch) in aliases {
            if fuzzy_matches(name, key, &words) {
                matches[index].insert(*ch, name.clone());
            }
        }
    }
    Ok(())
}

fn lookup_batch(
    queries: &[String],
    map_path: &std::path::Path,
    aliases: &BTreeMap<String, char>,
) -> Result<(Vec<Matches>, Vec<usize>)> {
    let mut indices = HashMap::new();
    let mut pending = HashMap::new();
    let mut matches = Vec::new();
    let mut order = Vec::new();
    for query in queries {
        let key = query.to_uppercase();
        if let Some(&index) = indices.get(&key) {
            order.push(index);
            continue;
        }
        let index = matches.len();
        let result = if let Some(ch) = aliases.get(&key) {
            BTreeMap::from([(*ch, key.clone())])
        } else if let Some(ch) = codepoint(query) {
            BTreeMap::from([(ch, format!("U+{:04X}", ch as u32))])
        } else {
            pending.insert(key.clone(), index);
            BTreeMap::new()
        };
        indices.insert(key, index);
        matches.push(result);
        order.push(index);
    }
    if !pending.is_empty() {
        let file = fs::File::open(map_path).map_err(|e| {
            format!(
                "{}: {e}; install the character map with make install or specify --charmap FILE",
                map_path.display()
            )
        })?;
        search_map(&mut BufReader::new(file), &pending, &mut matches, aliases)?;
    }
    Ok((matches, order))
}

fn format_matches(matches: &BTreeMap<char, String>, limit: usize) -> String {
    matches
        .iter()
        .take(limit)
        .map(|(ch, name)| format!("{ch} ({name})\n"))
        .collect()
}

fn resolve(query: &str, matches: &Matches) -> Result<char> {
    if matches.is_empty() {
        return Err(format!("no character matches {query:?}"));
    }
    if matches.len() == 1 {
        return Ok(*matches.keys().next().unwrap());
    }
    let mut options = format_matches(matches, 10);
    if matches.len() > 10 {
        options.push_str(&format!(
            ".. and {} more matches (use --pager) for more\n",
            matches.len() - 10
        ));
    }
    Err(format!(
        "multiple characters match {query:?}:\n{}",
        options.trim_end()
    ))
}

const HELP: &str = "Usage: ucc [OPTIONS] NAME

Emit a UTF-8 character without a trailing newline.

  --charmap FILE   Alternative character map CSV
  --aliases FILE   Alternative name,value alias CSV
  --combine NAME   Append a character (repeatable)
  --pager         List all matches to stdout for piping to a pager
  -h, --help       Show help
  -V, --version    Show version

Codepoints accept hexadecimal, U+XXXX, or 0xXXXX.
Fuzzy matching uses case-insensitive substrings or ordered word prefixes.";

struct Options {
    map_path: PathBuf,
    alias_path: Option<PathBuf>,
    query: String,
    combines: Vec<String>,
    pager: bool,
}

// Help and version requests return without running a lookup.
fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Option<Options>> {
    let mut map_path = None;
    let mut alias_path = None;
    let mut combines = Vec::new();
    let mut query = None;
    let mut positional = false;
    let mut pager = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" if !positional => {
                println!("{HELP}");
                return Ok(None);
            }
            "--version" | "-V" if !positional => {
                println!("ucc {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            "--pager" if !positional => pager = true,
            "--" if !positional => positional = true,
            "--charmap" | "--aliases" | "--combine" if !positional => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("{arg} requires a value"))?;
                match arg.as_str() {
                    "--charmap" => map_path = Some(value),
                    "--aliases" => alias_path = Some(value),
                    _ => combines.push(value),
                }
            }
            _ if !positional && arg.starts_with('-') => {
                return Err(format!("unknown option: {arg}"));
            }
            _ => {
                if query.replace(arg).is_some() {
                    return Err("expected one NAME argument; quote names containing spaces".into());
                }
            }
        }
    }
    let query = query
        .filter(|s| !s.trim().is_empty())
        .ok_or("a nonempty NAME is required (see --help)")?;
    if pager && !combines.is_empty() {
        return Err("--pager cannot be used with --combine; list each character separately".into());
    }
    let map_path = map_path.map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(option_env!("UCC_DATADIR").unwrap_or("/usr/local/share/ucc"))
            .join("printable.csv")
    });
    Ok(Some(Options {
        map_path,
        alias_path: alias_path.map(PathBuf::from),
        query,
        combines,
        pager,
    }))
}

fn load_aliases(alias_path: Option<PathBuf>) -> Result<BTreeMap<String, char>> {
    let explicit_alias_path = alias_path.is_some();
    let alias_path = alias_path.or_else(|| {
        env::var_os("XDG_CONFIG_HOME")
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|s| PathBuf::from(s).join(".config")))
            .map(|p| p.join("ucc/aliases.csv"))
    });
    let mut aliases = BTreeMap::new();
    if let Some(alias_path) = alias_path {
        match fs::read_to_string(&alias_path) {
            Ok(data) => {
                let mut record = 0;
                csv(&mut data.as_bytes(), |row| {
                    record += 1;
                    if row.len() != 2 || row[0].trim().is_empty() {
                        return Err(format!(
                            "invalid alias row {record} in {}",
                            alias_path.display()
                        ));
                    }
                    let value = &row[1];
                    let ch = (|| {
                        let mut chars = value.chars();
                        let ch = chars.next()?;
                        if chars.next().is_none() {
                            Some(ch)
                        } else {
                            None
                        }
                    })()
                    .or_else(|| codepoint(value))
                    .ok_or_else(|| format!("invalid alias value at row {record}"))?;
                    aliases.insert(row[0].to_uppercase(), ch);
                    Ok(())
                })?;
            }
            Err(e) if !explicit_alias_path && e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(format!("{}: {e}", alias_path.display())),
        }
    }
    Ok(aliases)
}

fn run() -> Result<()> {
    let Some(Options {
        map_path,
        alias_path,
        query,
        combines,
        pager,
    }) = parse_args(env::args().skip(1))?
    else {
        return Ok(());
    };
    let aliases = load_aliases(alias_path)?;
    if combines.iter().any(|name| name.trim().is_empty()) {
        return Err("combining character name must not be empty".into());
    }
    let queries: Vec<_> = std::iter::once(query).chain(combines).collect();
    let (matches, order) = lookup_batch(&queries, &map_path, &aliases)?;
    if pager {
        let result = &matches[order[0]];
        if result.is_empty() {
            return Err(format!("no character matches {:?}", queries[0]));
        }
        return io::stdout()
            .write_all(format_matches(result, usize::MAX).as_bytes())
            .map_err(|e| e.to_string());
    }
    let mut output = String::new();
    for (query, index) in queries.iter().zip(order) {
        output.push(resolve(query, &matches[index])?);
    }
    io::stdout()
        .write_all(output.as_bytes())
        .map_err(|e| e.to_string())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ucc: {error}");
        process::exit(1);
    }
}

#[cfg(test)]
#[path = "../tests/unit/mod.rs"]
mod tests;
