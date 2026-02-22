//! Execution listener — global hook for observing TemplateExecutor events.
//!
//! This module provides a lightweight global listener pattern (similar to
//! `crate::tracing`) that downstream crates (e.g., xybrid-sdk) can use to
//! receive execution lifecycle events without coupling xybrid-core to the
//! telemetry pipeline.
//!
//! # Usage
//!
//! ```rust,ignore
//! use xybrid_core::execution::listener::{set_execution_listener, ExecutionEvent};
//!
//! set_execution_listener(|event| {
//!     match event {
//!         ExecutionEvent::Started { model_id, method } => { /* ... */ }
//!         ExecutionEvent::Completed { model_id, method, latency_ms } => { /* ... */ }
//!         ExecutionEvent::Failed { model_id, method, latency_ms, error } => { /* ... */ }
//!     }
//! });
//! ```

use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Events emitted by TemplateExecutor during execution.
#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    /// Emitted when an execute method begins.
    Started {
        model_id: String,
        /// Which method was called (e.g., "execute", "execute_streaming").
        method: String,
    },
    /// Emitted when an execute method completes successfully.
    Completed {
        model_id: String,
        method: String,
        latency_ms: u64,
    },
    /// Emitted when an execute method fails.
    Failed {
        model_id: String,
        method: String,
        latency_ms: u64,
        error: String,
    },
}

type ListenerFn = Box<dyn Fn(ExecutionEvent) + Send + Sync>;

lazy_static::lazy_static! {
    static ref EXECUTION_LISTENER: Arc<Mutex<Option<ListenerFn>>> = Arc::new(Mutex::new(None));
}

/// Register a global execution listener.
///
/// Only one listener can be active at a time; calling this replaces any
/// previously registered listener.
pub fn set_execution_listener(listener: impl Fn(ExecutionEvent) + Send + Sync + 'static) {
    if let Ok(mut l) = EXECUTION_LISTENER.lock() {
        *l = Some(Box::new(listener));
    }
}

/// Remove the currently registered execution listener.
pub fn clear_execution_listener() {
    if let Ok(mut l) = EXECUTION_LISTENER.lock() {
        *l = None;
    }
}

/// Emit an execution event to the registered listener (if any).
pub(crate) fn emit(event: ExecutionEvent) {
    if let Ok(l) = EXECUTION_LISTENER.lock() {
        if let Some(listener) = l.as_ref() {
            listener(event);
        }
    }
}

/// RAII guard that emits `Started` on creation and `Completed`/`Failed` on drop.
///
/// Call [`set_failed`](ExecutionGuard::set_failed) before dropping to emit a
/// `Failed` event instead of `Completed`.
pub(crate) struct ExecutionGuard {
    model_id: String,
    method: String,
    start: Instant,
    error: Mutex<Option<String>>,
}

impl ExecutionGuard {
    /// Create a new guard and immediately emit `ExecutionEvent::Started`.
    pub(crate) fn new(model_id: impl Into<String>, method: impl Into<String>) -> Self {
        let model_id = model_id.into();
        let method = method.into();
        emit(ExecutionEvent::Started {
            model_id: model_id.clone(),
            method: method.clone(),
        });
        Self {
            model_id,
            method,
            start: Instant::now(),
            error: Mutex::new(None),
        }
    }

    /// Mark this execution as failed. The error message will be included
    /// in the `Failed` event emitted on drop.
    pub(crate) fn set_failed(&self, error: impl Into<String>) {
        if let Ok(mut e) = self.error.lock() {
            *e = Some(error.into());
        }
    }
}

impl Drop for ExecutionGuard {
    fn drop(&mut self) {
        let latency_ms = self.start.elapsed().as_millis() as u64;
        let error = self.error.lock().ok().and_then(|mut e| e.take());
        match error {
            Some(err) => emit(ExecutionEvent::Failed {
                model_id: self.model_id.clone(),
                method: self.method.clone(),
                latency_ms,
                error: err,
            }),
            None => emit(ExecutionEvent::Completed {
                model_id: self.model_id.clone(),
                method: self.method.clone(),
                latency_ms,
            }),
        }
    }
}
