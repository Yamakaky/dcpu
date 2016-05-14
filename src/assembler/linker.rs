use std::collections::HashMap;

use assembler::types::*;

#[derive(Debug)]
pub enum Error {
    MissingLabel(String),
    UnknownLabel(String),
    LocalBeforeGlobal(String),
}

pub fn link(ast: &[ParsedItem]) -> Result<Vec<u16>, Error> {

    let mut bin = Vec::new();
    let mut labels = extract_labels(ast);
    //let mut last_instruction_idx = 0;
    //let mut last_bin_idx = 0;

    Ok(bin)
}

fn extract_labels(ast: &[ParsedItem]) -> HashMap<String, usize> {
    let mut prev_label = None;
    ast.iter().filter_map(|l| match *l {
        ParsedItem::LabelDecl(ref s) => {
            prev_label = Some(s);
            Some((s.clone(), 0))
        },
        ParsedItem::LocalLabelDecl(ref s) =>
            Some((format!("{}_{}", prev_label.unwrap(), s), 0)),
        _ => None
    }).collect()
}

enum ResolveResult {
    Fixed,
    Between(usize, usize),
}

//impl ParsedInstruction {
//    fn resolve_labels(&self,
//                      labels: &HashMap<String, usize>)
//                    -> Result<ResolveResult, Error> {
//        match *self {
//            ParsedInstruction::BasicOp(op, b, a) => i.resolve_labels(bin, labels),
//            _ => 
//        }
//    }
//}
