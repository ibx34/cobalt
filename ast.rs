use crate::Tokens;

pub struct AST {
    pub ast: Vec<Tokens>,
}

impl AST {
    pub fn reconstruct(&self) -> String {
        let mut result = String::new();
        for t in &self.ast {
            match t {
                Tokens::SemiColon => result.push(';'),
                Tokens::Colon => result.push(':'),
                Tokens::DollarSign => result.push('$'),
                Tokens::Period => result.push('.'),
                Tokens::Word(word) => result.push_str(&String::from(word.which.clone())),
                Tokens::String(string) => result.push_str(&format!(r#""{}""#, string.as_str())),
            }
            result.push(' ');
        }
        result
    }
}
