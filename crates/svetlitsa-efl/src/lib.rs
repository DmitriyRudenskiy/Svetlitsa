//! Минимальный слой FFI к EFL/Elementary (ТЗ 3.2, ADR-0003, ADR-0010).
//!
//! Единственный крейт с `unsafe`. Имена символов отвечены от C-probe,
//! собранного в образе ворот `svetlitsa-gate:local` против установленных
//! заголовков EFL 1.29.0 (раунд 13, тикет 03): `efl_add`/`efl_show` в 1.29 —
//! макросы, а не символы; используем `_efl_add_internal_start`/`_efl_add_end`
//! и `efl_gfx_entity_visible_set`. Все вызовы — только из главного потока.
#![cfg(feature = "efl")]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use std::ffi::CString;

pub type EinaBool = c_int;
pub const TRUE: EinaBool = 1;
pub const FALSE: EinaBool = 0;

#[repr(C)]
pub struct Eo {
    _opaque: [u8; 0],
}
#[repr(C)]
pub struct EflClass {
    _opaque: [u8; 0],
}
#[repr(C)]
pub struct EflEventDescription {
    _opaque: [u8; 0],
}
#[repr(C)]
pub struct EflEvent {
    _opaque: [u8; 0],
}

/// `Eina_Rect` из eina_types: позиция 2×i32 + размер 2×i32.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct EinaRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EinaSize2D {
    pub w: i32,
    pub h: i32,
}

type TextGetCb = extern "C" fn(data: *mut c_void, obj: *mut Eo, part: *const c_char) -> *mut c_char;
type SimpleCb = extern "C" fn(data: *mut c_void, obj: *mut Eo, event_info: *mut c_void);
type EventCb = extern "C" fn(data: *mut c_void, event: *const EflEvent);
type TimerCb = extern "C" fn(data: *mut c_void) -> EinaBool;

/// `Elm_Gen_Item_Class_Functions` из elm_gen.h (EFL 1.29.0, проверено по
/// установленному заголовку — зеркалаем порядок полей один-в-один).
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ElmGenItemFuncs {
    pub text_get: Option<TextGetCb>,
    pub content_get:
        Option<extern "C" fn(data: *mut c_void, obj: *mut Eo, part: *const c_char) -> *mut Eo>,
    pub state_get:
        Option<extern "C" fn(data: *mut c_void, obj: *mut Eo, part: *const c_char) -> EinaBool>,
    pub del: Option<extern "C" fn(data: *mut c_void, obj: *mut Eo)>,
    pub filter_get:
        Option<extern "C" fn(data: *mut c_void, obj: *mut Eo, key: *mut c_void) -> EinaBool>,
    pub reusable_content_get: Option<
        extern "C" fn(
            data: *mut c_void,
            obj: *mut Eo,
            part: *const c_char,
            old: *mut Eo,
        ) -> *mut Eo,
    >,
}

/// `Elm_Gen_Item_Class` из elm_gen.h (он же `Elm_Genlist_Item_Class`).
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ElmGenlistItemClass {
    pub version: c_int,
    pub refcount: u32,
    pub delete_me: EinaBool,
    pub item_style: *const c_char,
    pub decorate_item_style: *const c_char,
    pub decorate_all_item_style: *const c_char,
    pub func: ElmGenItemFuncs,
}

unsafe extern "C" {
    // eina
    fn eina_init() -> EinaBool;
    fn eina_shutdown();
    fn strdup(s: *const c_char) -> *mut c_char;
    // eo/efl core
    static _EFL_EVENT_DEL: EflEventDescription;
    fn _efl_add_internal_start(
        file: *const c_char,
        line: c_int,
        klass: *const EflClass,
        parent: *const Eo,
        reffee: EinaBool,
        is_fallback: EinaBool,
    ) -> *mut Eo;
    fn _efl_add_end(obj: *mut Eo, reffee: EinaBool, is_fallback: EinaBool) -> *mut Eo;
    fn efl_event_callback_priority_add(
        obj: *const Eo,
        desc: *const EflEventDescription,
        priority: c_int,
        cb: EventCb,
        data: *const c_void,
    ) -> EinaBool;
    fn efl_gfx_entity_visible_set(obj: *mut Eo, visible: EinaBool);
    fn efl_gfx_entity_geometry_set(obj: *mut Eo, rect: EinaRect);
    fn efl_gfx_entity_size_set(obj: *mut Eo, size: EinaSize2D);
    // elementary
    fn elm_init(argc: c_int, argv: *const *const c_char) -> EinaBool;
    fn elm_run();
    fn elm_exit();
    fn elm_shutdown();
    fn efl_ui_win_class_get() -> *const EflClass;
    fn elm_layout_add(parent: *mut Eo) -> *mut Eo;
    fn elm_layout_file_set(obj: *mut Eo, file: *const c_char, group: *const c_char) -> EinaBool;
    fn elm_object_part_text_set(obj: *mut Eo, part: *const c_char, label: *const c_char);
    fn elm_genlist_add(parent: *mut Eo) -> *mut Eo;
    fn elm_genlist_item_class_new() -> *mut ElmGenlistItemClass;
    fn elm_genlist_item_append(
        obj: *mut Eo,
        itc: *const ElmGenlistItemClass,
        data: *mut c_void,
        parent: *mut c_void,
        it_type: c_int,
        func: Option<SimpleCb>,
        func_data: *mut c_void,
    ) -> *mut c_void;
    // ecore
    fn ecore_timer_add(interval: f64, func: TimerCb, data: *mut c_void) -> *mut c_void;
}

/// Точка входа: эквивалент `eina_init(); elm_init(0, NULL)`.
pub fn init() -> Result<(), &'static str> {
    unsafe {
        if eina_init() == FALSE {
            return Err("eina_init не удался");
        }
        if elm_init(0, ptr::null()) == FALSE {
            return Err("elm_init не удался: нет дисплея?");
        }
    }
    Ok(())
}

pub fn run() {
    unsafe { elm_run() }
}

pub fn shutdown() {
    unsafe {
        elm_shutdown();
        eina_shutdown();
    }
}

/// Аналог `efl_add_ref(EFL_UI_WIN_CLASS, NULL)`: окно — корневой объект,
/// в 1.29 создание без ref запрещено (ERR «should use efl_add_ref»).
pub fn add_window() -> *mut Eo {
    unsafe {
        let w = _efl_add_internal_start(
            c"svetlitsa-efl".as_ptr(),
            0,
            efl_ui_win_class_get(),
            ptr::null(),
            TRUE,
            FALSE,
        );
        _efl_add_end(w, TRUE, FALSE)
    }
}

pub fn show(obj: *mut Eo) {
    unsafe { efl_gfx_entity_visible_set(obj, TRUE) }
}

pub fn set_geometry(obj: *mut Eo, x: i32, y: i32, w: i32, h: i32) {
    unsafe { efl_gfx_entity_geometry_set(obj, EinaRect { x, y, w, h }) }
}

pub fn set_size(obj: *mut Eo, w: i32, h: i32) {
    unsafe { efl_gfx_entity_size_set(obj, EinaSize2D { w, h }) }
}

pub fn add_del_callback(obj: *mut Eo, cb: EventCb, data: *mut c_void) {
    unsafe {
        efl_event_callback_priority_add(obj, ptr::addr_of!(_EFL_EVENT_DEL), 0, cb, data);
    }
}

/// `elm_layout_file_set` + `elm_object_part_text_set(part, "title", text)`.
pub fn add_layout(parent: *mut Eo, theme_file: &CStr, group: &CStr, title: &str) -> *mut Eo {
    unsafe {
        let l = elm_layout_add(parent);
        elm_layout_file_set(l, theme_file.as_ptr(), group.as_ptr());
        let t = CString::new(title).unwrap_or_else(|_| CString::default());
        elm_object_part_text_set(l, c"title".as_ptr(), t.as_ptr());
        l
    }
}

/// Адаптер `&[String]` → класс items genlist + по одному `item_append` на строку.
///
/// Данные — указатели на `String` внутри среза; срез обязан пережить genlist
/// (в spike — живой `Vec` в кадре вызова до конца `run`).
pub fn add_genlist<T: AsRef<[String]>>(parent: *mut Eo, items: T, cb: Option<SimpleCb>) -> *mut Eo {
    extern "C" fn text_get(data: *mut c_void, _obj: *mut Eo, part: *const c_char) -> *mut c_char {
        // part == "default" или NULL для стиля default; иначе не наш элемент.
        if !part.is_null() {
            let part = unsafe { CStr::from_ptr(part) };
            if part != c"default" {
                return ptr::null_mut();
            }
        }
        let s = unsafe { &*(data.cast::<String>()) };
        unsafe { strdup(s.as_ptr().cast()) }
    }
    unsafe {
        let g = elm_genlist_add(parent);
        let itc = elm_genlist_item_class_new();
        (*itc).item_style = c"default".as_ptr();
        (*itc).func.text_get = Some(text_get);
        for s in items.as_ref() {
            let data = ptr::from_ref(s).cast::<c_void>().cast_mut();
            elm_genlist_item_append(g, itc, data, ptr::null_mut(), 0, cb, ptr::null_mut());
        }
        g
    }
}

/// `ecore_timer_add(seconds, cb, NULL)`.
pub fn add_timer(seconds: f64, cb: TimerCb) -> *mut c_void {
    unsafe { ecore_timer_add(seconds, cb, ptr::null_mut()) }
}

pub fn exit_loop() {
    unsafe { elm_exit() }
}
