#[derive(Debug, Clone)]
pub struct EbnfDiagram {
    pub title: Option<String>,
    pub comment: Option<String>,
    pub rules: Vec<EbnfRule>,
}

#[derive(Debug, Clone)]
pub struct EbnfRule {
    pub name: String,
    pub expr: EbnfExpr,
}

#[derive(Debug, Clone)]
pub enum EbnfExpr {
    Terminal(String),
    NonTerminal(String),
    Sequence(Vec<EbnfExpr>),
    Alternation(Vec<EbnfExpr>),
    Optional(Box<EbnfExpr>),
    Repetition(Box<EbnfExpr>),
    Group(Box<EbnfExpr>),
    Special(String),
}
