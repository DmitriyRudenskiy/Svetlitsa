//! Инфраструктура окна «Светлицы» (ТЗ 3.2).
//!
//! Окно Elementary с edje-макетом темы, список позиций из `Vec<String>`
//! через genlist и Eo-колбэк удаления окна. Всё взаимодействие с EFL — в
//! главном потоке (проверено smoke; тикет 03, M0).

#[cfg(feature = "efl")]
mod window;

#[cfg(feature = "efl")]
pub use window::{Report, smoke_window};
