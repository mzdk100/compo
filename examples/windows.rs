use {
    compo::prelude::*,
    std::time::Duration,
    windows::{
        Win32::{
            Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
            Graphics::Gdi::HBRUSH,
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::{
                CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW,
                DefWindowProcW, DestroyWindow, DispatchMessageW, GWLP_USERDATA, GetClientRect,
                HCURSOR, HICON, MSG, PM_REMOVE, PeekMessageW, PostQuitMessage, RegisterClassW,
                SW_SHOW, SWP_NOMOVE, SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos, SetWindowTextW,
                ShowWindow, TranslateMessage, WM_CREATE, WM_DESTROY, WM_QUIT, WNDCLASSW,
                WS_EX_LEFT, WS_OVERLAPPEDWINDOW,
            },
        },
        core::{PCWSTR, w},
    },
};

// Window procedure callback function
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let create_struct = lparam.0 as *const CREATESTRUCTW;
            let window_ptr = unsafe { (*create_struct).lpCreateParams };
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_ptr as isize) };
            LRESULT::default()
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT::default()
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

// Window component
#[component]
pub async fn window(
    #[default = "Window example"] title: &str,
    #[default = 800] width: i32,
    #[default = 600] height: i32,
    #[default = true] shown: bool,
) {
    #[field]
    // This is a field of the component's internal structure, not a variable in the current scope, so it can persist across multiple renders
    let hwnd: Option<HWND> = None;

    if *shown {
        if hwnd.is_none() {
            // Register window class
            let h_instance = match unsafe { GetModuleHandleW(PCWSTR::null()) } {
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
                Ok(h) => h.into(),
            };

            let class_name = w!("CompoWindow");
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: h_instance,
                hIcon: HICON::default(),
                hCursor: HCURSOR::default(),
                hbrBackground: HBRUSH::default(),
                lpszMenuName: PCWSTR::null(),
                lpszClassName: class_name,
            };

            unsafe {
                RegisterClassW(&wc);

                // Create window
                // Convert string to UTF-16 and ensure it ends with null
                let mut window_title: Vec<u16> = title.encode_utf16().collect();
                window_title.push(0); // Add null terminator
                let hwnd_value = match CreateWindowExW(
                    WS_EX_LEFT,
                    class_name,
                    PCWSTR(window_title.as_ptr()),
                    WS_OVERLAPPEDWINDOW,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    *width,
                    *height,
                    None,
                    None,
                    Some(h_instance),
                    None,
                ) {
                    Ok(h) => h,
                    Err(e) => {
                        eprintln!("{}", e);
                        return;
                    }
                };

                hwnd.replace(hwnd_value);
            }
        }

        // Show window and update parameters
        if let Some(hwnd) = hwnd {
            let _ = unsafe { ShowWindow(*hwnd, SW_SHOW) };

            // Update window title (supports reactive updates)
            let mut window_title: Vec<u16> = title.encode_utf16().collect();
            window_title.push(0); // Add null terminator
            let _ = unsafe { SetWindowTextW(*hwnd, PCWSTR(window_title.as_ptr())) };

            // Update window size (supports reactive updates)
            let _ = unsafe {
                SetWindowPos(
                    *hwnd,
                    None,
                    0,
                    0, // x, y (unchanged)
                    *width,
                    *height,
                    SWP_NOMOVE | SWP_NOZORDER, // Keep position unchanged, don't change Z-order
                )
            };

            // Get client area size
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            let _ = unsafe { GetClientRect(*hwnd, &mut rect) };

            eprintln!(
                "Window updated with client area: {}x{}",
                rect.right - rect.left,
                rect.bottom - rect.top
            );
        } else {
            eprintln!("Failed to create window");
        }
    } else if let Some(hwnd) = hwnd.take() {
        // If shown is false and window exists, destroy the window
        let _ = unsafe { DestroyWindow(hwnd) };
    }
}

#[component]
async fn app() {
    let mut title = "hello";
    let mut width = 800;
    let mut height = 600;

    #[render]
    window {
        title,
        width,
        height,
    };

    // Wait 1-second then update the title
    sleep(Duration::from_secs(1)).await;
    title = "你好";

    // Wait another second then update the window size
    sleep(Duration::from_secs(1)).await;
    width = 1000;
    height = 800;
}

fn handle_windows_message(r#loop: &Loop) {
    // Use PeekMessage instead of GetMessage because GetMessage blocks until a message is available
    unsafe {
        let mut msg = MSG::default();
        // Check if there are messages in the queue without blocking
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            // If it's a WM_QUIT message, exit the loop
            if msg.message == WM_QUIT {
                r#loop.quit();
                break;
            }

            // Translate virtual key messages
            let _ = TranslateMessage(&msg);
            // Dispatch message to window procedure
            DispatchMessageW(&msg);
        }
    }
}

fn main() {
    Loop::new()
        .register_poll_handler(handle_windows_message)
        .run(app)
}
