#[path = "macros.rs"]
#[macro_use]
mod macros;

use {
    crate::utils::to_camel,
    proc_macro::{Delimiter, TokenStream, TokenTree},
    std::collections::HashMap,
};

pub(super) fn handle_block(
    stream: TokenStream,
    var_bindings: &mut Vec<TokenStream>,
    component_name: &str,
) -> (TokenStream, Vec<TokenStream>, Vec<TokenStream>) {
    let mut iter = stream.into_iter();
    let mut has_attr = false;
    let mut attrs = Vec::new();
    let mut field_defines = Vec::new();
    let mut field_initializers = Vec::new();
    let mut stmts = Vec::new();
    let mut component_name_index = 0;
    let mut stmt = Vec::new();
    let mut refer_to_component = HashMap::<_, HashMap<_, (String, Vec<_>)>>::new();

    while let Some(tree) = iter.next() {
        match tree {
            TokenTree::Group(g) if has_attr && g.delimiter() == Delimiter::Bracket => {
                attrs.push(g.stream());
                has_attr = false;
            }
            TokenTree::Punct(p) if !has_attr && p.as_char() == '#' => has_attr = true,
            t if has_attr => {
                return (
                    error!(raw, t.span(), "Expected attribute, got {}", t),
                    Default::default(),
                    Default::default(),
                );
            }
            t if !attrs.is_empty()
                && attrs
                    .iter()
                    .find(|i| {
                        let s = i.to_string();
                        &s == "field" || &s == "render"
                    })
                    .is_some() =>
            {
                let mut is_field = false;
                let mut is_render = false;
                attrs.retain(|i| {
                    let s = i.to_string();
                    is_field |= &s == "field";
                    is_render |= &s == "render";
                    &s != "field" && &s != "render"
                });

                if is_field {
                    if let TokenTree::Ident(i) = &t
                        && i.to_string() != "let"
                    {
                        return (
                            error!(
                                block,
                                i.span(),
                                "Field attribute must be used with `let` var."
                            ),
                            Default::default(),
                            Default::default(),
                        );
                    }
                    let Some(TokenTree::Ident(ident)) = iter.next() else {
                        return (
                            error!(block, "Expected ident (field name)"),
                            Default::default(),
                            Default::default(),
                        );
                    };
                    let field_name = ident.to_string();
                    if field_name == "mut" {
                        return (
                            error!(
                                block,
                                ident.span(),
                                "Expected ident (field name), got keyword `mut`"
                            ),
                            Default::default(),
                            Default::default(),
                        );
                    }
                    match iter.next() {
                        Some(TokenTree::Punct(p)) if p.as_char() == ':' => (),
                        Some(t) => {
                            return (
                                error!(
                                    block,
                                    t.span(),
                                    "Expected ':', got `{}` (field `{}` must specify a data type)",
                                    t,
                                    field_name
                                ),
                                Default::default(),
                                Default::default(),
                            );
                        }
                        None => {
                            return (
                                error!(block, "Expected ':', got eof"),
                                Default::default(),
                                Default::default(),
                            );
                        }
                    }
                    let Some(token) = iter.next() else {
                        return (
                            error!(block, "Expected ident (field type)"),
                            Default::default(),
                            Default::default(),
                        );
                    };
                    let field_type = token.to_string();
                    match iter.next() {
                        Some(TokenTree::Punct(p)) if p.as_char() == '=' => (),
                        Some(t) => {
                            return (
                                error!(
                                    block,
                                    t.span(),
                                    "Expected '=', got `{}` (field `{}` must be initialized with a value)",
                                    t,
                                    field_name
                                ),
                                Default::default(),
                                Default::default(),
                            );
                        }
                        None => {
                            return (
                                error!(block, "Expected '=', got eof"),
                                Default::default(),
                                Default::default(),
                            );
                        }
                    }
                    let Some(field_value) = iter.next() else {
                        return (
                            error!(block, "Expected a value (field initializer)"),
                            Default::default(),
                            Default::default(),
                        );
                    };
                    match iter.next() {
                        Some(TokenTree::Punct(p)) if p.as_char() == ';' => (),
                        Some(t) => {
                            return (
                                error!(block, t.span(), "Expected ';', got `{}`", t),
                                Default::default(),
                                Default::default(),
                            );
                        }
                        None => {
                            return (
                                error!(block, "Expected ';', got eof"),
                                Default::default(),
                                Default::default(),
                            );
                        }
                    }

                    let attrs = attrs
                        .iter()
                        .map(|i| format!("#[{}]", i))
                        .collect::<String>();
                    var_bindings.push(ts!(
                        "{} let {}: &mut _ = unsafe {{ transmute(this.{}.get()) }};",
                        attrs,
                        field_name,
                        field_name
                    ));
                    field_defines.push(ts!(
                        "{} {}: UnsafeCell<{}>,",
                        attrs,
                        field_name,
                        field_type
                    ));
                    field_initializers.push(ts!(
                        "{} {}: {}.into(),",
                        attrs,
                        field_name,
                        field_value
                    ));
                }

                if is_render {
                    let TokenTree::Ident(ident) = &t else {
                        return (
                            error!(block, t.span(), "Expected ident (component name)"),
                            Default::default(),
                            Default::default(),
                        );
                    };
                    let component_name = ident.to_string();
                    let Some(TokenTree::Group(g)) = iter.next() else {
                        return (
                            error!(block, t.span(), "Expected block (component properties)"),
                            Default::default(),
                            Default::default(),
                        );
                    };
                    match iter.next() {
                        Some(TokenTree::Punct(p)) if p.as_char() == ';' => (),
                        _ => {
                            return (
                                error!(block, g.span(), "Expected semicolon ';'"),
                                Default::default(),
                                Default::default(),
                            );
                        }
                    };
                    let component_id = format!("_{}", component_name_index);
                    let component_name_camel = to_camel(&component_name);
                    let attrs = attrs
                        .iter()
                        .map(|i| format!("#[{}]", i))
                        .collect::<String>();
                    field_defines.push(ts!(
                        "{} {}: Rc<{}<'a>>,",
                        attrs,
                        component_id,
                        component_name_camel
                    ));
                    field_initializers.push(ts!(
                        "{} {}: {}::new(rt.clone()).into(),",
                        attrs,
                        component_id,
                        component_name_camel
                    ));

                    let mut iter = g.stream().into_iter();
                    while let Some(i) = iter.next() {
                        let property_name = i.to_string();
                        let mut property_value = property_name.clone();
                        match iter.next() {
                            Some(TokenTree::Punct(p)) if p.as_char() == ':' => {
                                let mut value = Vec::new();
                                while let Some(i) = iter.next() {
                                    if let TokenTree::Punct(p) = &i
                                        && p.as_char() == ','
                                    {
                                        break;
                                    }
                                    value.push(i);
                                }
                                property_value = value.iter().map(|i| i.to_string()).collect();
                            }
                            Some(t) => {
                                return (
                                    error!(
                                        block,
                                        t.span(),
                                        "Expected comma ',' or colon ':', got {}",
                                        t
                                    ),
                                    Default::default(),
                                    Default::default(),
                                );
                            }
                            None => (),
                        }
                        stmts.push(ts!(
                            "this.{}.set_{}({});",
                            component_id,
                            property_name,
                            property_value
                        ));
                        if !refer_to_component.contains_key(&property_value) {
                            refer_to_component.insert(property_value.clone(), Default::default());
                        }
                        if let Some(components) = refer_to_component.get_mut(&property_value) {
                            if !components.contains_key(&component_name) {
                                components.insert(
                                    component_name.clone(),
                                    (component_id.clone(), Default::default()),
                                );
                            }
                            if let Some((_, properties)) = components.get_mut(&component_name) {
                                properties.push(property_name);
                            }
                        }
                    }
                    stmts.push(ts!(
                        "this.spawn({}(Rc::downgrade(&this.{})));",
                        component_name,
                        component_id
                    ));
                    component_name_index += 1;
                }

                attrs.clear();
            }
            t => {
                stmt.push(t.clone());
                if let TokenTree::Punct(p) = t
                    && p.as_char() == ';'
                {
                    stmts.push(TokenStream::from_iter(stmt.clone()));
                    let mut iter = stmt.iter();
                    while let Some(i) = iter.next() {
                        if let TokenTree::Ident(ident) = i
                            && let Some(c) = refer_to_component.get(&ident.to_string())
                            && let Some(TokenTree::Punct(p)) = iter.next()
                            && p.as_char() == '='
                        {
                            for (component_name, (component_id, properties)) in c.iter() {
                                for proprty_name in properties.iter() {
                                    stmts.push(ts!(
                                        "this.{}.set_{}({});",
                                        component_id,
                                        proprty_name,
                                        i
                                    ));
                                }
                                stmts.push(ts!(
                                    "this.spawn({}(Rc::downgrade(&this.{})));",
                                    component_name,
                                    component_id
                                ));
                            }
                            break;
                        }
                    }
                    stmt.clear();
                }
            }
        }
    }
    let var_bindings = var_bindings
        .iter()
        .map(|i| i.to_string())
        .collect::<String>();
    let stmts = stmts.iter().map(|i| format!("{} ", i)).collect::<String>();

    (
        ts!(
            "{{\nlet Some(this) = this.upgrade() else {{\neprintln!(\"{} dropped.\");\nreturn;\n}};\n{}\n{}\n}}",
            to_camel(&component_name),
            var_bindings,
            stmts
        ),
        field_defines,
        field_initializers,
    )
}
