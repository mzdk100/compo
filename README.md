# Compo - Declarative and Reactive Component Framework

[中文文档](README-zh-CN.md)

---

Compo is a general-purpose declarative and reactive component framework designed for single-threaded asynchronous
execution, offering high performance and safety guarantees.
It does not include any pre-implemented components but provides a `#[component]` macro and essential type exports,
completely independent of third-party libraries.
Suitable for GUI scenarios or other similar non-GUI component systems.

## Features

- **Concise Syntax**: Ergonomic design, eliminates 99% of unnecessary boilerplate code, making it simple to use.
- **Declarative Components**: Easily define components using the `#[component]` macro.
- **Reactive Rendering**: Automatically re-renders child components when dependent variables change.
- **Minimal Trait Dependencies**: No `Send`/`Sync` or `'static` constraints, and no cross-thread synchronization
  mechanisms.
- **Single-Threaded Async**: Efficiently runs in a single-threaded environment, ideal for high-performance scenarios.
- **No Third-Party Dependencies**: Fully standalone, with no external libraries.
- **Safety Guarantees**: Strict type checking and runtime safety mechanisms.

## Quick Start

### Installation

Add Compo to your `Cargo.toml`:

```shell
cargo add compo
```

### Example Code

Here's a simple example demonstrating how to define and use components:

```rust
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
```

### Running the Example

When you run this example with `cargo run --example basic`, you'll see the following output:

```
hello, 0
9 seconds.
world, 0
world, 1
8 seconds.
7 seconds.
6 seconds.
5 seconds.
4 seconds.
3 seconds.
2 seconds.
1 seconds.
0 seconds.
Hello, app!
```

This output demonstrates:

1. The button component first renders with default text "hello" and id=0
2. The countdown component starts emitting events with countdown values from 9 to 0
3. The button component re-renders with text "world" and id=0, then again with id=1
4. The app component continues receiving countdown events
5. After the countdown completes, the app component prints "Hello, app!"

For more advanced usage of the compo library, please refer to the examples in the [examples](examples) directory.

## API Documentation

### `#[component]` Macro

Used to define components. Components must be asynchronous functions and support rendering child components and reactive
updates.

### `#[render]` Attribute

Marks child components for rendering. If dependent variables change, the child component will re-render.

### `#[field]` Attribute

Defines internal fields for components, with lifetimes matching the `run` function.

### `#[event]` Attribute

Marks a component parameter as an event emitter. This allows child components to send events to their parent components. The parameter should be of type `Option<T>`, where `T` is the type of data to be emitted. Events can be emitted using the `.emit()` method and received by the parent component using the `.listen().await` method.

## Contributing

Issues and Pull Requests are welcome!

## License

Apache-2.0
