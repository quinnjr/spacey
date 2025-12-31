//! Browser Extension APIs
//!
//! Implements the WebExtensions APIs that Firefox extensions use.
//! We provide FULL API support, especially for:
//! - webRequest with blocking (critical for content blockers)
//! - storage (local, sync, session)
//! - tabs
//! - runtime
//! - alarms
//! - etc.

pub mod storage;
pub mod webrequest;

pub use storage::{ExtensionStorage, StorageArea, StorageChange, StorageError};
pub use webrequest::{
    WebRequestApi, RequestDetails, RequestFilter, BlockingResponse,
    ResourceType, HttpHeader, RequestAction, ExtraInfoSpec, WebRequestListener,
};
