#![feature(proc_macro_diagnostic, proc_macro_tracked_env)]
use proc_macro::{Diagnostic, Level, Literal, TokenStream, TokenTree, tracked_env};

const DEFAULT_FALLBACK_VALUE: &str = "DEFAULT-FALLBACK-VALUE";

/// Internal helper for building error diagnostics.
macro_rules! err_diag {
    ($span:expr, $fmt:literal, $($tts:tt)*) => {{
        let messg = format!($fmt, $($tts)*);
        Diagnostic::spanned($span, Level::Error, messg)
    }};
    ($span:expr, $messg:expr) => {{
        Diagnostic::spanned($span, Level::Error, $messg)
    }};
    ($fmt:literal, $($tts:tt)*) => {{
        let messg = format!($fmt, $($tts)*);
        Diagnostic::spanned(proc_macro::Span::call_site(), Level::Error, messg)
    }};
    ($messg:expr) => {{
        Diagnostic::spanned(proc_macro::Span::call_site(), Level::Error, $messg)
    }};
}

#[cfg(feature = "env-fallback-warn")]
macro_rules! emit_warn_diag {
    ($span:expr, $fmt:literal, $($tts:tt)*) => {{
        let messg = format!($fmt, $($tts)*);
        Diagnostic::spanned($span, Level::Warning, messg).emit()
    }};
    ($span:expr, $messg:expr) => {{
        Diagnostic::spanned($span, Level::Warning, $messg).emit()
    }};
    ($fmt:literal, $($tts:tt)*) => {{
        let messg = format!($fmt, $($tts)*);
        Diagnostic::spanned(proc_macro::Span::call_site(), Level::Warning, messg).emit()
    }};
    ($messg:expr) => {{
        Diagnostic::spanned(proc_macro::Span::call_site(), Level::Warning, $messg).emit()
    }};
}

#[cfg(not(feature = "env-fallback-warn"))]
macro_rules! emit_warn_diag {
    ($($tts:tt)*) => {};
}

#[proc_macro]
pub fn fallback_env(ts: TokenStream) -> TokenStream {
    match release_env_debug(ts) {
        Ok(lit) => TokenTree::Literal(lit).into(),
        Err(diag) => {
            diag.emit();
            TokenTree::Literal(Literal::string("")).into()
        }
    }
}

/// Attempts to grab an environment variable at compile time (like [`env!`]), but only if
/// compiled with `--release`.
///
/// When not compiled with `--release`, it will still check for the main environment variable, but
/// if it's missing, it uses a fallback environment variable or value. To specify that a fallback
/// environment variable is used, qualify the fallback with `env:`
///
/// ```ignore
/// # #[macro_use]
/// # extern crate env_macros;
/// # fn main() {
/// // If running in release, `$TOKEN` will be a required variable, but if not, `$TEST_TOKEN` will
/// // be used instead. If `$TEST_TOKEN` is also unset, then this will trigger a compiliation
/// // error.
/// static TOKEN: &str = release_env!("TOKEN", env: "TEST_TOKEN");
/// # }
/// ```
///
/// Similarly, to use a value as a fallback, qualify the 2nd argument with a `value:` annotation
///
/// ```
/// # #[macro_use]
/// # extern crate env_macros;
/// # fn main() {
/// // If '$TOKEN' isn't set, the value of this will be "TOKEN_FALLBACK". Since we have a known
/// // value, this variant cannot cause a compiler error (when running in non-release mode).
/// static TOKEN: &str = release_env!("TOKEN", value: "TOKEN_FALLBACK");
/// # }
/// ```
///
/// If a fallback value/environment variable isn't specified, a defualt fallback value of
/// `DEFAULT-DEBUG-FALLBACK` will be used.
///
/// ```
/// # #[macro_use]
/// # extern crate env_macros;
/// # fn main() {
/// // If '$TOKEN' isn't set, then this will expand to the value "DEFAULT-DEBUG-FALLBACK".
/// static TOKEN: &str = release_env!("TOKEN");
/// # }
/// ```
///
/// Under the hood this uses `#[cfg(debug_assertions)]` and `#[cfg(not(debug_assertions))]` to
/// determine which to use.
#[proc_macro]
pub fn release_env(ts: TokenStream) -> TokenStream {
    #[cfg(not(debug_assertions))]
    let result = release_env_release(ts);
    #[cfg(debug_assertions)]
    let result = release_env_debug(ts);

    match result {
        Ok(output) => TokenTree::Literal(output).into(),
        Err(error) => {
            error.emit();
            return TokenTree::Literal(Literal::string("")).into();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmptyString {
    Ok,
    Error,
}

fn pull_string_literal(
    tree: Option<TokenTree>,
    empty: EmptyString,
) -> Result<(String, Literal), Diagnostic> {
    match tree {
        Some(TokenTree::Literal(lit)) => {
            let string = enforce_string_literal(&lit, empty)?;
            Ok((string, lit))
        }
        Some(other) => Err(err_diag!(other.span(), "expected a string literal")),
        None => Err(err_diag!("must supply an environment variable name")),
    }
}

#[cfg(not(debug_assertions))]
fn release_env_release(ts: TokenStream) -> Result<Literal, Diagnostic> {
    let mut ts_iter = ts.into_iter();

    let (env_str, env_lit) = pull_string_literal(ts_iter.next(), EmptyString::Error)?;

    get_env_var(&env_str, &env_lit).map(|opt_var_lit| {
        opt_var_lit.ok_or_else(|| err_diag!(env_lit.span(), "{} is not set", env_str))
    })?
}

fn release_env_debug(ts: TokenStream) -> Result<Literal, Diagnostic> {
    let mut ts_iter = ts.into_iter();

    let (env_str, env_lit) = pull_string_literal(ts_iter.next(), EmptyString::Error)?;

    if let Some(lit) = get_env_var(&env_str, &env_lit)? {
        return Ok(lit);
    }

    let next = ts_iter.next();

    if next.is_none() {
        emit_warn_diag!(
            "{} not found, using the default fallback value '{}'",
            env_str,
            DEFAULT_FALLBACK_VALUE,
        );

        return Ok(Literal::string(DEFAULT_FALLBACK_VALUE));
    }

    pull_punct(next, ',', env_lit.span())?;

    let ident = pull_ident(ts_iter.next(), env_lit.span())?;
    let ident_str = ident.to_string();

    let (is_env, next_lit_empty) = match ident_str.as_str() {
        "value" => (false, EmptyString::Ok),
        "env" => (true, EmptyString::Error),
        other => {
            return Err(err_diag!(
                ident.span(),
                "expected 'value' or 'env', not '{}'",
                other
            ));
        }
    };

    pull_punct(ts_iter.next(), ':', ident.span())?;

    let (fallback_str, fallback_lit) = pull_string_literal(ts_iter.next(), next_lit_empty)?;

    if !is_env {
        emit_warn_diag!(
            fallback_lit.span(),
            "{} not found, using this fallback value",
            env_str
        );

        return Ok(fallback_lit);
    }

    match get_env_var(&fallback_str, &fallback_lit)? {
        Some(lit) => {
            emit_warn_diag!(
                "{} not found, using fallback env var {}",
                env_str,
                fallback_str
            );

            Ok(lit)
        }
        None => Err(err_diag!(
            fallback_lit.span(),
            "both {} and {} env vars missing",
            env_str,
            fallback_str,
        )),
    }
}

fn pull_punct(
    tree: Option<TokenTree>,
    expected: char,
    fallback_span: proc_macro::Span,
) -> Result<proc_macro::Punct, Diagnostic> {
    match tree {
        Some(TokenTree::Punct(punct)) => match punct.as_char() {
            ch if ch == expected => Ok(punct),
            ch => Err(err_diag!(
                "unexpected punct, found '{}', expected: '{}'",
                ch,
                expected
            )),
        },
        Some(other) => Err(err_diag!(
            other.span(),
            "expected punct '{}', found '{}' instead",
            expected,
            other.to_string(),
        )),
        None => Err(err_diag!(
            fallback_span,
            "unexpected end of token stream, expected punct '{}'",
            expected,
        )),
    }
}

fn pull_ident(
    tree: Option<TokenTree>,
    fallback_span: proc_macro::Span,
) -> Result<proc_macro::Ident, Diagnostic> {
    match tree {
        Some(TokenTree::Ident(ident)) => Ok(ident),
        Some(other) => Err(err_diag!(other.span(), "expected an ident")),
        None => Err(err_diag!(fallback_span, "expected an ident")),
    }
}

fn get_env_var(key: &str, lit: &Literal) -> Result<Option<Literal>, Diagnostic> {
    match tracked_env::var(key) {
        Ok(value) => Ok(Some(Literal::string(value.as_str()))),
        Err(std::env::VarError::NotUnicode(_)) => Err(err_diag!(
            lit.span(),
            "{} exists, but is not valid UTF-8",
            key
        )),
        Err(std::env::VarError::NotPresent) => Ok(None),
    }
}

fn enforce_string_literal(lit: &Literal, empty: EmptyString) -> Result<String, Diagnostic> {
    let lit_str = lit.to_string();

    if !lit_str.starts_with('"') || !lit_str.ends_with('"') {
        return Err(err_diag!(lit.span(), "expeceted a string literal"));
    }

    if empty == EmptyString::Error && lit_str.len() <= 2 {
        return Err(err_diag!(lit.span(), "cannot be an empty string"));
    }

    Ok(lit_str)
}
