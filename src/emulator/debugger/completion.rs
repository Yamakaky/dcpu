use std::collections::BTreeSet;

use rustyline;

pub struct DebuggerCompleter;

impl rustyline::completion::Completer for DebuggerCompleter {
    fn complete(&self, line: &str, pos: usize)
        -> rustyline::Result<(usize, Vec<String>)> {

        let break_chars = {
            let mut set = BTreeSet::new();
            set.insert(' ');
            set
        };
        let cmds = [
            "r", "x", "b", "s", "c",
            "devices",
            "disassemble",
            "breakpoints",
            "delete",
            "hook",
            "logs",
        ];
        let (i, word) = rustyline::completion::extract_word(line,
                                                            pos,
                                                            &break_chars);
        let completions = cmds.iter()
                              .filter(|cmd| cmd.starts_with(word))
                              .cloned()
                              .map(|s| (*s).into())
                              .collect();
        Ok((i, completions))
    }
}
