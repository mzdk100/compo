# Compo - 声明式与响应式组件框架

[English Documentation](README.md)

---

Compo 是一个通用的声明式和响应式的组件框架，专为单线程异步运行设计，具有极快的速度和安全保证。
它不包含任何预实现的组件，仅提供一个 `#[component]` 宏和一些必要的类型导出，且完全独立于第三方库。
可用于GUI场景或者其他类似的非GUI组件系统

## 特性

- **语法简洁**：人体工程学的设计，消除了99%不必要的样板代码，使用简单。
- **声明式组件**：使用 `#[component]` 宏轻松定义组件。
- **响应式渲染**：依赖变量变化时自动重新渲染子组件。
- **最小化trait依赖**：没有Send/Sync和'static的限制，且不包含任何跨线程的同步机制。
- **单线程异步**：高效运行在单线程环境中，适合高性能场景。
- **无第三方依赖**：完全独立，不引入任何外部库。
- **安全保证**：严格的类型检查和运行时安全机制。

## 快速开始

### 安装

将compo添加到你的 `Cargo.toml` 文件中：

```shell
cargo add compo
```

### 示例代码

以下是一个简单的示例，展示如何定义和使用组件：

```rust
use compo::prelude::*;

fn main() {
    run(app);
}

#[component]
async fn app() {
    #[render] // 渲染被推迟到下一次轮询
    row {};
    #[render] // 组件参数可以省略（使用默认值）
    button {};
    
    let tick_listener = Default::default(); // 创建一个事件监听器
    #[render]
    countdown {
        on_tick: tick_listener, // 如果子组件发出了事件，监听器可以接收
    };
    while let Some(i) = tick_listener.listen().await.as_ref() {
        // 接收countdown组件的事件
        println!("{} seconds.", i);
    }
    
    println!("Hello, app!");
}

#[component]
async fn row() {
    let mut text = "Hello";
    #[render] // 渲染一个组件，如果依赖变量发生变化，则重新渲染子组件
    button {
        text: text,
        width: 32,
    };
    text = "world"; // 将重新渲染button组件
}

#[component]
async fn button(#[default = "hello"] text: &str, #[doc = "width"] width: u32) {
    #[field]
    // 字段会自动添加到内部结构体（不暴露），值可以更改但不会触发依赖子组件的重新渲染
    let id: i32 = 0;
    println!("{}, {}", text, id);
    *id = 1; // 字段的生命周期与run函数相同，因此值可以在多次渲染中重复使用
}

#[component]
async fn countdown(#[event] on_tick: Option<u32>) {
    // 发出事件让父组件接收
    for i in (0..10).rev() {
        let _ = on_tick.emit(Some(i));
        sleep(Duration::from_secs(1)).await;
    }
    let _ = on_tick.emit(None);
}
```

### 运行示例

当你使用 `cargo run --example basic` 运行这个示例时，你将看到以下输出：

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

这个输出演示了：

1. button组件首次渲染时使用默认文本 "hello" 和 id=0
2. countdown组件开始发出从9到0的倒计时事件
3. button组件重新渲染，先使用文本 "world" 和 id=0，然后再次渲染使用 id=1
4. app组件继续接收倒计时事件
5. 倒计时完成后，app组件打印 "Hello, app!"

有关compo库更高级的用法，请参见[examples](examples)目录中的示例。

## API 文档

### `#[component]` 宏

用于定义组件。组件必须是异步函数，支持渲染子组件和响应式更新。

### `#[render]` 属性

标记需要渲染的子组件。如果依赖的变量发生变化，子组件会重新渲染。

### `#[field]` 属性

定义组件的内部字段，其生存期与 `run` 函数相同。

### `#[event]` 属性

将组件参数标记为事件发射器。这允许子组件向父组件发送事件。参数应为 `Option<T>` 类型，其中 T 是要发送的数据类型。事件可以通过 `.emit()` 方法发送，父组件可以使用 `.listen().await` 方法接收事件。

## 贡献

欢迎提交 Issue 或 Pull Request！

## 许可证

Apache-2.0