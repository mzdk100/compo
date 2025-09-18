use proc_macro::{Span, TokenStream, TokenTree, token_stream::IntoIter};

pub(super) fn to_camel(text: &str) -> String {
    let mut result = String::new();
    let mut capitalize = false;
    let mut chars = text.chars();
    if let Some(c) = chars.next() {
        result.push(c.to_ascii_uppercase());
    }
    for c in chars {
        if c == '_' || c == '-' || c == ' ' {
            capitalize = true;
        } else if capitalize {
            result.push(c.to_ascii_uppercase());
            capitalize = false;
        } else {
            result.push(c);
        }
    }

    result
}

pub fn take_while(iter: &mut IntoIter, end_token: &TokenTree) -> Result<TokenStream, Span> {
    let mut tokens = Vec::new();
    let mut found_end = false;
    let mut span = Span::call_site();

    while let Some(token) = iter.next() {
        span = token.span();
        if token.to_string() == end_token.to_string() {
            found_end = true;
            break;
        }
        tokens.push(token);
    }

    if !found_end {
        return Err(span);
    }

    Ok(TokenStream::from_iter(tokens.into_iter()))
}
