use assembler::types::*;

pub fn clean(ast: Vec<ParsedItem>) -> Vec<ParsedItem> {
    let used_labels: Vec<String> = ast.clone().into_iter().flat_map(|i| i.used_labels().into_iter()).collect();
    let mut res = vec![];
    let mut keep = true;
    for item in ast {
        match &item {
            &ParsedItem::LabelDecl(ref l) |
                &ParsedItem::Directive(Directive::Lcomm(ref l, _))=> {
                keep = used_labels.contains(&l);
                if !keep {
                    println!("Removing {}", l);
                }
            }
            _ => (),
        }
        if keep {
            res.push(item);
        }
    }
    res
}

impl ParsedItem {
    fn used_labels(&self) -> Vec<String> {
        match *self {
            ParsedItem::Directive(ref d) => d.used_labels(),
            ParsedItem::Comment(_) |
                ParsedItem::LabelDecl(_) |
                ParsedItem::LocalLabelDecl(_) => vec![],
            ParsedItem::Instruction(ref i) => i.used_labels(),
        }
    }
}

impl Directive {
    fn used_labels(&self) -> Vec<String> {
        use assembler::types::Directive::*;

        match *self {
            Dat(ref ds) => ds.clone().into_iter().flat_map(|d| d.used_labels()).collect(),
            Lcomm(..) | Org(..) | Skip(..) | Global | Text | BSS => vec![],
        }
    }
}

impl DatItem {
    fn used_labels(&self) -> Vec<String> {
        match *self {
            DatItem::E(ref e) => e.used_labels(),
            DatItem::S(..) => vec![],
        }
    }
}

impl Instruction<Expression> {
    fn used_labels(&self) -> Vec<String> {
        match *self {
            Instruction::BasicOp(_, ref b, ref a) => {
                let mut x = a.used_labels();
                x.extend(b.used_labels());
                x
            }
            Instruction::SpecialOp(_, ref a) => a.used_labels(),
        }
    }
}

impl Value<Expression> {
    fn used_labels(&self) -> Vec<String> {
        use types::Value::*;

        match *self {
            AtRegPlus(_, ref e) | Pick(ref e) | AtAddr(ref e) | Litteral(ref e) => {
                e.used_labels()
            }
            _ => vec![],
        }
    }
}

impl Expression {
    fn used_labels(&self) -> Vec<String> {
        use assembler::types::Expression::*;

        match *self {
            Label(ref l) => vec![l.clone()],
            Not(ref e) => e.used_labels(),
            Add(ref a, ref b) |
                Sub(ref a, ref b) |
                Mul(ref a, ref b) |
                Div(ref a, ref b) |
                Shr(ref a, ref b) |
                Shl(ref a, ref b) |
                Mod(ref a, ref b) |
                Less(ref a, ref b) |
                Equal(ref a, ref b) |
                Greater(ref a, ref b) => {
                    let mut x = a.used_labels();
                    x.extend(b.used_labels());
                    x
                }
            LocalLabel(_) | Num(_) => vec![],
        }
    }
}
