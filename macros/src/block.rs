#[path = "macros.rs"]
#[macro_use]
mod macros;
mod recurse;

use {
    crate::utils::to_camel, proc_macro::TokenStream, recurse::handle_block_recursively,
    std::collections::HashMap,
};

pub(super) fn handle_block(
    stream: TokenStream,
    var_bindings: &mut Vec<TokenStream>,
    component_name: &str,
) -> (TokenStream, Vec<TokenStream>, Vec<TokenStream>) {
    let mut has_attr = false;
    let mut attrs = Vec::new();
    let mut field_defines = Vec::new();
    let mut field_initializers = Vec::new();
    let mut stmts = Vec::new();
    let mut component_name_index = 0;
    let mut refer_to_component = HashMap::new();

    let error = handle_block_recursively(
        &stream,
        &mut has_attr,
        &mut attrs,
        var_bindings,
        &mut field_defines,
        &mut field_initializers,
        &mut component_name_index,
        &mut stmts,
        &mut refer_to_component,
    );
    if !error.is_empty() {
        return (error, Default::default(), Default::default());
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
