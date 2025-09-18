use {
    proc_macro::{TokenStream, TokenTree},
    std::collections::HashMap,
};

pub(super) fn handle_stmt(
    stmts: &mut Vec<TokenStream>,
    stmt: &mut Vec<TokenTree>,
    refer_to_component: &mut HashMap<String, HashMap<String, (String, Vec<String>)>>,
) {
    stmts.push(TokenStream::from_iter(stmt.clone()));
    let mut iter = stmt.iter();
    while let Some(i) = iter.next() {
        if let TokenTree::Ident(ident) = i
            && let Some(c) = refer_to_component.get(&ident.to_string())
            && let Some(TokenTree::Punct(p)) = iter.next()
            && p.as_char() == '='
        {
            for (component_name, (component_id, properties)) in c.iter() {
                for property_name in properties.iter() {
                    stmts.push(ts!("this.{}.set_{}(&{});", component_id, property_name, i));
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
