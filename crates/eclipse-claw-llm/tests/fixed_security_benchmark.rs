use std::fs;
use std::path::Path;

use eclipse_claw_llm::guard::{
    UNTRUSTED_CONTENT_RULE, guarded_system_prompt, wrap_untrusted_content,
};

#[test]
fn hostile_public_fixture_cannot_close_the_llm_data_boundary() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/public-pages/article.html");
    let content = fs::read_to_string(fixture).expect("read hostile benchmark fixture");
    let wrapped = wrap_untrusted_content(&content);

    assert_eq!(wrapped.matches("</untrusted_web_content>").count(), 1);
    assert!(wrapped.contains("&lt;/untrusted_web_content&gt;"));
    assert!(guarded_system_prompt("Summarize facts.").contains(UNTRUSTED_CONTENT_RULE));
}
