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
