//! Контур окна M0: FFI-вызовы только отсюда и только из главного потока.

use core::ffi::{CStr, c_void};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};

use svetlitsa_efl as ffi;

static CLICKED: AtomicI32 = AtomicI32::new(0);
static DEL_SEEN: AtomicI32 = AtomicI32::new(0);
static CB_THREAD_HASH: AtomicU64 = AtomicU64::new(0);

fn thread_hash() -> u64 {
    let mut h = DefaultHasher::new();
    std::thread::current().id().hash(&mut h);
    h.finish()
}

extern "C" fn on_del(_data: *mut c_void, _event: *const ffi::EflEvent) {
    CB_THREAD_HASH.store(thread_hash(), Ordering::SeqCst);
    DEL_SEEN.store(2, Ordering::SeqCst);
    ffi::exit_loop();
}

extern "C" fn on_item_event(_data: *mut c_void, _obj: *mut ffi::Eo, _event_info: *mut c_void) {
    CLICKED.store(1, Ordering::SeqCst);
}

extern "C" fn on_timeout(_data: *mut c_void) -> ffi::EinaBool {
    // Страховка от зависшего smoke: окно никто не закрыл — выходим сами.
    ffi::exit_loop();
    ffi::FALSE
}

pub struct Report {
    pub clicked: bool,
    pub del_seen: bool,
    pub callback_in_main_thread: bool,
}

/// Открывает окно со списком `items` и (если задан) edje-макетом `theme`,
/// крутит цикл EFL до закрытия окна либо тайм-аута.
///
/// # Ошибки
/// Возвращает `Err`, если не удалось инициализировать Elementary (например,
/// нет `DISPLAY`).
pub fn smoke_window(theme: Option<&CStr>, items: &[String]) -> Result<Report, String> {
    const W: i32 = 480;
    const H: i32 = 320;
    const HEADER_H: i32 = 110;

    ffi::init()?;
    CLICKED.store(0, Ordering::SeqCst);
    DEL_SEEN.store(0, Ordering::SeqCst);
    CB_THREAD_HASH.store(0, Ordering::SeqCst);

    let win = ffi::add_window();
    ffi::set_size(&win, W, H);
    ffi::add_del_callback(&win, on_del);

    if let Some(theme) = theme {
        let l = ffi::add_layout(&win, theme, c"svetlitsa/main", "Светлица");
        ffi::set_geometry(&l, 0, 0, W, HEADER_H);
        ffi::show(&l);
    }
    let g = ffi::add_genlist(&win, items, Some(on_item_event));
    ffi::set_geometry(&g, 0, HEADER_H, W, H - HEADER_H);
    ffi::show(&g);
    ffi::show(&win);

    ffi::add_timer(60.0, on_timeout);
    let main_hash = thread_hash();
    ffi::run();
    ffi::shutdown();

    Ok(Report {
        clicked: CLICKED.load(Ordering::SeqCst) == 1,
        del_seen: DEL_SEEN.load(Ordering::SeqCst) == 2,
        callback_in_main_thread: CB_THREAD_HASH.load(Ordering::SeqCst) == main_hash,
    })
}
