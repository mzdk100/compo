#[path = "macros.rs"]
#[macro_use]
mod macros;

use {
    crate::utils::to_camel,
    proc_macro::{Delimiter, TokenStream, TokenTree},
};

pub(super) fn handle_arguments(
    arguments: TokenStream,
    component_name: &str,
) -> (
    TokenStream,
    Vec<TokenStream>,
    Vec<TokenStream>,
    Vec<TokenStream>,
    Vec<TokenStream>,
) {
    let mut iter = arguments.into_iter();
    let mut attrs = Vec::new();
    let mut has_attr = false;
    let mut var_bindings = Vec::new();
    let mut field_defines = Vec::new();
    let mut field_initializers = Vec::new();
    let mut field_getters_and_setters = Vec::new();

    while let Some(tree) = iter.next() {
        match tree {
            TokenTree::Group(g) if has_attr && g.delimiter() == Delimiter::Bracket => {
                attrs.push(g.stream());
                has_attr = false;
            }
            TokenTree::Punct(p) if !has_attr && p.as_char() == '#' => has_attr = true,
            t if has_attr => {
                return (
                    error!(arg, t.span(), "Expected attribute, got {}", t),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                );
            }
            t => {
                let TokenTree::Ident(ident) = t else {
                    return (
                        error!(arg, "Expected ident (property name)"),
                        Default::default(),
                        Default::default(),
                        Default::default(),
                        Default::default(),
                    );
                };
                let property_name = ident.to_string();
                match iter.next() {
                    Some(TokenTree::Punct(p)) if p.as_char() == ':' => (),
                    Some(t) => {
                        return (
                            error!(
                                arg,
                                t.span(),
                                "Expected ':', got `{}` (property `{}` must specify a data type)",
                                t,
                                property_name
                            ),
                            Default::default(),
                            Default::default(),
                            Default::default(),
                            Default::default(),
                        );
                    }
                    None => {
                        return (
                            error!(arg, "Expected ':', got eof"),
                            Default::default(),
                            Default::default(),
                            Default::default(),
                            Default::default(),
                        );
                    }
                }
                let mut argument_type = Vec::new();
                while let Some(i) = iter.next() {
                    if let TokenTree::Punct(p) = &i
                        && p.as_char() == ','
                    {
                        break;
                    }

                    argument_type.push(i.clone());
                }
                if argument_type.is_empty() {
                    return (
                        error!(arg, "Expected type (property type)"),
                        Default::default(),
                        Default::default(),
                        Default::default(),
                        Default::default(),
                    );
                }
                let property_type = argument_type
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<String>();
                let mut default_value = Vec::new();
                let mut is_event = false;
                let attrs_str = attrs
                    .iter()
                    .filter(|i| {
                        let mut iter = i.to_owned().to_owned().into_iter();
                        let item = iter.next();
                        if let Some(TokenTree::Ident(i)) = &item
                            && i.to_string() == "default"
                            && let Some(TokenTree::Punct(p)) = iter.next()
                            && p.as_char() == '='
                        {
                            while let Some(i) = iter.next() {
                                default_value.push(i);
                            }
                            false
                        } else if let Some(TokenTree::Ident(i)) = &item
                            && i.to_string() == "event"
                        {
                            is_event = true;
                            false
                        } else {
                            true
                        }
                    })
                    .map(|i| format!("#[{}]", i))
                    .collect::<String>();
                attrs.clear();

                var_bindings.push(ts!(
                    "{} let {} = this.get_{}();",
                    attrs_str,
                    property_name,
                    property_name
                ));
                field_defines.push(if is_event {
                    ts!(
                        "{} {}: UnsafeCell<EventEmitter<'a, {}>>,",
                        attrs_str,
                        property_name,
                        property_type.replace("&", "&'a ")
                    )
                } else {
                    ts!(
                        "{} {}: UnsafeCell<{}>,",
                        attrs_str,
                        property_name,
                        property_type.replace("&", "&'a ")
                    )
                });
                if is_event {
                    field_initializers.push(ts!(
                        "{} {}: EventEmitter::default().into(),",
                        attrs_str,
                        property_name
                    ));
                    field_getters_and_setters.push(ts!(
                        "{} pub fn get_{}(&self) -> &EventEmitter<'a, {}> {{\nunsafe {{ transmute(self.{}.get()) }}\n}}",
                        attrs_str,
                        property_name,
                        property_type.replace("&", "&'a "),
                        property_name
                    ));
                    field_getters_and_setters.push(ts!(
                        "{} pub fn set_{}(&self, value: &EventListener<'a, {}>) {{\nunsafe {{\n*self.{}.get() = value.new_emitter()\n}}\n}}",
                        attrs_str,
                        property_name,
                        property_type.replace("&", "&'a "),
                        property_name
                    ))
                } else {
                    let property_default_value = if default_value.is_empty() {
                        format!("<{}>::default()", property_type)
                    } else {
                        default_value.iter().map(|i| i.to_string()).collect()
                    };
                    field_initializers.push(ts!(
                        "{} {}: {}.into(),",
                        attrs_str,
                        property_name,
                        property_default_value
                    ));
                    field_getters_and_setters.push(ts!(
                        "{} pub fn get_{}(&self) -> &{} {{\nunsafe {{ transmute(self.{}.get()) }}\n}}",
                        attrs_str,
                        property_name,
                        property_type.replace("&", "&'a "),
                        property_name
                    ));
                    field_getters_and_setters.push(ts!(
                        "{} pub fn set_{}(&self, value: &{}) {{\nunsafe {{ *self.{}.get() = *value }}\n}}",
                        attrs_str,
                        property_name,
                        property_type.replace("&", "&'a "),
                        property_name
                    ))
                }
            }
        }
    }

    (
        ts!("this: Weak<{}<'_>>", to_camel(component_name)),
        var_bindings,
        field_defines,
        field_initializers,
        field_getters_and_setters,
    )
}
