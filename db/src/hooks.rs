use std::cell::RefCell;

use fastrace::{local::LocalParentGuard, prelude::SpanContext, Span};
use pgrx::{pg_sys::ffi::pg_guard_ffi_boundary, prelude::*};

static mut PREV_EXECUTOR_START_HOOK: pg_sys::ExecutorStart_hook_type = None;
static mut PREV_EXECUTOR_END_HOOK: pg_sys::ExecutorEnd_hook_type = None;

thread_local! {
    static EXECUTION_SPAN: RefCell<Option<LocalParentGuard>> = const { RefCell::new(None) };
}

#[pg_guard]
unsafe extern "C-unwind" fn executor_start_hook(query: *mut pg_sys::QueryDesc, data: i32) {
    EXECUTION_SPAN.with(|span| {
        let mut span = span.borrow_mut();

        if span.is_none() {
            let new_span = Span::root("query-execution", SpanContext::random());
            let _ = span.insert(new_span.set_local_parent());
        }
    });

    if let Some(prev_hook) = PREV_EXECUTOR_START_HOOK {
        pg_guard_ffi_boundary(|| prev_hook(query, data));
    } else {
        pg_sys::standard_ExecutorStart(query, data);
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn executor_end_hook(query: *mut pg_sys::QueryDesc) {
    EXECUTION_SPAN.with(|span| {
        let mut span = span.borrow_mut();

        // will be dropped
        let _span = span.take();
    });

    if let Some(prev_hook) = PREV_EXECUTOR_END_HOOK {
        pg_guard_ffi_boundary(|| prev_hook(query));
    } else {
        pg_sys::standard_ExecutorEnd(query);
    }
}

pub unsafe fn register_hooks() {
    PREV_EXECUTOR_START_HOOK = pg_sys::ExecutorStart_hook;
    pg_sys::ExecutorStart_hook = Some(executor_start_hook);

    PREV_EXECUTOR_END_HOOK = pg_sys::ExecutorEnd_hook;
    pg_sys::ExecutorEnd_hook = Some(executor_end_hook);
}
