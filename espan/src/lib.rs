use ansi_term::Colour::{Blue, Red, Yellow};
use std::collections::HashMap;
use std::io::{self, Write};
use std::ops::Range;

pub enum AnnotationType {
    Error,
    Warning,
    Info,
}

pub struct EspanAnnotation {
    pub ty: AnnotationType,
    pub label: Option<String>,
    pub span: Range<usize>,
    pub line: usize,
}

pub struct ESpan {
    pub source_file_name: String,
    pub source: String,
    pub annotations: HashMap<usize, EspanAnnotation>,
}

impl ESpan {
    pub fn emit(self) {
        let mut buffer = String::new();
        let mut lines = self.source.split('\n').enumerate().peekable();
        let blue = Blue.bold();
        while let Some((line, content)) = lines.peek() {
            if let Some(annot) = self.annotations.get(&(line + 1)) {
                let mut annotation_header = String::new();
                let line_starter = blue
                    .paint(format!("{} | ", &annot.line.to_string()))
                    .to_string();
                annotation_header.push_str(&line_starter);
                annotation_header.push_str(&format!("{content}\n"));

                let color = match annot.ty {
                    AnnotationType::Error => Red.bold(),
                    AnnotationType::Warning => Yellow.bold(),
                    AnnotationType::Info => Blue.bold(),
                };

                let arrows = color
                    .paint("^".repeat(annot.span.clone().count()))
                    .to_string();
                let left_padding = " ".repeat(annot.span.start + 1);
                let mut annot_string = String::from(&blue.paint("  |").to_string());
                annot_string.push_str(&left_padding);
                annot_string.push_str(&arrows);

                if let Some(label) = &annot.label {
                    annot_string.push(' ');
                    annot_string.push_str(&color.paint(label).to_string());
                }
                annotation_header.push_str(&annot_string);
                buffer.push_str(&format!("{annotation_header}"));
                lines.next();
                if lines.peek().is_some() {
                    buffer.push('\n');
                }
                continue;
            }
            buffer.push_str(&blue.paint("  | ").to_string());
            buffer.push_str(content);
            lines.next();
            if lines.peek().is_some() {
                buffer.push('\n');
            }
        }
        println!("{buffer}");
    }
}
