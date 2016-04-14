use std::collections::HashSet;

use types::*;
use parser::*;

#[derive(Debug)]
pub enum Error {
    MissingLabel(String),
    UnknownLabel(String)
}

pub fn resolve_labels(ast: &[ParsedItem]) -> Result<Vec<u16>, Error> {
    //let labels = ast.iter()
    //    .enumerate()
    //    .filter_map(|(i, ref item)| match *item {
    //        ParsedItem::LabelDecl(_) | ParsedItem::LocalLabelDecl(_) => Some((i, item.clone())),
    //        _ => None
    //    }).co;

    //let mut bin = Vec::new();
    //let mut last_instruction_idx = 0;
    //let mut last_bin_idx = 0;

    //Ok(bin)
    let mut bin = [0; 3];
    let i = Instruction::BasicOp(BasicOp::SET, Value::PC, Value::Litteral(0));
    i.encode(&mut bin);
    Ok(vec!(bin[0]))
}
