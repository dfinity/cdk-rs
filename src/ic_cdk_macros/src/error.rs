use proc_macro2::Span;
use syn::spanned::Spanned;

type Error = syn::Error;

#[derive(Clone, Default)]
pub(crate) struct Errors(Vec<Error>);

/// A list of errors which will be sent to the user
impl Errors {
    pub fn message<M: Into<String>>(msg: M) -> Errors {
        Errors::single(msg, Span::call_site())
    }

    pub fn single<S: Spanned, M: Into<String>>(msg: M, s: S) -> Errors {
        Errors::from(Error::new(s.span(), msg.into()))
    }

    pub fn emit(self) {
        if !self.0.is_empty() {
            let errors: Vec<String> = self
                .0
                .iter()
                .map(|e| format!("{}", e.to_string()))
                .collect();

            panic!(errors.join("\n"))
        }
    }
}

impl From<Error> for Errors {
    fn from(e: Error) -> Self {
        Self(vec![e])
    }
}
