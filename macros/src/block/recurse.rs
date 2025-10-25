use {
    super::stmt::handle_stmt,
    crate::utils::{take_while, to_camel},
    proc_macro::{Delimiter, Punct, Spacing, TokenStream, TokenTree},
    std::collections::HashMap,
};

pub(super) fn handle_block_recursively(
    stream: &TokenStream,
    has_attr: &mut bool,
    attrs: &mut Vec<TokenStream>,
    var_bindings: &mut Vec<TokenStream>,
    field_defines: &mut Vec<TokenStream>,
    field_initializers: &mut Vec<TokenStream>,
    component_name_index: &mut u32,
    stmts: &mut Vec<TokenStream>,
    refer_to_component: &mut HashMap<String, HashMap<String, Vec<String>>>,
) -> TokenStream {
    let mut iter = stream.clone().into_iter();
    let mut stmt = Vec::new();

    while let Some(tree) = iter.next() {
        match tree {
            TokenTree::Group(g) if *has_attr && g.delimiter() == Delimiter::Bracket => {
                attrs.push(g.stream());
                *has_attr = false;
            }
            TokenTree::Punct(p) if !*has_attr && p.as_char() == '#' => *has_attr = true,
            t if *has_attr => {
                return error!(raw, t.span(), "Expected attribute, got {}", t);
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
                        return error!(
                            block,
                            i.span(),
                            "Field attribute must be used with `let` var."
                        );
                    }
                    let Some(TokenTree::Ident(ident)) = iter.next() else {
                        return error!(block, "Expected ident (field name)");
                    };
                    let field_name = ident.to_string();
                    if field_name == "mut" {
                        return error!(
                            block,
                            ident.span(),
                            "Expected ident (field name), got keyword `mut`"
                        );
                    }

                    match iter.next() {
                        Some(TokenTree::Punct(p)) if p.as_char() == ':' => (),
                        Some(t) => {
                            return error!(
                                block,
                                t.span(),
                                "Expected ':', got `{}` (field `{}` must specify a data type)",
                                t,
                                field_name
                            );
                        }
                        None => {
                            return error!(block, "Expected ':', got eof");
                        }
                    }

                    let field_type = match take_while(
                        &mut iter,
                        &TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                    ) {
                        Err(s) => {
                            return error!(
                                block,
                                s,
                                "Expected '=', got eof (field `{}` must be initialized with a value)",
                                field_name
                            );
                        }
                        Ok(f) if f.is_empty() => {
                            return error!(block, t.span(), "Expected type (field type), got '='");
                        }
                        Ok(f) => f,
                    };

                    let field_value = match take_while(
                        &mut iter,
                        &TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                    ) {
                        Err(s) => return error!(block, s, "Expected ';', got eof"),
                        Ok(f) if f.is_empty() => {
                            return error!(block, t.span(), "Expected expr (field value), got ';'");
                        }
                        Ok(f) => f,
                    };

                    let attrs = attrs
                        .iter()
                        .map(|i| format!("#[{}]", i))
                        .collect::<String>();
                    var_bindings.push(ts!(
                        "{} let {}: &mut {} = unsafe {{ transmute(this.{}.get()) }};",
                        attrs,
                        field_name,
                        field_type,
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
                        return error!(block, t.span(), "Expected ident (component name)");
                    };
                    let component_name = ident.to_string();
                    let Some(TokenTree::Group(g)) = iter.next() else {
                        return error!(block, t.span(), "Expected block (component properties)");
                    };
                    match iter.next() {
                        Some(TokenTree::Punct(p)) if p.as_char() == ';' => (),
                        _ => {
                            return error!(block, g.span(), "Expected semicolon ';'");
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
                        let property_value = match iter.next() {
                            Some(TokenTree::Punct(p)) if p.as_char() == ',' => {
                                property_name.clone()
                            }
                            Some(TokenTree::Punct(p)) if p.as_char() == ':' => {
                                let iter2 = iter.clone();
                                match take_while(
                                    &mut iter,
                                    &TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                                ) {
                                    Ok(f) => f.to_string(),
                                    Err(_) => iter2.map(|i| i.to_string()).collect(),
                                }
                            }
                            Some(t) => {
                                return error!(
                                    block,
                                    t.span(),
                                    "Expected comma ',' or colon ':', got {}",
                                    t
                                );
                            }
                            None => {
                                return error!(
                                    block,
                                    t.span(),
                                    "Expected comma ',' or colon ':', got eof"
                                );
                            }
                        };
                        stmts.push(ts!(
                            "this.{}.set_{}(&{});",
                            component_id,
                            property_name,
                            property_value
                        ));
                        if !refer_to_component.contains_key(&property_value) {
                            refer_to_component.insert(property_value.clone(), Default::default());
                        }
                        if let Some(components) = refer_to_component.get_mut(&property_value) {
                            if !components.contains_key(&component_name) {
                                components.insert(component_id.clone(), Default::default());
                            }
                            if let Some(properties) = components.get_mut(&component_id) {
                                properties.push(property_name);
                            }
                        }
                    }
                    stmts.push(ts!(
                        "this.spawn({}(Rc::downgrade(&this.{})));",
                        component_name,
                        component_id
                    ));
                    *component_name_index += 1;
                }

                attrs.clear();
            }
            t => {
                stmt.push(t.clone());
                if let TokenTree::Punct(p) = &t
                    && p.as_char() == ';'
                {
                    handle_stmt(stmts, &mut stmt, refer_to_component);
                } else if let TokenTree::Group(g) = &t
                    && g.delimiter() == Delimiter::Brace
                {
                    stmt.pop();
                    stmts.push(TokenStream::from_iter(stmt.clone().into_iter()));
                    stmt.clear();
                    let mut stmts2 = Vec::new();
                    let error = handle_block_recursively(
                        &g.stream(),
                        has_attr,
                        attrs,
                        var_bindings,
                        field_defines,
                        field_initializers,
                        component_name_index,
                        &mut stmts2,
                        refer_to_component,
                    );
                    let stmts2 = stmts2.iter().map(|i| format!("{} ", i)).collect::<String>();
                    stmts.push(ts!("{{\n{}\n}}", stmts2));
                    if !error.is_empty() {
                        return error;
                    }
                }
            }
        }
    }
    if stmt.iter().last().is_some() {
        handle_stmt(stmts, &mut stmt, refer_to_component);
    }

    Default::default()
}
