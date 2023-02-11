#[derive(Debug, PartialEq, Eq)]
pub struct Word {
    /// Whether or not the word should be treated as plural
    pub plural: bool,
    /// Which word is it?
    pub which: Words,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Words {
    Define,
    Module,
    Function,
    Call,
    Equal,
    Argument,
    The,
    With,
    Contents,
    End,
    Is,
    To,
    Set,
    A,
    Expects,
    That,
    Returns,
    Contains,
    Display,
    If,
    Then,
    Do,
    True,
    False,
    Begin,
    Program,
}

impl TryFrom<&str> for Words {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "define" => Ok(Self::Define),
            "module" => Ok(Self::Module),
            "function" => Ok(Self::Function),
            "call" => Ok(Self::Call),
            "equal" => Ok(Self::Equal),
            "argument" => Ok(Self::Argument),
            "the" => Ok(Self::The),
            "with" => Ok(Self::With),
            "contents" => Ok(Self::Contents),
            "contains" => Ok(Self::Contains),
            "end" => Ok(Self::End),
            "is" => Ok(Self::Is),
            "to" => Ok(Self::To),
            "set" => Ok(Self::Set),
            "a" => Ok(Self::A),
            "expects" => Ok(Self::Expects),
            "that" => Ok(Self::That),
            "returns" => Ok(Self::Returns),
            "display" => Ok(Self::Display),
            "if" => Ok(Self::If),
            "then" => Ok(Self::Then),
            "do" => Ok(Self::Do),
            "true" => Ok(Self::True),
            "false" => Ok(Self::False),
            "begin" => Ok(Self::Begin),
            "program" => Ok(Self::Program),
            _ => Err(String::from("Ye bad")),
        }
    }
}

impl From<Words> for String {
    fn from(src: Words) -> String {
        match src {
            Words::Define => "define",
            Words::Module => "module",
            Words::Function => "function",
            Words::Call => "call",
            Words::Equal => "equal",
            Words::Argument => "argument",
            Words::The => "the",
            Words::With => "with",
            Words::Contents => "contents",
            Words::End => "end",
            Words::Is => "is",
            Words::To => "to",
            Words::Set => "set",
            Words::A => "a",
            Words::Expects => "expects",
            Words::That => "that",
            Words::Contains => "contains",
            Words::Returns => "returns",
            Words::Display => "display",
            Words::If => "if",
            Words::Then => "then",
            Words::Do => "do",
            Words::True => "true",
            Words::False => "false",
            Words::Program => "program",
            Words::Begin => "begin",
        }
        .to_ascii_uppercase()
    }
}
