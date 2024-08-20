use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM, RECT};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::{ClientToScreen, ScreenToClient};

use enigo::{Enigo, Mouse,
    Button, Coordinate,
    Direction::{Click, Press, Release},};

use anyhow::Result;

pub fn post_message_to_control(v_num: Vec<i32>) -> Result<()> {
    // let title_name = "植物大战僵尸杂交版v2.3.5 ";
    let title_name = "植物大战僵尸杂交版v2.0.88";

    let title_name_u16: Vec<u16> = OsStr::new(title_name)
        .encode_wide()
        .chain(std::iter::once(0)) // 添加 null terminator
        .collect();
    let hwnd: HWND = unsafe { FindWindowW(None, PCWSTR(title_name_u16.as_ptr())) }.unwrap();
    let mut point = POINT { x: v_num[0], y: v_num[1] };
    let mut rect: RECT = RECT::default();
    unsafe { GetClientRect(hwnd, &mut rect); }
    point.x = point.x * rect.right / v_num[2];
    point.y = point.y * rect.bottom / v_num[3]; 
    let lparam= (point.y as u32) << 16 | point.x as u32;
    let lparam = LPARAM(lparam as isize);
    // unsafe { ClientToScreen(hwnd, &mut point) };
    println!("{:?}", point);

    // let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();
    // enigo.move_mouse(point.x, point.y, enigo::Coordinate::Abs);
    // enigo.button(Button::Left, Press);
    // enigo.button(Button::Left, Release);

    unsafe {
        // PostMessageW(hwnd, WM_MOUSEMOVE, WPARAM(0), lparam);
        PostMessageW(hwnd, WM_LBUTTONDOWN, WPARAM(0), lparam);
        PostMessageW(hwnd, WM_LBUTTONUP, WPARAM(0), lparam);
    }
    Ok(())
}