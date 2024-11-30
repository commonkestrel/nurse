use std::collections::HashMap;

use crate::{diagnostic::Diagnostic, lookup::Lookup};

pub trait Reporter {
    type Emitter: ?Sized;

    fn report(&mut self, diagnostic: Diagnostic);
    fn report_all(&mut self, diagnostics: Vec<Diagnostic>) {
        for diagnostic in diagnostics {
            self.report(diagnostic);
        }
    }

    fn emit(&self, emitter: &mut Self::Emitter, diagnostic: Diagnostic);
    fn emit_all(&mut self, emitter: &mut Self::Emitter);

    fn has_errors(&self) -> bool;
    fn is_empty(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SerialReporter {
    diagnostics: Vec<Diagnostic>,
    lookups: HashMap<String, Lookup>,
}

impl SerialReporter {
    pub fn new() -> SerialReporter {
        Self::default()
    }

    pub fn register_file(&mut self, name: String, contents: String) {
        self.lookups.insert(name, Lookup::new(contents));
    }
}

impl Reporter for SerialReporter {
    type Emitter = dyn std::io::Write;

    fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn report_all(&mut self, mut diagnostics: Vec<Diagnostic>) {
        self.diagnostics.append(&mut diagnostics);
    }

    fn emit(&self, emitter: &mut Self::Emitter, diagnostic: Diagnostic) {
        todo!()
    }

    fn emit_all(&mut self, emitter: &mut Self::Emitter) {
        for diagnostic in self.diagnostics.drain(..) {
            todo!()
        }
    }

    fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|diagnostic| diagnostic.is_error())
    }

    fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

impl Default for SerialReporter {
    fn default() -> Self {
        SerialReporter {
            diagnostics: Vec::new(),
            lookups: HashMap::new(),
        }
    }
}
