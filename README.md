# Crafting Interpreters (in Rust)

## Chapter Notes (as I go)
### Chapter 4
* I had some uncertainty about error handling. The way Rust does errors is one of my favorite things about it, but I was a little unsure how far to let `Err`s bubble up, especially given the way Nystrom's Java implementation does it (a mutable `hasError` attribute in the scanner class.) I think I handled it ok, but reserve the right to go back and change things.
* Nystrom makes `Token` a class with an enum `TokenType` field. I just made it a regular Rust `enum`.
    * The only place this kind of bit me was in [`scanner::ident_t`](../main/src/scanner.rs#L199). Nystrom handles it with a `Map`; since Rust (afaik) doesn't allow partial application of variants[^1], I had to do yet another big `match` statement.
* I also ran into a bit of an unfortunate situation in [`scanner::scan_token`](../main/src/scanner.rs#L69) handling comments and whitespace. Nystrom has an `addToken` method that appends to a mutable list of tokens stored in the scanner class. I decided to go Rustier and iterate + match. The upside to this was that it's more idiomatic and I didn't need all the helper methods he has to manage the index etc. The problem is that, unlike Nystrom, I can't just throw away whitespace and comments, because
  there's otherwise no way to distinguish between "stuff we don't care about" and `eof`. It's not a big hurdle, I'll just have to be sure to filter out `Token::Whitespace` and `Token::Comment` when it's time to build the AST.
* I'm not sure (yet) what the point of having both `lexeme` and `literal` is. Maybe it'll become clearer in later chapters. 
* Once again, Rust's testing story absolutely whips ass. Being able to return an error type from a test is so beautiful, even though you have to add `Ok(())` at the end of every test. If I knew more lua/vimscript, I'd write a macro or whatever that would scaffold `#[test]\nOk(())` for you.
  * The tests are pretty trivial for now, but already caught a few bugs, including 2 places where I'd forgotten to advance the iterator, leading to an endless loop.
* I had initially wanted to do this without any external libraries. If I weren't doing this for my own edification, I'd obviously reach for [nom](https://docs.rs/nom/latest/nom/), but figured (correctly) that doing the parsing by hand would be a satisfying experience.
  * I ended up going with [thiserror](https://docs.rs/thiserror/latest/thiserror/), even though to be perfectly honest [anyhow](https://docs.rs/anyhow/latest/anyhow/) would probably be fine for something this scale. 
  * [rustyline](https://crates.io/crates/rustyline) gave me a few nice things that I think bring Rust's built-in readline stuff up to par with Java's.
  * [peekmore](https://docs.rs/peekmore/latest/peekmore/) addresses [one very specific need](../main/src/scanner.rs#L159)--checking if the character after the `.` when parsing a number is a digit or not. I maybe could've tried to do it with just [Peekable](https://doc.rust-lang.org/stable/std/iter/struct.Peekable.html) but peeking->advancing->peeking->putting stuff back on top of the iterator could've gotten messy quickly, and I'd already pulled in other crates so who cares.

### Chapter 5
This is where things get a little off the rails...Nystrom reaches for "metaprogramming" (actually, code generation) to cut down on boilerplate, which, as I understand it, is a long-storied and proud Java tradition. The Rust equivalent would be macros, which a) I'm not really comfortable with, and b) don't seem necessary. My entire `Expr` + pretty-printer implementations are 99 lines, including some Rust-specific boilerplate/shenanigans. Macros, schmacros.
* Nystrom liberally (ab)uses Java's `object` type/class. The Rust equivalent (I guess???) would be `dyn Any` (or in this case maybe `dyn Debug` or `dyn SomeTraitIMakeUpMyselfForJustThisPurposeWithNoRealPointOtherwise`) and _no thank you_. I checked `Token` and the only "literals" are `String`, `Number`, and `Nil`, so I just made a `Value` enum. It ends up being a little more typing (see [the tests](../main/src/expr.rs#L107)), but it's not onerous.
* Nystrom also goes with a Visitor pattern. I _think_ this (at least how he does it/what he uses it for) maps best onto [Traits](https://doc.rust-lang.org/book/ch10-02-traits.html), but again, I don't _really_ need it here. He goes with Visitor because he (auto)generates a bunch of separate subclasses for his `Expr` types, while I, once again, just used an enum. Writing a whole trait that I know will only ever be implemented by one thing is foolish, however...
* I _did_ do something Visitor-ish, kinda, with [`ExprPrinter`](../main/src/expr.rs#L42). The motivation behind that wasn't an homage to Nystrom, but rather a simple (somewhat hard-won) acknowledgement that passing a mutable string around through potentially recursive methods is a short-cut to Getting-Yelled-At-By-the-Compiler-Town. I scratched my head for a bit, then reached into the bag of patterns, past Visitor, and pulled out Builder, which ends up being invaluable due to the borrow checker. I'm not entirely convinced I actually need all the methods to return `InterpreterResult<Self>` instead of just `Self`, but I ended up adding a [`Fmt`](../main/src/errors.rs#L12) variant to my `InterpreterError` enum to painlessly handle `std::fmt::Error`, and besides, what's a few monads among friends?
* In all honesty, the initial reason I went with `InterpreterResult<Self>` was because I initially thought `build_binary` and `build_unary` were going to have to check whether the `Token` they were passed in _had_ a `lexeme` field. I reread the relevant part of Nystrom's code and thought back to last week when I was writing the `Token` enum, and realized I was mistaken&mdash;Nystrom has `lexeme` as a field on _all_ of his `Token`s, because he does it as class + tag, whereas I'm just using a regular rust enum. So I just went back and wrote `impl std::fmt::Display` for my `Token` enum, and just used that. It's just the pretty-printer; we don't have to care about syntactic correctness until the next chapter.
* Once again, Rust's testing absolutely rules. Nystrom farts out a throwaway `main` method to pretty-print one fairly-complex `Expr`; we can write actual tests to handle all the cases separately so we know it actually works.



[^1]: e.g.
    ```rust
      lazy_static! {
        static ref RESERVED_WORDS: HashMap<String, Token> = ...
      }
      fn ident_maker(s: String, line: usize, t: Option<Token>) -> Token {
        if let Some(t) = RESERVED_WORDS.get(s) {
          t { line, literal: s.clone(), lexeme: s }
        } else {
          Token::Identifier { line, literal: s.clone(), lexeme: s }
        }
      }
    ```
