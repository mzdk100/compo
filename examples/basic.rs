use compo::prelude::*;

fn main() {
    run(app);
}

#[component]
async fn app() {
    #[render] // Rendering is deferred to the next polling
    row {};
    #[render] // Component parameters can be omitted (using default values)
    button {};
    println!("Hello, app!");
}

#[component]
async fn row() {
    let mut text = "Hello";
    #[render] // Renders a component, re-renders child component if dependent variables change
    button {
        text: text,
        width: 32,
    };
    text = "world"; // Will re-render the button component
}

#[component]
async fn button(#[default = "hello"] text: &str, #[doc = "width"] width: u32) {
    #[field]
    // Fields are automatically added to internal struct (not exposed), values can be changed but won't trigger re-render of dependent child components
    let id: i32 = 0;
    println!("{}, {}", text, id);
    *id = 1; // Field lifetime is same as run function, so value can be reused across multiple renders
}
