use rustyline::error::ReadlineError;
use rustyline::Editor;

pub struct Prompt {
    rl: Editor<()>,
    prompt: String,
}

impl Prompt {
    pub fn new<T>(prompt: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            rl: Editor::<()>::new(),
            prompt: prompt.into(),
        }
    }
}

impl Iterator for Prompt {
    type Item = Result<String, ReadlineError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.rl.readline(&self.prompt) {
            Ok(l) => Some(Ok(l)),
            Err(ReadlineError::Eof) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
