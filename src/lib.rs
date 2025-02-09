pub type Parser<I, O> = Box<dyn FnOnce(I) -> Option<(O, I)>>;

fn satisfy_char<'a, F: Fn(char) -> bool + 'static>(pred: F) -> Parser<&'a [char], char> {
    Box::new(move |input: &'a [char]| {
        if input.is_empty() {
            return None;
        }
        match input.split_at(1) {
            (p, r) if pred(p[0]) => Some((p[0], r)),
            _ => None,
        }
    })
}

pub fn any_char<'a>() -> Parser<&'a [char], char> {
    satisfy_char(|_| true)
}

pub fn char<'a>(c: char) -> Parser<&'a [char], char> {
    satisfy_char(move |cc| cc == c)
}

pub fn any_digit<'a>() -> Parser<&'a [char], char> {
    satisfy_char(move |cc| cc.is_digit(10))
}
pub fn digit<'a>(d: char) -> Parser<&'a [char], char> {
    satisfy_char(move |cc| cc.is_digit(10) && cc == d)
}

#[cfg(test)]
mod tests {
    use super::*;
}
