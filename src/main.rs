use windows::{core::*, Win32::Foundation::*, Win32::Graphics::Gdi::ValidateRect, Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::UI::WindowsAndMessaging::*, Win32::Graphics::Gdi::*, Win32::System::Com::*};
use std::{thread, time};

static mut clicked: bool = false;
struct Window {
    handle: HWND,
    clicked: bool,
}

impl Window {
    fn new() -> Result<Self>{
        Ok(Window{
            handle: HWND(0),
            clicked: false,
        })
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


            let atom = RegisterClassA(&wc);
            debug_assert!(atom != 0);

            CreateWindowExA(Default::default(), window_class, "Clicker", WS_OVERLAPPEDWINDOW | WS_VISIBLE, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, None, None, instance, std::ptr::null());

            let mut message = MSG::default();

            while GetMessageA(&mut message, HWND(0), 0, 0).into() {
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }

            Ok(())
            }
        }
        extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
            unsafe {
                let mut ps = PAINTSTRUCT::default();
                let mut rect = RECT::default();
                let mut hbrush = CreateSolidBrush( 0x00FFFFFF as u32);
                
                match message as u32 {
                    WM_PAINT => {
                        println!("WM_PAINT");

                        if clicked {
                            let hdc = BeginPaint(window, &mut ps);
                            GetClientRect(window, &mut rect);
                            FillRect(hdc, &rect, hbrush);
                            EndPaint(window, &ps);
                            thread::sleep(time::Duration::from_millis(200));
                        }
         
                        clicked = false;
                        InvalidateRect(window, &mut rect, true);
                        LRESULT(0)
                    }
         
                    WM_DESTROY => {
                        println!("WM_DESTROY");
                        PostQuitMessage(0);
                        LRESULT(0)
                    }
         
                    WM_LBUTTONDOWN	=>{
                        println!("Clicked! {}", clicked);
                        clicked = true;
                        GetClientRect(window, &mut rect);
                        InvalidateRect(window, &mut rect, true);
         
                        LRESULT(0)
                    }
         
                    WM_ERASEBKGND =>{
                        println!("WM_ERASEBKGND Called!");
                        let hdc = BeginPaint(window, &mut ps);
                        hbrush = CreateSolidBrush( 0x00000000 as u32);
                        GetClientRect(window, &mut rect);
                        FillRect(hdc, &rect, hbrush);
                        EndPaint(window, &ps);
         
                        LRESULT(0)
                    }
                    _ => DefWindowProcA(window, message, wparam, lparam),
                }
            }
        }
}


fn main() -> Result<()> {
    unsafe {
        CoInitializeEx(std::ptr::null(), COINIT_MULTITHREADED)?;
    }
    let mut window = Window::new()?;
    window.run()
}