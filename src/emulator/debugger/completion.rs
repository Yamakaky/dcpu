use std::collections::BTreeSet;

use assembler::types::Globals;

use rustyline;

pub struct DebuggerCompleter {
    symbols: Vec<String>,
}

impl DebuggerCompleter {
    pub fn new(globals: &Globals) -> DebuggerCompleter {
        DebuggerCompleter {
            symbols: globals.keys().cloned().collect(),
        }
    }
}

impl rustyline::completion::Completer for DebuggerCompleter {
    fn complete(&self, line: &str, pos: usize)
        -> rustyline::Result<(usize, Vec<String>)> {

        let break_chars = {
            let mut set = BTreeSet::new();
            set.insert(' ');
            set
        };
        let (i, word) = rustyline::completion::extract_word(line,
                                                            pos,
                                                            &break_chars);
        let completions = self.symbols
                              .iter()
                              .filter(|cmd| cmd.starts_with(word))
                              .cloned()
                              .map(|s| (*s).into())
                              .collect();
        Ok((i, completions))
    }
}
