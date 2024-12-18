use std::{collections::HashMap, io};
use url::Url;

use crate::{diagnostic::Diagnostic, lookup::Lookup};

use super::Reporter;

pub struct LspReporter {
    diagnostics: Vec<Diagnostic>,
    lookups: HashMap<Url, Lookup>,
}

impl LspReporter {
    pub fn insert_file(&mut self, url: Url, contents: String) {
        self.lookups.insert(url, Lookup::new(contents));
    }
}
