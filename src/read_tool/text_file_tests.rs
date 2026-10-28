use super::*;
use crate::model::ToolContent;

fn budget(value: usize) -> TokenBudget {
    TokenBudget {
        value,
        variable: "FASTCTX_READ_TOKEN_BUDGET",
    }
}

fn text(response: ToolResponse) -> String {
    assert!(!response.is_error, "{response:?}");
    let [ToolContent::Text(text)] = response.content.as_slice() else {
        panic!("expected one text block: {response:?}");
    };
    text.clone()
}

#[test]
fn uncaptured_unicode_and_split_crlf_keep_window_and_total_lines() {
    let input = format!(
        "{}\r\nsecond\r\nchosen😀\r\n{}\n",
        "ignored界".repeat(9000),
        "tail界".repeat(10000),
    );
    for chunk_bytes in [1, 7, 64, 65536] {
        let mut collector = LineCollector::new(3, 1, 4096, true);
        let mut start = 0_usize;
        while start < input.len() {
            let mut end = start.saturating_add(chunk_bytes).min(input.len());
            while !input.is_char_boundary(end) {
                end += 1;
            }
            assert!(collector.push(&input[start..end]));
            start = end;
        }
        collector.finish_eof();
        assert_eq!(
            text(collector.into_response(None, budget(4096))),
            "3\tchosen😀\n\n(Partial: line 3 of 5 shown. Continue with offset=4.)",
            "chunk_bytes={chunk_bytes}",
        );
    }
}

#[test]
fn large_file_head_and_deep_offset_keep_exact_continuations() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("large.txt");
    let input = (1..=80000)
        .map(|line| format!("row {line:05} 界😀\r\n"))
        .collect::<String>();
    fs::write(&path, input).unwrap();
    assert_eq!(
        text(read_text_file(
            &path,
            "fixture",
            Some(1),
            Some(2),
            None,
            None,
            budget(4096)
        )),
        "1\trow 00001 界😀\n2\trow 00002 界😀\n\n(Partial: lines 1-2 of 80001 shown. Continue with offset=3.)",
    );
    assert_eq!(
        text(read_text_file(
            &path,
            "fixture",
            Some(80000),
            Some(2),
            None,
            None,
            budget(4096)
        )),
        "80000\trow 80000 界😀\n80001\t\n\n(Complete: reached end of file; lines 80000-80001 of 80001 shown.)",
    );
}

#[test]
fn malformed_tail_outside_the_window_is_still_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("malformed.txt");
    let mut input = "first\n".repeat(20000).into_bytes();
    input.push(0xff);
    fs::write(&path, input).unwrap();
    for encoding in [None, Some("utf-8")] {
        let response = read_text_file(
            &path,
            "fixture",
            Some(1),
            Some(1),
            encoding,
            None,
            budget(4096),
        );
        assert!(response.is_error, "{response:?}");
    }
}

#[test]
fn transcoded_single_and_batch_windows_keep_their_notes_and_counts() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("encoded.txt");
    let mut utf16 = vec![0xff, 0xfe];
    for unit in "first\r\n中\r\nlast\n".encode_utf16() {
        utf16.extend(unit.to_le_bytes());
    }
    for (input, encoding, label) in [
        (utf16, None, "UTF-16LE"),
        (b"first\r\n\xd6\xd0\r\nlast\n".to_vec(), Some("gbk"), "GBK"),
    ] {
        fs::write(&path, input).unwrap();
        let output = text(read_text_file(
            &path,
            "fixture",
            Some(2),
            Some(1),
            encoding,
            None,
            budget(4096),
        ));
        assert!(output.starts_with("2\t中\n\n"), "{output}");
        assert!(
            output.contains(&format!("decoded from {label}")),
            "{output}"
        );
        assert!(
            output.ends_with("(Partial: line 2 of 4 shown. Continue with offset=3.)"),
            "{output}"
        );
        let batch =
            read_batch_text_file(&path, "fixture", Some(2), Some(1), encoding, None, 4096).unwrap();
        assert_eq!(batch.lines, ["2\t中"]);
        assert_eq!(batch.total_lines, 4);
        assert!(batch.transcoding_note.unwrap().contains(label));
    }
}

#[test]
fn budget_saturation_keeps_the_exact_total_and_resume_offset() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("budget.txt");
    fs::write(
        &path,
        (1..=1000)
            .map(|line| format!("row {line:04} content 😀\n"))
            .collect::<String>(),
    )
    .unwrap();
    let output = text(read_text_file(
        &path,
        "fixture",
        Some(1),
        None,
        None,
        None,
        budget(256),
    ));
    assert!(estimate_tokens(&output) <= 256, "{output}");
    let last = output
        .lines()
        .filter_map(|line| line.split_once('\t')?.0.parse::<usize>().ok())
        .next_back()
        .unwrap();
    assert!(last < 1000);
    assert!(
        output.ends_with(&format!(
            "(Partial: lines 1-{last} of 1001 shown. Continue with offset={}.)",
            last + 1
        )),
        "{output}"
    );
}

#[test]
fn captured_long_lines_still_count_unicode_and_strip_crlf() {
    let mut collector = LineCollector::new(2, 1, 65536, true);
    let long = "界".repeat(MAX_LINE_CHARS + 9);
    assert!(collector.push(&format!("skip\r\n{long}\r")));
    assert!(collector.push("\ntail\n"));
    collector.finish_eof();
    assert_eq!(
        text(collector.into_response(None, budget(65536))),
        format!(
            "2\t{}... [line truncated: {} chars total]\n\n(Partial: line 2 of 4 shown. Continue with offset=3.)",
            "界".repeat(MAX_LINE_CHARS),
            MAX_LINE_CHARS + 9
        ),
    );
}

#[test]
fn unterminated_lines_and_cr_only_endings_keep_eof_counts() {
    for ending in ["last", "last\r"] {
        let mut collector = LineCollector::new(2, 1, 4096, true);
        assert!(collector.push(&format!("skip\n{ending}")));
        collector.finish_eof();
        assert_eq!(
            text(collector.into_response(None, budget(4096))),
            "2\tlast\n\n(Complete: reached end of file; line 2 of 2 shown.)",
        );
    }

    let long = "uncaptured界".repeat(20000);
    let mut collector = LineCollector::new(1, 1, 4096, true);
    assert!(collector.push(&format!("first\n{long}\r")));
    collector.finish_eof();
    assert_eq!(
        text(collector.into_response(None, budget(4096))),
        "1\tfirst\n\n(Partial: line 1 of 2 shown. Continue with offset=2.)",
    );

    let mut collector = LineCollector::new(2, 1, 4096, true);
    assert!(collector.push(&long));
    collector.finish_eof();
    assert_eq!(
        text(collector.into_response(None, budget(4096))),
        "Warning: the file has only 1 line, but offset=2 was requested.",
    );
}
