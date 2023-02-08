use std::ops::Range;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use phf::phf_map;

// TODO: In the future if this language ever gets somewhere add a page in the documentation with a list of errors
// and then add a link to it here.
pub struct Error<'a>(&'a str);

pub enum MessageKind {
    ERROR,
    WARNING,
}

static ERRORS: phf::Map<&'static str, Error> = phf_map! {
    "0001" => Error("Provided keyword did not match the expected keyword."),
    "0002" => Error("Non-Existent keyword.")
};

pub struct ErrorFile<'a> {
    pub name: &'a str,
    pub path: &'a str,
}

pub struct ErrorClient<'a> {
    pub error_code: &'a str,
    pub error: &'a Error<'a>,
    pub files: SimpleFiles<&'a str, String>,
    pub file_id: Option<usize>,
    pub kind: MessageKind,
    pub end_process: bool,
    pub error_span: Option<Range<usize>>,
    pub notes: Vec<&'a str>,
    pub labels: Vec<Label<usize>>,
}

impl<'a> ErrorClient<'a> {
    pub fn new(err: &'a str, kind: MessageKind) -> Self {
        Self {
            error_code: err,
            error: ERRORS.get(err).unwrap(),
            kind,
            files: SimpleFiles::new(),
            file_id: None,
            error_span: None,
            end_process: false,
            notes: vec![],
            labels: vec![],
        }
    }
    pub fn add_label(&mut self, message: Option<&'a str>) -> &mut Self {
        if let (Some(file_id), Some(range)) = (self.file_id, self.error_span.clone()) {
            let label = Label::primary(file_id, range);
            let label = if let Some(message) = message {
                label.with_message(message)
            } else {
                label
            };
            self.labels.push(label);
        }
        self
    }
    pub fn add_note(&mut self, note: &'a str) -> &mut Self {
        self.notes.push(note);
        self
    }
    pub fn set_file(&mut self, name: &'a str, path: &'a str) -> &mut Self {
        self.file_id = Some(self.files.add(name, std::fs::read_to_string(path).unwrap()));
        self
    }
    pub fn set_span(&mut self, span: Range<usize>) -> &mut Self {
        self.error_span = Some(span);
        self
    }
    pub fn end_process(&mut self, toggle: bool) -> &mut Self {
        self.end_process = toggle;
        self
    }

    pub fn build_and_emit(self) {
        let diagnostic = Diagnostic::error()
            .with_code(format!("E{}", self.error_code))
            .with_labels(self.labels)
            .with_message(self.error.0);

        let diagnostic = if self.notes.len() > 0 {
            diagnostic.with_notes(
                self.notes
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>(),
            )
        } else {
            diagnostic
        };

        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();

        term::emit(&mut writer.lock(), &config, &self.files, &diagnostic).unwrap();
        if self.end_process {
            std::process::exit(1);
        }
    }
}
