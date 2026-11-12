use std::{
    fs,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ucc"))
        .args([
            "--charmap",
            concat!(env!("CARGO_MANIFEST_DIR"), "/printable.csv"),
        ])
        .args(args)
        .env("XDG_CONFIG_HOME", "/nonexistent-ucc-test-config")
        .output()
        .unwrap()
}
fn fixture(data: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ucc-test-{}-{}.csv",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, data).unwrap();
    path
}
#[test]
fn default_map_names_aliases_and_codepoints() {
    for (query, expected) in [
        ("em dash", "—"),
        ("period", "."),
        ("U+1F600", "😀"),
        ("0x2014", "—"),
        ("2014", "—"),
        ("LATIN CAPITAL LETTER A", "A"),
    ] {
        let result = run(&[query]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout, expected.as_bytes());
    }
}
#[test]
fn fuzzy_and_ambiguous_results() {
    let path = fixture("na,cp,na1,aliases\r\nEM DASH,2014,,[]\r\nEN DASH,2013,,[]\r\n");
    let p = path.to_str().unwrap();
    assert_eq!(run(&["--charmap", p, "em da"]).stdout, "—".as_bytes());
    let result = run(&["--charmap", p, "dash"]);
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    let err = String::from_utf8(result.stderr).unwrap();
    assert!(err.contains("— (EM DASH)"));
    assert!(err.contains("– (EN DASH)"));
    fs::remove_file(path).unwrap();
}
#[test]
fn user_aliases_and_combining_output_are_atomic() {
    let path = fixture("dash,2014\nletter,A\n\"comma,name\",\",\"\naccent,U+0301\n");
    let p = path.to_str().unwrap();
    assert_eq!(run(&["--aliases", p, "DASH"]).stdout, "—".as_bytes());
    assert_eq!(run(&["--aliases", p, "comma,name"]).stdout, b",");
    assert_eq!(
        run(&[
            "--aliases",
            p,
            "letter",
            "--combine",
            "accent",
            "--combine",
            "U+0308"
        ])
        .stdout,
        "A\u{301}\u{308}".as_bytes()
    );
    let bad = run(&["--aliases", p, "letter", "--combine", "no such character"]);
    assert!(!bad.status.success());
    assert!(bad.stdout.is_empty());
    fs::remove_file(path).unwrap();
}
#[test]
fn invalid_inputs_fail_without_output() {
    for args in [
        vec![],
        vec![""],
        vec!["--unknown"],
        vec!["--combine"],
        vec!["U+D800"],
        vec!["U+110000"],
        vec!["--aliases", "/nonexistent-ucc-file", "SPACE"],
    ] {
        let output = run(&args);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
    for data in [
        "na,cp,na1,aliases\nBAD,D800,,[]\n",
        "na,cp,na1,aliases\nBAD,0041,,invalid\n",
        "\"unclosed",
    ] {
        let path = fixture(data);
        assert!(
            !run(&["--charmap", path.to_str().unwrap(), "BROKEN NAME"])
                .status
                .success()
        );
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn match_limit_and_pager() {
    for count in [10, 11, 15] {
        let mut data = "na,cp,na1,aliases\n".to_string();
        for i in 0..count {
            data.push_str(&format!("TEST LETTER {i},00{:02X},,[]\n", 65 + i));
        }
        let path = fixture(&data);
        let p = path.to_str().unwrap();
        let result = run(&["--charmap", p, "TEST"]);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        let err = String::from_utf8(result.stderr).unwrap();
        assert_eq!(
            err.lines().filter(|l| l.contains("(TEST LETTER")).count(),
            10
        );
        if count > 10 {
            assert!(err.contains(&format!(
                ".. and {} more matches (use --pager) for more",
                count - 10
            )));
            assert!(!err.contains("(TEST LETTER 10)"));
        } else {
            assert!(!err.contains("more matches"));
        }
        let result = run(&["--charmap", p, "--pager", "TEST"]);
        assert!(result.status.success());
        assert!(result.stderr.is_empty());
        let expected: String = (0..count)
            .map(|i| format!("{} (TEST LETTER {i})\n", char::from_u32(65 + i).unwrap()))
            .collect();
        assert_eq!(result.stdout, expected.as_bytes());
        let single = run(&["--charmap", p, "--pager", "TEST LETTER 0"]);
        assert!(single.status.success());
        assert_eq!(single.stdout, b"A (TEST LETTER 0)\n");
        let missing = run(&["--charmap", p, "--pager", "NOT FOUND"]);
        assert!(!missing.status.success());
        assert!(missing.stdout.is_empty());
        let combined = run(&["--charmap", p, "--pager", "TEST", "--combine", "U+0301"]);
        assert!(!combined.status.success());
        assert!(combined.stdout.is_empty());
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn abbreviated_words_match_in_order() {
    let result = run(&["subscr one"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, "₁".as_bytes());
    assert_eq!(
        run(&["--pager", "SUBSCR ONE"]).stdout,
        "₁ (SUBSCRIPT ONE)\n".as_bytes()
    );

    let path = fixture(
        "na,cp,na1,aliases\nLATIN CAPITAL LETTER A,0041,,[]\nLATIN CAPITAL LETTER B,0042,,[]\nLAT CAP,0043,,[]\n",
    );
    let p = path.to_str().unwrap();
    assert_eq!(run(&["--charmap", p, "lat cap a"]).stdout, b"A");
    // Exact names still take priority over the additional prefix matches.
    assert_eq!(run(&["--charmap", p, "LAT CAP"]).stdout, b"C");
    let ambiguous = run(&["--charmap", p, "lat capit"]);
    assert!(!ambiguous.status.success());
    assert!(ambiguous.stdout.is_empty());
    let err = String::from_utf8(ambiguous.stderr).unwrap();
    assert!(err.contains("A (LATIN CAPITAL LETTER A)"));
    assert!(err.contains("B (LATIN CAPITAL LETTER B)"));
    assert_eq!(
        run(&["--charmap", p, "--pager", "lat capit"]).stdout,
        b"A (LATIN CAPITAL LETTER A)\nB (LATIN CAPITAL LETTER B)\n"
    );
    assert!(!run(&["--charmap", p, "cap lat"]).status.success());
    // A query word must use a separate name word, and must match its prefix.
    assert!(!run(&["--charmap", p, "lat lat"]).status.success());
    assert!(!run(&["--charmap", p, "atin cap"]).status.success());
    fs::remove_file(path).unwrap();

    let aliases = fixture("my special accent,U+0301\n");
    assert_eq!(
        run(&[
            "--aliases",
            aliases.to_str().unwrap(),
            "U+0041",
            "--combine",
            "my acc"
        ])
        .stdout,
        "A\u{301}".as_bytes()
    );
    fs::remove_file(aliases).unwrap();
}

#[test]
fn direct_queries_skip_map_and_preserve_alias_precedence() {
    let aliases = fixture("U+0041,Z\nletter,B\n");
    let p = aliases.to_str().unwrap();
    let result = run(&[
        "--charmap",
        "/nonexistent-ucc-map",
        "--aliases",
        p,
        "U+0041",
        "--combine",
        "letter",
        "--combine",
        "0x0043",
        "--combine",
        "LETTER",
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, b"ZBCB");
    let pager = run(&["--charmap", "/nonexistent-ucc-map", "--pager", "U+0041"]);
    assert!(pager.status.success());
    assert_eq!(pager.stdout, b"A (U+0041)\n");
    fs::remove_file(aliases).unwrap();
}

#[test]
fn batch_preserves_exact_precedence_order_and_deduplication() {
    let map = fixture(concat!(
        "na,cp,na1,aliases\n",
        "ALPHA,0041,ALPHA,\"[\"\"ALPHA\"\",\"\"FIRST LETTER\"\"]\"\n",
        "ALPHABET,0042,,[]\n",
        "SECOND LETTER,0043,,[]\n",
        "THIRD LETTER,0044,,[]\n",
    ));
    let aliases = fixture("alpha extra,Z\nthird abbreviated,E\n");
    let result = run(&[
        "--charmap",
        map.to_str().unwrap(),
        "--aliases",
        aliases.to_str().unwrap(),
        "alpha",
        "--combine",
        "second let",
        "--combine",
        "FIRST LETTER",
        "--combine",
        "ALPHA",
        "--combine",
        "third abb",
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, b"ACAAE");
    let missing = run(&[
        "--charmap",
        map.to_str().unwrap(),
        "ALPHA",
        "--combine",
        "MISSING NAME",
    ]);
    assert!(!missing.status.success());
    assert!(missing.stdout.is_empty());
    fs::remove_file(map).unwrap();
    fs::remove_file(aliases).unwrap();
}

#[test]
fn exact_scan_detects_late_ambiguity_and_invalid_records() {
    for data in [
        "na,cp,na1,aliases\nTARGET,0041,,[]\nOTHER,0042,TARGET,[]\n",
        "na,cp,na1,aliases\nTARGET,0041,,[]\nBROKEN,D800,,[]\n",
        "na,cp,na1,aliases\nTARGET,0041,,[]\n\"unterminated",
    ] {
        let map = fixture(data);
        let result = run(&["--charmap", map.to_str().unwrap(), "TARGET"]);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        fs::remove_file(map).unwrap();
    }
}
