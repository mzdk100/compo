mod arguments;
mod block;
#[macro_use]
mod macros;
mod utils;

use {
    arguments::handle_arguments,
    block::handle_block,
    proc_macro::{TokenStream, TokenTree},
    utils::to_camel,
};

#[proc_macro_attribute]
pub fn component(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let mut iter = item.into_iter();
    let mut ident = iter.next();
    let vis = if let Some(TokenTree::Ident(i)) = &ident
        && i.to_string() == "pub"
    {
        ident = iter.next();
        "pub"
    } else {
        ""
    };
    match &ident {
        Some(TokenTree::Ident(i)) if i.to_string() != "async" => {
            return error!(raw, i.span(), "Expected keyword `async`, got `{}`", i);
        }
        None => return error!(raw, "Expected keyword `async`, got eof"),
        _ => (),
    }
    if let Some(TokenTree::Ident(i)) = iter.next()
        && i.to_string() != "fn"
    {
        return error!(raw, i.span(), "Expected keyword `fn`, got `{}`", i);
    }
    let component_name = if let Some(TokenTree::Ident(i)) = iter.next() {
        i.to_string()
    } else {
        return error!(raw, "Expected ident (component name), got eof");
    };
    let Some(TokenTree::Group(g)) = iter.next() else {
        return error!(raw, "Expected function arguments (component properties)");
    };
    let (
        component_arguments,
        mut component_var_bindings,
        property_field_defines,
        property_field_initializers,
        property_field_getters_and_setters,
    ) = handle_arguments(g.stream(), &component_name);
    let Some(TokenTree::Group(g)) = iter.next() else {
        return error!(raw, "Expected function block");
    };
    let (component_block, mut field_defines, mut field_initializers) =
        handle_block(g.stream(), &mut component_var_bindings, &component_name);
    let component_name_camel = to_camel(&component_name);
    field_defines.extend_from_slice(&property_field_defines);
    let field_defines = field_defines
        .iter()
        .map(|i| i.to_string())
        .collect::<String>();
    let component_struct = ts!(
        "{}\n{} struct {} <'a> {{\n_rt: Weak<Runtime<'a, ()>>,\n_cancellable: RefCell<Cancellable>,\n{}\n}}",
        attrs,
        vis,
        component_name_camel,
        field_defines
    );
    field_initializers.extend_from_slice(&property_field_initializers);
    let field_initializers = field_initializers
        .iter()
        .map(|i| i.to_string())
        .collect::<String>();

    let component_new = ts!(
        "fn new(rt: Weak<Runtime<'a, ()>>) -> Self {{ Self {{\n{}\n_rt: rt,\n_cancellable: Default::default(),\n}} }}",
        field_initializers
    );
    let component_get_rt = ts!("fn get_rt(&self) -> Weak<Runtime<'a, ()>> {{ self._rt.clone() }}");
    let component_update = ts!(
        "fn update(self: &Rc<Self>) {{\nlet mut cancellable = self._cancellable.borrow_mut();\ncancellable.cancel();\n*cancellable = self.spawn({}(Rc::downgrade(self)));\n}}",
        component_name
    );

    let component_field_getters_and_setters = property_field_getters_and_setters
        .iter()
        .map(|i| i.to_string())
        .collect::<String>();
    let component_impl = ts!(
        "{}\nimpl<'a> Component<'a> for {} <'a> {{\n{}\n{}\n{}\n}}\n{}\nimpl<'a> {} <'a> {{\n{}\n}}",
        attrs,
        component_name_camel,
        component_new,
        component_get_rt,
        component_update,
        attrs,
        component_name_camel,
        component_field_getters_and_setters
    );
    ts!(
        "{}\n{}\n{}\n{} async fn {}({}) {}",
        component_struct,
        component_impl,
        attrs,
        vis,
        component_name,
        component_arguments,
        component_block
    )
}
