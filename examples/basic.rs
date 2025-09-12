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

    let tick_listener = Default::default(); // Create an event listener
    #[render]
    countdown {
        on_tick: tick_listener, // If a child component emits an event, the listener can receive it
    };
    while let Some(i) = tick_listener.listen().await.as_ref() {
        // Receive events from the countdown component
        println!("{} seconds.", i);
    }

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

#[component]
async fn countdown(#[event] on_tick: Option<u32>) {
    // Emit events for the parent component to receive
    for i in (0..10).rev() {
        let _ = on_tick.emit(Some(i));
        sleep(Duration::from_secs(1)).await;
    }
    let _ = on_tick.emit(None);
}
