pub type ParserFunction<'a, I, O> = dyn Fn(I) -> Option<(O, Option<I>)> + 'a;

pub struct Parser<'a, I, O>(Box<ParserFunction<'a, &'a [I], O>>);
pub type StringParser<'a, O> = Parser<'a, char, O>;

pub type ThenMaybe<'a, I, O, O2> = Parser<'a, I, (O, Option<O2>)>;
pub type And<'a, I, O, O2> = Parser<'a, I, (O, O2)>;
pub type Many<'a, I, O> = Parser<'a, I, Vec<O>>;
pub type Many1<'a, I, O> = Parser<'a, I, Option<Vec<O>>>;
pub type Or<'a, I, O> = Parser<'a, I, O>;
pub type Skip<'a, I> = Parser<'a, I, ()>;

impl<'a, I: 'a, O: 'a> Parser<'a, I, O> {
    /// Create a new parser from the specified function
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&'a [I]) -> Option<(O, Option<&'a [I]>)> + 'a,
    {
        Self(Box::new(f))
    }

    /// This parser just skips the parsed input by consuming the string and returning unit in the
    /// output field
    pub fn skip(self) -> Skip<'a, I> {
        Parser::new(move |input: &'a [I]| {
            if let Some((_p, r)) = self.0(input) {
                Some(((), r))
            } else {
                None
            }
        })
    }

    /// If the function doesnt match then this parser doesn't consume the input and passes it
    /// forwards
    pub fn maybe(self) -> Parser<'a, I, Option<O>> {
        Parser::new(move |input: &'a [I]| match self.0(input) {
            Some((p, Some(r))) => Some((Some(p), Some(r))),
            Some((p, None)) => Some((Some(p), None)),
            None => Some((None, Some(input))),
        })
    }

    pub fn or(self, other: Parser<'a, I, O>) -> Or<'a, I, O> {
        Parser::new(move |input: &'a [I]| {
            if let Some((p, r)) = self.0(input) {
                return Some((p, r));
            }
            other.0(input)
        })
    }

    /// This combinator requires to match both parsers and if it doesn't match then it will fail
    pub fn and<O2: 'a>(self, other: Parser<'a, I, O2>) -> And<'a, I, O, O2> {
        Parser::new(move |input: &'a [I]| match self.0(input) {
            Some((p1, Some(r))) => match other.0(r) {
                Some((p2, r)) => Some(((p1, p2), r)),
                None => None,
            },
            Some((_p1, None)) => None,
            None => None,
        })
    }

    /// This combinator first matches the `self` parser and then tries to match the second one and
    /// if it doesn't match then it doesn't fail (when compared to the `and` combinator)
    pub fn then_maybe<O2: 'a>(self, other: Parser<'a, I, O2>) -> ThenMaybe<'a, I, O, O2> {
        Parser::new(move |input| match self.0(input) {
            Some((p1, Some(r))) => match other.0(r) {
                Some((p2, r1)) => Some(((p1, Some(p2)), r1)),
                None => Some(((p1, None), Some(r))),
            },
            Some((p1, None)) => Some(((p1, None), None)),
            None => None,
        })
    }

    /// Matches zero or more elements based on the inside parser
    pub fn many(self) -> Many<'a, I, O> {
        Parser::new(move |mut input: &'a [I]| {
            let mut elements = vec![];
            while let Some((p, r)) = self.0(input) {
                elements.push(p);
                match r {
                    Some(r) => input = r,
                    None => return Some((elements, None)),
                }
            }
            Some((elements, Some(input)))
        })
    }

    /// Matches atleast one or more elements based on the inside parser
    pub fn many1(self) -> Many1<'a, I, O> {
        Parser::new(move |mut input: &'a [I]| {
            let mut elements = vec![];
            while let Some((p, r)) = self.0(input) {
                elements.push(p);
                match r {
                    Some(r) => input = r,
                    None => return Some((Some(elements), None)),
                }
            }
            if elements.is_empty() {
                None
            } else {
                Some((Some(elements), Some(input)))
            }
        })
    }

    /// Tries the combinators in order, and either returns the first match or None
    pub fn choice(possibilities: Vec<Parser<'a, I, O>>) -> Parser<'a, I, O> {
        Parser::new(move |input: &[I]| {
            for parser in &possibilities {
                if let Some((p, r)) = parser.0(input) {
                    return Some((p, r));
                }
            }
            return None;
        })
    }

    pub fn map<F, NewO: 'a>(self, f: F) -> Parser<'a, I, NewO>
    where
        F: Fn(O) -> NewO + 'a,
    {
        Parser::new(move |input: &[I]| self.0(input).map(|(o, r)| (f(o), r)))
    }
    pub fn filter<F>(self, f: F) -> Parser<'a, I, O>
    where
        F: Fn(&O) -> bool + 'a,
    {
        Parser::new(move |input: &[I]| self.0(input).filter(|(o, _r)| f(o)))
    }

    pub fn parse(&self, input: &'a [I]) -> Option<(O, Option<&'a [I]>)> {
        self.0(input)
    }
}

pub fn satisfy<'a, F>(f: F) -> StringParser<'a, char>
where
    F: Fn(char) -> bool + 'a,
{
    Parser::new(move |input: &[char]| match input.split_at_checked(1) {
        Some((p, r)) if !p.is_empty() && f(p[0]) && !r.is_empty() => Some((p[0], Some(r))),
        Some((p, r)) if !p.is_empty() && f(p[0]) && r.is_empty() => Some((p[0], None)),
        _ => None,
    })
}

pub fn char<'a>(c: char) -> StringParser<'a, char> {
    Parser::new(move |input: &[char]| match input.split_at_checked(1) {
        Some((p, r)) if !p.is_empty() && p[0] == c && !r.is_empty() => Some((p[0], Some(r))),
        Some((p, r)) if !p.is_empty() && p[0] == c && r.is_empty() => Some((p[0], None)),
        _ => None,
    })
}

pub fn digit<'a>() -> StringParser<'a, char> {
    Parser::new(move |input: &[char]| match input.split_at_checked(1) {
        Some((p, r)) if !p.is_empty() && p[0].is_ascii_digit() && !r.is_empty() => {
            Some((p[0], Some(r)))
        }
        Some((p, r)) if !p.is_empty() && p[0].is_ascii_digit() && r.is_empty() => {
            Some((p[0], None))
        }
        _ => None,
    })
}

pub trait CollectChars {
    fn into_string(&self) -> String;
}

impl CollectChars for char {
    fn into_string(&self) -> String {
        self.to_string()
    }
}

impl CollectChars for Vec<char> {
    fn into_string(&self) -> String {
        self.iter().collect()
    }
}

impl<T: CollectChars> CollectChars for Option<T> {
    fn into_string(&self) -> String {
        self.as_ref().map(|t| t.into_string()).unwrap_or_default()
    }
}

impl<A: CollectChars, B: CollectChars> CollectChars for (A, B) {
    fn into_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&self.0.into_string());
        s.push_str(&self.1.into_string());
        s
    }
}

impl<'a, I: 'a, O: CollectChars + 'a> Parser<'a, I, O> {
    pub fn into_string(self) -> Parser<'a, I, String> {
        self.map(|o| o.into_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn single_char() {
        let c_parser = char('c').into_string();
        assert_eq!(c_parser.parse(&['c']).unwrap(), ("c".to_string(), None))
    }
    #[test]
    fn single_digit() {
        let digit_parser = digit().into_string();
        assert_eq!(digit_parser.parse(&['1']).unwrap(), ("1".to_string(), None))
    }
    #[test]
    fn or() {
        let c_parser = char('c');
        let digit_parser = digit();
        let c_or_digit_parser = c_parser.or(digit_parser).into_string();
        assert_eq!(
            c_or_digit_parser.parse(&['c']),
            Some(("c".to_string(), None))
        );
        assert_eq!(
            c_or_digit_parser.parse(&['1']),
            Some(("1".to_string(), None))
        );
        assert_eq!(c_or_digit_parser.parse(&['a']), None);
    }

    #[test]
    fn and() {
        let c_parser = char('c');
        let d_parser = char('d');
        let c_and_d_parser = c_parser.and(d_parser).into_string();
        let c: &[char] = &['c'];
        assert_eq!(
            c_and_d_parser.parse(&['c', 'd']),
            Some(("cd".to_string(), None))
        );
        assert_eq!(c_and_d_parser.parse(&['c']), None);
        assert_eq!(c_and_d_parser.parse(&['c', 'c']), None);
        assert_eq!(
            c_and_d_parser.parse(&['c', 'd', 'c']),
            Some(("cd".to_string(), Some(c)))
        );
        assert_eq!(c_and_d_parser.parse(&['a']), None);
    }

    #[test]
    fn then_maybe() {
        let c_parser = char('c');
        let d_parser = char('d');
        let c: &[char] = &['c'];
        let c_and_then_maybe_d_parser = c_parser.then_maybe(d_parser).into_string();
        assert_eq!(
            c_and_then_maybe_d_parser.parse(&['c', 'd']),
            Some(("cd".to_string(), None))
        );
        assert_eq!(
            c_and_then_maybe_d_parser.parse(&['c', 'c']),
            Some(("c".to_string(), Some(c)))
        );
        assert_eq!(
            c_and_then_maybe_d_parser.parse(&['c', 'd', 'c']),
            Some(("cd".to_string(), Some(c)))
        );
        assert_eq!(c_and_then_maybe_d_parser.parse(&['d', 'c']), None);
    }
    #[test]
    fn many() {
        let many_c_parser = char('c').many().into_string();
        let d: &[char] = &['d'];
        assert_eq!(
            many_c_parser.parse(&['c', 'c', 'c']),
            Some(("ccc".to_string(), None))
        );
        assert_eq!(
            many_c_parser.parse(&['c', 'c', 'd']),
            Some(("cc".to_string(), Some(d)))
        );
        assert_eq!(
            many_c_parser.parse(&['c', 'd']),
            Some(("c".to_string(), Some(d)))
        );
    }

    #[test]
    fn many1() {
        let many1_c_parser = char('c').many1().into_string();
        assert_eq!(
            many1_c_parser.parse(&['c', 'c', 'c']),
            Some(("ccc".to_string(), None))
        );
        assert_eq!(many1_c_parser.parse(&['d']), None);
    }

    #[test]
    fn identifier() {
        let ident_parser = satisfy(|c: char| c.is_alphabetic() || c == '_')
            .then_maybe(satisfy(|c: char| c.is_alphanumeric() || c == '_').many())
            .into_string();

        assert_eq!(
            ident_parser.parse(&['h', 'e', 'l', 'l', 'o', '_', 'w', 'o', 'r', 'l', 'd']),
            Some(("hello_world".to_string(), None))
        );
    }
}
