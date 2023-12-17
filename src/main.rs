#[warn(unused_must_use)]
use chrono::{format::strftime, prelude::*, NaiveDateTime};
slint::include_modules!();

use std::{
    cell::RefCell,
    ffi::OsStr,
    os::windows::prelude::OsStrExt,
    ptr::{null, null_mut},
    sync::{Arc, Mutex, MutexGuard},
};

use slint::{Model, ModelRc, SharedString, VecModel};
use winapi::{
    shared::windef::HWND,
    um::winuser::{
        AddClipboardFormatListener, CreateWindowExW, GetMessageW, HWND_MESSAGE, MSG,
        WM_CLIPBOARDUPDATE,
    },
};

use clipboard::{ClipboardContext, ClipboardProvider};

fn main() -> Result<(), slint::PlatformError> {
    const MAX_NUM: usize = 20;
    let is_add_to_list = Arc::new(Mutex::new(true));
    let is_add_to_list_clone_copy = is_add_to_list.clone();
    let is_add_to_list_clone_del = is_add_to_list.clone();
    let ui = AppWindow::new()?;
    ui.on_copy_clicked(move |name| {
        let mut lock_is_add_to_list = is_add_to_list_clone_copy.lock().unwrap();
        *lock_is_add_to_list = false;
        // print!("name:{}", name);
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        let _ = ctx.set_contents(name.into());
    });
    let ui_handle_del = ui.as_weak();
    ui.on_del_clicked(move |idx| {
        let _ = ui_handle_del.upgrade_in_event_loop(move |handle| {
            let contents =
                slint::VecModel::from(handle.get_clipboard_contents().iter().collect::<Vec<_>>());
            contents.remove(idx as usize);
            handle.set_clipboard_contents(std::rc::Rc::new(contents).into());
        });
    });
    let ui_handle = ui.as_weak();
    ClipboardListen::run(move || {
        //TODO::剪贴板内容获取
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        match ctx.get_contents() {
            Ok(content) => {
                // println!("内容为:{}", content);
                let mut lock_is_add_to_list = is_add_to_list_clone_del.lock().unwrap();
                if !*lock_is_add_to_list {
                    *lock_is_add_to_list = true;
                    return;
                }
                let _ = ui_handle.upgrade_in_event_loop(move |handle| {
                    let is_use = handle.get_is_use();
                    if !is_use {
                        return;
                    }
                    let contents = slint::VecModel::from(
                        handle.get_clipboard_contents().iter().collect::<Vec<_>>(),
                    );
                    let row_count = contents.row_count();
                    if row_count >= MAX_NUM {
                        contents.remove(MAX_NUM - 1);
                    }
                    let lines = content.split("\n").count() as i32;
                    let date = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    contents.insert(
                        0,
                        ClipboardContents {
                            content: SharedString::from(content),
                            lines: lines,
                            date: SharedString::from(date),
                        },
                    );

                    handle.set_clipboard_contents(std::rc::Rc::new(contents).into());
                });
            }
            Err(_) => {}
        }
    });

    ui.run()
}
pub struct ClipboardListen {}

impl ClipboardListen {
    pub fn run<F: Fn() + Send + 'static>(callback: F) {
        std::thread::spawn(move || {
            for msg in Message::new() {
                match msg.message {
                    WM_CLIPBOARDUPDATE => callback(),
                    _ => (),
                }
            }
        });
    }
}

pub struct Message {
    hwnd: HWND,
}

impl Message {
    pub fn new() -> Self {
        // 创建消息窗口
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                str_to_lpcwstr("STATIC").as_ptr(),
                null(),
                0,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                null_mut(),
                // wnd_class.hInstance,
                null_mut(),
                null_mut(),
            )
        };
        if hwnd == null_mut() {
            panic!("CreateWindowEx failed");
        }

        unsafe { AddClipboardFormatListener(hwnd) };

        Self { hwnd }
    }

    fn get(&self) -> Option<MSG> {
        let mut msg = unsafe { std::mem::zeroed() };
        let ret = unsafe { GetMessageW(&mut msg, self.hwnd, 0, 0) };
        if ret == 1 {
            Some(msg)
        } else {
            None
        }
    }
}

impl Iterator for Message {
    type Item = MSG;

    fn next(&mut self) -> Option<Self::Item> {
        self.get()
    }
}

fn str_to_lpcwstr(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
