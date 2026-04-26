// Shared HTML raw-text safety helpers for inline <script> and <style> emission.
pub(crate) fn sanitize_html_raw_text_end_tag(source: &str, tag_name: &str) -> String {
    let source_bytes = source.as_bytes();
    let tag_name_bytes = tag_name.as_bytes();
    let mut output = String::with_capacity(source.len());
    let mut copied_until = 0;
    let mut index = 0;

    while index + 2 + tag_name_bytes.len() <= source_bytes.len() {
        if source_bytes[index] == b'<'
            && source_bytes[index + 1] == b'/'
            && source_bytes[index + 2..index + 2 + tag_name_bytes.len()]
                .eq_ignore_ascii_case(tag_name_bytes)
        {
            output.push_str(&source[copied_until..index + 1]);
            output.push_str("\\/");
            copied_until = index + 2;
        }
        index += 1;
    }

    if copied_until == 0 {
        source.to_owned()
    } else {
        output.push_str(&source[copied_until..]);
        output
    }
}

pub(crate) fn sanitize_inline_script_source(source: &str) -> String {
    sanitize_html_raw_text_end_tag(source, "script")
}

pub(crate) fn sanitize_inline_style_source(source: &str) -> String {
    sanitize_html_raw_text_end_tag(source, "style")
}
