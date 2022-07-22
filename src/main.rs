#![allow(dead_code)]
use windows::{core::*, Win32::Foundation::*, Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::UI::WindowsAndMessaging::*, Win32::Graphics::Gdi::*, Win32::System::Com::*, Win32::System::Console::GetConsoleWindow};
use std::{thread, time};
use std::sync::Mutex;
use std::sync::mpsc::*;


// Global
static mut CLICKED: bool = false;
static mut RESULT_SENDER: Option<Mutex<Sender<bool>>> = None;
static WHITE: u32 = 0x00FFFFFF as u32;
static BLACK: u32 = 0x00000000 as u32;
fn main() -> Result<()> {
    let (tx, rx) = channel();
    
    // hide console
    unsafe {
        let window = GetConsoleWindow();
        ShowWindow(window, SW_HIDE);
        CoInitializeEx(std::ptr::null(), COINIT_MULTITHREADED)?;
        RESULT_SENDER = Some(Mutex::new(tx));
    }
    
    let mut window = Window::new()?;

    //receiver waiting for click event
    thread::spawn(move || {
            loop {
            let x = rx.recv().expect("can't receive from channel");
            if x{window.call_update();}
            }
        });
    window.run()
}


#[derive(Copy, Clone)]

struct Window {
    handle: HWND,
}

impl Window {
    fn new() -> Result<Self>{
        Ok(Window{
            handle: HWND(0),
        })
    }
    
    fn call_update(&self){
        let window = self.handle;
        let mut rect = RECT::default();
        unsafe{
            CLICKED = true;
            GetClientRect(window, &mut rect);
            InvalidateRect(window, &mut rect, false);

            thread::sleep(time::Duration::from_millis(200));
            CLICKED = false;
            InvalidateRect(window, &mut rect, false);
        }
    }

    fn run(&mut self) -> Result<()> {
        unsafe {
            let instance = GetModuleHandleA(None)?;
            debug_assert!(instance.0 != 0);
    
            let window_class = "window";

            let wc = WNDCLASSA {
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                hInstance: instance,
                lpszClassName: PCSTR(b"window\0".as_ptr()),
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                ..Default::default()
            };

            let window_width = 100;
            let window_height = 100;

            let atom = RegisterClassA(&wc);
            debug_assert!(atom != 0);

            CreateWindowExA(WS_EX_TOPMOST, window_class, "Clicker", WS_OVERLAPPEDWINDOW | WS_VISIBLE, CW_USEDEFAULT, CW_USEDEFAULT, window_width, window_height, None, None, instance, std::ptr::null());

            let mut message = MSG::default();

            while GetMessageA(&mut message, HWND(0), 0, 0).into() {
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }

            Ok(())
            }
        }
        extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
            let mut ps = PAINTSTRUCT::default();
            let mut rect = RECT::default();
            
            match message as u32 {
                WM_PAINT => {
                    println!("WM_PAINT called");
                    unsafe{
                        let hdc = BeginPaint(window, &mut ps);
                        if CLICKED{
                            let hbrush = CreateSolidBrush(WHITE);
                            let htmp = SelectObject(hdc, hbrush);
                            GetClientRect(window, &mut rect);
                            FillRect(hdc, &rect, hbrush);
                            DeleteObject(SelectObject(hdc, htmp));
                            EndPaint(window, &ps);
                        }

                        else{
                            let hbrush = CreateSolidBrush(BLACK);
                            let htmp = SelectObject(hdc, hbrush);
                            GetClientRect(window, &mut rect);
                            FillRect(hdc, &rect, hbrush);
                            DeleteObject(SelectObject(hdc, htmp));
                            EndPaint(window, &ps);
                        }
                    }
                    LRESULT(0)
                }
                
                WM_DESTROY => {
                    println!("WM_DESTROY called");
                    unsafe { PostQuitMessage(0); }
                    LRESULT(0)
                    }
                    
                WM_LBUTTONUP=>{
                    println!("WM_LBUTTONUP");
                    unsafe{
                        let tx = RESULT_SENDER.as_ref().unwrap().lock().unwrap().clone();
                        tx.send(true).expect("Cannot send");
                    }

                    LRESULT(0)
                }

                WM_CREATE =>{
                    println!("WM_CREATE called");
                    LRESULT(0)
                }

                _ => unsafe{DefWindowProcA(window, message, wparam, lparam)},
                }
            }
        }
