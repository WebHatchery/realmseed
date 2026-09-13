// The shared file-size gate from CODE_STANDARDS §2.2 — every physical line,
// including tests — is enforced under plain `cargo test`.

#[test]
fn source_files_stay_under_the_limit() {
    macroquad_toolkit::source_gate::assert_source_files_within_limit(
        env!("CARGO_MANIFEST_DIR"),
        &[],
    );
}
