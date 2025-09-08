macro_rules! ts {
    ($fmt: literal$(, $i: expr)*) => {{
        use std::str::FromStr;
        let msg = format!($fmt$(, $i)*);
        TokenStream::from_str(msg.as_str()).unwrap()
    }};
}

macro_rules! error {
    (raw, $span: expr, $fmt: literal$(, $i: expr)*) => {{
        let msg = format!($fmt$(, $i)*);
        ts!("compile_error!(\"{}\\n --> {}:{}:{}\");", msg, $span.file().replace("\\", r"\\"), $span.line(), $span.column())
    }};
    (arg, $span: expr, $fmt: literal$(, $i: expr)*) => {{
        let s = error!(raw, $span, $fmt $(, $i)*);
        let s = s.to_string();
        ts!("#[deny = {}] _: ()", &s[..s.len() -1])
    }};
    (block, $span: expr, $fmt: literal$(, $i: expr)*) => {{
        let s = error!(raw, $span, $fmt $(, $i)*);
        ts!("{{ {} }}", s)
    }};
    (raw, $fmt: literal$(, $i: expr)*) => {{
        let msg = format!($fmt$(, $i)*);
        ts!("compile_error!(\"{}\");", msg)
    }};
    (arg, $fmt: literal$(, $i: expr)*) => {{
        let s = error!(raw, $fmt $(, $i)*);
        let s = s.to_string();
        ts!("#[deny = {}] _: ()", &s[..s.len() -1])
    }};
    (block, $fmt: literal$(, $i: expr)*) => {{
        let s = error!(raw, $fmt $(, $i)*);
        ts!("{{ {} }}", s)
    }};
}
