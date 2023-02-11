use espan::{AnnotationType, ESpan, EspanAnnotation};
use std::collections::HashMap;
fn main() {
    let file_name = "test.file";
    let file_contents = std::fs::read_to_string(file_name).unwrap();

    let mut espan = ESpan {
        source_file_name: file_name.to_string(),
        source: file_contents,
        annotations: HashMap::new(),
    };

    espan.annotations.insert(
        1,
        EspanAnnotation {
            span: 0..6,
            line: 1,
            ty: AnnotationType::Error,
            label: Some(String::from("This word is whatever")),
        },
    );

    espan.emit();
}
