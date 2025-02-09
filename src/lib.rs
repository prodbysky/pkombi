pub type ParserFunction<'a, I, O> = dyn Fn(I) -> Option<(O, I)> + 'a;

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
        F: Fn(&'a [I]) -> Option<(O, &'a [I])> + 'a,
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
        Parser::new(move |input: &'a [I]| {
            if let Some((p, r)) = self.0(input) {
                Some((Some(p), r))
            } else {
                Some((None, input))
            }
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
        Parser::new(move |input: &'a [I]| {
            if let Some((p1, r)) = self.0(input) {
                if let Some((p2, r)) = other.0(r) {
                    return Some(((p1, p2), r));
                }
            }
            None
        })
    }

    /// This combinator first matches the `self` parser and then tries to match the second one and
    /// if it doesn't match then it doesn't fail (when compared to the `and` combinator)
    pub fn then_maybe<O2: 'a>(self, other: Parser<'a, I, O2>) -> ThenMaybe<'a, I, O, O2> {
        Parser::new(move |input| {
            if let Some((p1, r1)) = self.0(input) {
                if let Some((p2, r2)) = other.0(r1) {
                    return Some(((p1, Some(p2)), r2));
                }
                return Some(((p1, None), r1));
            }
            None
        })
    }

    /// Matches zero or more elements based on the inside parser
    pub fn many(self) -> Many<'a, I, O> {
        Parser::new(move |mut input: &'a [I]| {
            let mut elements = vec![];
            while let Some((p, r)) = self.0(input) {
                elements.push(p);
                input = r;
            }
            Some((elements, input))
        })
    }

    /// Matches atleast one or more elements based on the inside parser
    pub fn many1(self) -> Many1<'a, I, O> {
        Parser::new(move |mut input: &'a [I]| {
            let mut elements = vec![];
            while let Some((p, r)) = self.0(input) {
                elements.push(p);
                input = r;
            }
            if elements.is_empty() {
                Some((None, input))
            } else {
                Some((Some(elements), input))
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

    pub fn parse(self, input: &'a [I]) -> Option<(O, &'a [I])> {
        self.0(input)
    }
}

pub fn char<'a>(c: char) -> StringParser<'a, char> {
    Parser::new(move |input: &[char]| match input.split_at_checked(1) {
        Some((p, r)) if !p.is_empty() && p[0] == c => Some((p[0], r)),
        _ => None,
    })
}

pub fn digit<'a>() -> StringParser<'a, char> {
    Parser::new(move |input: &[char]| match input.split_at_checked(1) {
        Some((p, r)) if !p.is_empty() && p[0].is_ascii_digit() => Some((p[0], r)),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn xd_parser() {
        let float_parser = super::digit()
            .many()
            .maybe()
            .then_maybe(super::char('.').and(super::digit().many()));

        let input: Vec<_> = ".123".chars().collect();
        let (parsed, _) = float_parser.parse(&input).unwrap();
        dbg!(parsed);
    }
}
