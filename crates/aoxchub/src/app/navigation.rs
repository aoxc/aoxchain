pub fn scroll_to_anchor(anchor: &str) {
    let section_id = anchor.trim_start_matches('#');
    let script = format!(
        "const node = document.getElementById('{section_id}'); if (node) {{ node.scrollIntoView({{ behavior: 'smooth', block: 'start' }}); }}"
    );
    dioxus::document::eval(&script);
}
