# PKombi - Parser combinator library in Rust

## Why?
Well first of all because I can.
Also because I want to understand Rust better,
I want to understand how parser combinators work under the hood.
Also this is quite the functional thingamabobo

## Currently implemented parsers and their combinators
### Parsers
- char(c): matches a single character
- digit(): matches a any ascii base 10 digit
### Combinators
- skip(): skips the matched input
- maybe(): optional parse result
- or(other): first tries the `self` parser and if it fails then tries `other`
- and(other): tries to match both the `self` and the `other` parser
- then_maybe(other): optional `other` parser match
- many(): matches 0 or more elements
- many1(): matches atleast 1 or more elements
- choice(possibilities): matches against the provided parsers and returns the first valid match
