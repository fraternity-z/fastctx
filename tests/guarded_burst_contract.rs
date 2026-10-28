#![cfg(debug_assertions)]

mod common;

use common::{McpSession, mcp_text, normalized, write};
use std::process::Command;

#[test]
fn two_guarded_reads_overlap_and_share_one_real_mcp_connection_budget() {
    let root = tempfile::tempdir().unwrap();
    let codex = root.path().join(".codex");
    std::fs::create_dir_all(&codex).unwrap();
    write(
        &codex.join("config.toml"),
        b"model_provider = 'relay'\n[model_providers.relay]\nname = 'OpenAI'\nbase_url = 'https://relay.example/v1'\n",
    );
    let first = root.path().join("first.txt");
    let second = root.path().join("second.txt");
    let barrier = root.path().join("tool-barrier");
    std::fs::create_dir(&barrier).unwrap();
    let body = (0..2_000)
        .map(|line| format!("{line:04} 0123456789abcdef fedcba9876543210 alpha-beta-gamma-delta\n"))
        .collect::<String>();
    write(&first, &body);
    write(&second, &body);

    let mut command = Command::new(env!("CARGO_BIN_EXE_fastctx"));
    command
        .arg("serve")
        .current_dir(root.path())
        .env("HOME", root.path())
        .env("USERPROFILE", root.path())
        .env("FASTCTX_TEST_TOOL_BARRIER_DIR", &barrier);
    let mut session = McpSession::start(command);
    let first_id = session.begin_call(
        "inspect_local_file",
        serde_json::json!({"file_path": normalized(&first)}),
    );
    let second_id = session.begin_call(
        "inspect_local_file",
        serde_json::json!({"file_path": normalized(&second)}),
    );
    let first_response = session.await_response(first_id);
    let second_response = session.await_response(second_id);

    assert_eq!(std::fs::read_dir(&barrier).unwrap().count(), 2);
    let first_text = mcp_text(&first_response);
    let second_text = mcp_text(&second_response);
    let tokenizer = tiktoken_rs::o200k_base().unwrap();
    let total = tokenizer.encode_with_special_tokens(first_text).len()
        + tokenizer.encode_with_special_tokens(second_text).len();
    assert!(total <= 9_000, "guarded responses used {total} tokens");
    assert!(first_text.contains("Partial:"), "{first_text}");
    assert!(second_text.contains("Partial:"), "{second_text}");
}
