//! Extension Runtime
//!
//! Manages the execution environment for extensions, including:
//! - Background script execution
//! - Content script injection
//! - Message passing between scripts
//! - API access control

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use spacey_servo::SpaceyServo;

use crate::extensions::loader::{Extension, ExtensionId};
use crate::extensions::manifest::ContentScript;
use crate::extensions::apis::{
    ExtensionStorage, StorageArea, WebRequestApi, RequestDetails, BlockingResponse,
};

/// Extension runtime context
pub struct ExtensionRuntime {
    /// JavaScript engine for background scripts
    js_engine: Arc<RwLock<SpaceyServo>>,
    /// Storage API
    storage: Arc<ExtensionStorage>,
    /// WebRequest API
    webrequest: Arc<WebRequestApi>,
    /// Active background contexts
    background_contexts: RwLock<HashMap<ExtensionId, BackgroundContext>>,
    /// Registered message handlers
    message_handlers: RwLock<HashMap<ExtensionId, Vec<String>>>,
}

/// Background script context for an extension
pub struct BackgroundContext {
    /// Extension ID
    pub extension_id: ExtensionId,
    /// Whether the background is persistent (MV2) or event-based
    pub persistent: bool,
    /// Whether the background script has been loaded
    pub loaded: bool,
    /// Registered alarms
    pub alarms: Vec<Alarm>,
}

/// Alarm for scheduled execution
#[derive(Debug, Clone)]
pub struct Alarm {
    pub name: String,
    pub scheduled_time: f64,
    pub period_in_minutes: Option<f64>,
}

/// Message sent between extension contexts
#[derive(Debug, Clone)]
pub struct ExtensionMessage {
    /// Source extension ID
    pub source_extension_id: Option<ExtensionId>,
    /// Source tab ID (if from content script)
    pub source_tab_id: Option<i64>,
    /// Source frame ID
    pub source_frame_id: Option<i64>,
    /// Message data
    pub data: serde_json::Value,
}

/// Port for long-lived connections
pub struct Port {
    /// Port name
    pub name: String,
    /// Connected extension ID
    pub extension_id: ExtensionId,
    /// Source tab ID
    pub sender_tab_id: Option<i64>,
    /// Whether the port is still connected
    pub connected: bool,
}

impl ExtensionRuntime {
    /// Create a new extension runtime
    pub fn new(storage_dir: std::path::PathBuf) -> Self {
        Self {
            js_engine: Arc::new(RwLock::new(SpaceyServo::new())),
            storage: Arc::new(ExtensionStorage::new(storage_dir.join("storage"))),
            webrequest: Arc::new(WebRequestApi::new()),
            background_contexts: RwLock::new(HashMap::new()),
            message_handlers: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize an extension's background context
    pub fn init_extension(&self, extension: &Extension) -> Result<(), RuntimeError> {
        log::info!("Initializing extension: {}", extension.id);

        // Create background context
        let context = BackgroundContext {
            extension_id: extension.id.clone(),
            persistent: extension.manifest.is_persistent_background(),
            loaded: false,
            alarms: Vec::new(),
        };

        self.background_contexts.write().insert(extension.id.clone(), context);

        // Load background scripts
        for script_path in extension.manifest.get_background_scripts() {
            self.load_background_script(extension, &script_path)?;
        }

        // Mark as loaded
        if let Some(ctx) = self.background_contexts.write().get_mut(&extension.id) {
            ctx.loaded = true;
        }

        Ok(())
    }

    /// Load a background script
    fn load_background_script(
        &self,
        extension: &Extension,
        script_path: &str,
    ) -> Result<(), RuntimeError> {
        let script_full_path = extension.path.join(script_path);
        
        let script_content = std::fs::read_to_string(&script_full_path)
            .map_err(|e| RuntimeError::ScriptLoadError(e.to_string()))?;

        // Inject browser API bindings
        let wrapped_script = self.wrap_background_script(&extension.id, &script_content);

        // Execute in JavaScript engine
        self.js_engine.write()
            .eval(&wrapped_script)
            .map_err(|e| RuntimeError::ScriptExecutionError(e))?;

        log::debug!("Loaded background script: {}", script_path);
        Ok(())
    }

    /// Wrap a background script with API bindings
    fn wrap_background_script(&self, extension_id: &str, script: &str) -> String {
        format!(r#"
(function(browser, chrome) {{
    'use strict';
    
    // Extension ID for API calls
    const EXTENSION_ID = "{}";
    
    // === browser.runtime API ===
    browser.runtime = {{
        id: EXTENSION_ID,
        getManifest: function() {{
            return __spacey_extension_getManifest(EXTENSION_ID);
        }},
        getURL: function(path) {{
            return "extension://" + EXTENSION_ID + "/" + path;
        }},
        sendMessage: function(message, options) {{
            return __spacey_extension_sendMessage(EXTENSION_ID, message, options);
        }},
        onMessage: {{
            addListener: function(callback) {{
                __spacey_extension_addMessageListener(EXTENSION_ID, callback);
            }},
            removeListener: function(callback) {{
                __spacey_extension_removeMessageListener(EXTENSION_ID, callback);
            }}
        }},
        onInstalled: {{
            addListener: function(callback) {{
                __spacey_extension_addInstalledListener(EXTENSION_ID, callback);
            }}
        }},
        onStartup: {{
            addListener: function(callback) {{
                __spacey_extension_addStartupListener(EXTENSION_ID, callback);
            }}
        }}
    }};
    
    // === browser.storage API ===
    browser.storage = {{
        local: {{
            get: function(keys) {{
                return __spacey_storage_get(EXTENSION_ID, "local", keys);
            }},
            set: function(items) {{
                return __spacey_storage_set(EXTENSION_ID, "local", items);
            }},
            remove: function(keys) {{
                return __spacey_storage_remove(EXTENSION_ID, "local", keys);
            }},
            clear: function() {{
                return __spacey_storage_clear(EXTENSION_ID, "local");
            }},
            getBytesInUse: function(keys) {{
                return __spacey_storage_getBytesInUse(EXTENSION_ID, "local", keys);
            }}
        }},
        sync: {{
            get: function(keys) {{
                return __spacey_storage_get(EXTENSION_ID, "sync", keys);
            }},
            set: function(items) {{
                return __spacey_storage_set(EXTENSION_ID, "sync", items);
            }},
            remove: function(keys) {{
                return __spacey_storage_remove(EXTENSION_ID, "sync", keys);
            }},
            clear: function() {{
                return __spacey_storage_clear(EXTENSION_ID, "sync");
            }}
        }},
        session: {{
            get: function(keys) {{
                return __spacey_storage_get(EXTENSION_ID, "session", keys);
            }},
            set: function(items) {{
                return __spacey_storage_set(EXTENSION_ID, "session", items);
            }},
            remove: function(keys) {{
                return __spacey_storage_remove(EXTENSION_ID, "session", keys);
            }},
            clear: function() {{
                return __spacey_storage_clear(EXTENSION_ID, "session");
            }}
        }},
        onChanged: {{
            addListener: function(callback) {{
                __spacey_storage_addChangedListener(EXTENSION_ID, callback);
            }}
        }}
    }};
    
    // === browser.webRequest API (FULL BLOCKING SUPPORT!) ===
    browser.webRequest = {{
        onBeforeRequest: {{
            addListener: function(callback, filter, extraInfoSpec) {{
                __spacey_webRequest_addListener(
                    EXTENSION_ID, 
                    "onBeforeRequest", 
                    callback, 
                    filter, 
                    extraInfoSpec
                );
            }},
            removeListener: function(callback) {{
                __spacey_webRequest_removeListener(EXTENSION_ID, "onBeforeRequest", callback);
            }}
        }},
        onBeforeSendHeaders: {{
            addListener: function(callback, filter, extraInfoSpec) {{
                __spacey_webRequest_addListener(
                    EXTENSION_ID, 
                    "onBeforeSendHeaders", 
                    callback, 
                    filter, 
                    extraInfoSpec
                );
            }}
        }},
        onSendHeaders: {{
            addListener: function(callback, filter, extraInfoSpec) {{
                __spacey_webRequest_addListener(
                    EXTENSION_ID, 
                    "onSendHeaders", 
                    callback, 
                    filter, 
                    extraInfoSpec
                );
            }}
        }},
        onHeadersReceived: {{
            addListener: function(callback, filter, extraInfoSpec) {{
                __spacey_webRequest_addListener(
                    EXTENSION_ID, 
                    "onHeadersReceived", 
                    callback, 
                    filter, 
                    extraInfoSpec
                );
            }}
        }},
        onResponseStarted: {{
            addListener: function(callback, filter, extraInfoSpec) {{
                __spacey_webRequest_addListener(
                    EXTENSION_ID, 
                    "onResponseStarted", 
                    callback, 
                    filter, 
                    extraInfoSpec
                );
            }}
        }},
        onCompleted: {{
            addListener: function(callback, filter, extraInfoSpec) {{
                __spacey_webRequest_addListener(
                    EXTENSION_ID, 
                    "onCompleted", 
                    callback, 
                    filter, 
                    extraInfoSpec
                );
            }}
        }},
        onErrorOccurred: {{
            addListener: function(callback, filter, extraInfoSpec) {{
                __spacey_webRequest_addListener(
                    EXTENSION_ID, 
                    "onErrorOccurred", 
                    callback, 
                    filter, 
                    extraInfoSpec
                );
            }}
        }},
        // Filter/blocking response types
        filterResponseData: function(requestId) {{
            return __spacey_webRequest_filterResponseData(EXTENSION_ID, requestId);
        }},
        handlerBehaviorChanged: function() {{
            return __spacey_webRequest_handlerBehaviorChanged(EXTENSION_ID);
        }},
        MAX_HANDLER_BEHAVIOR_CHANGED_CALLS_PER_10_MINUTES: 20
    }};
    
    // === browser.tabs API ===
    browser.tabs = {{
        query: function(queryInfo) {{
            return __spacey_tabs_query(EXTENSION_ID, queryInfo);
        }},
        get: function(tabId) {{
            return __spacey_tabs_get(EXTENSION_ID, tabId);
        }},
        create: function(createProperties) {{
            return __spacey_tabs_create(EXTENSION_ID, createProperties);
        }},
        update: function(tabId, updateProperties) {{
            return __spacey_tabs_update(EXTENSION_ID, tabId, updateProperties);
        }},
        remove: function(tabIds) {{
            return __spacey_tabs_remove(EXTENSION_ID, tabIds);
        }},
        sendMessage: function(tabId, message, options) {{
            return __spacey_tabs_sendMessage(EXTENSION_ID, tabId, message, options);
        }},
        executeScript: function(tabId, details) {{
            return __spacey_tabs_executeScript(EXTENSION_ID, tabId, details);
        }},
        insertCSS: function(tabId, details) {{
            return __spacey_tabs_insertCSS(EXTENSION_ID, tabId, details);
        }},
        onCreated: {{
            addListener: function(callback) {{
                __spacey_tabs_addCreatedListener(EXTENSION_ID, callback);
            }}
        }},
        onUpdated: {{
            addListener: function(callback) {{
                __spacey_tabs_addUpdatedListener(EXTENSION_ID, callback);
            }}
        }},
        onRemoved: {{
            addListener: function(callback) {{
                __spacey_tabs_addRemovedListener(EXTENSION_ID, callback);
            }}
        }},
        onActivated: {{
            addListener: function(callback) {{
                __spacey_tabs_addActivatedListener(EXTENSION_ID, callback);
            }}
        }}
    }};
    
    // === browser.alarms API ===
    browser.alarms = {{
        create: function(name, alarmInfo) {{
            return __spacey_alarms_create(EXTENSION_ID, name, alarmInfo);
        }},
        get: function(name) {{
            return __spacey_alarms_get(EXTENSION_ID, name);
        }},
        getAll: function() {{
            return __spacey_alarms_getAll(EXTENSION_ID);
        }},
        clear: function(name) {{
            return __spacey_alarms_clear(EXTENSION_ID, name);
        }},
        clearAll: function() {{
            return __spacey_alarms_clearAll(EXTENSION_ID);
        }},
        onAlarm: {{
            addListener: function(callback) {{
                __spacey_alarms_addListener(EXTENSION_ID, callback);
            }}
        }}
    }};
    
    // Chrome compatibility (alias)
    const chromeApi = browser;
    
    // Execute the extension script
    {}
    
}})(typeof browser !== 'undefined' ? browser : {{}}, typeof chrome !== 'undefined' ? chrome : {{}});
"#, extension_id, script)
    }

    /// Get content scripts to inject for a URL
    pub fn get_content_scripts_for_url<'a>(
        &self,
        extensions: &'a [&Extension],
        url: &str,
    ) -> Vec<(&'a Extension, &'a ContentScript)> {
        let mut result = Vec::new();

        for extension in extensions {
            if !extension.enabled {
                continue;
            }

            for cs in &extension.manifest.content_scripts {
                if self.url_matches_content_script(url, cs) {
                    result.push((*extension, cs));
                }
            }
        }

        result
    }

    /// Check if URL matches content script
    fn url_matches_content_script(&self, url: &str, cs: &ContentScript) -> bool {
        // Check exclude patterns first
        for pattern in &cs.exclude_matches {
            if self.url_matches_pattern(url, pattern) {
                return false;
            }
        }

        // Check include patterns
        for pattern in &cs.matches {
            if self.url_matches_pattern(url, pattern) {
                return true;
            }
        }

        false
    }

    /// Match URL against pattern
    fn url_matches_pattern(&self, url: &str, pattern: &str) -> bool {
        if pattern == "<all_urls>" {
            return url.starts_with("http://") || url.starts_with("https://");
        }

        if pattern == "*://*/*" {
            return url.starts_with("http://") || url.starts_with("https://");
        }

        // Convert pattern to regex
        let pattern = pattern
            .replace(".", r"\.")
            .replace("*", ".*");

        regex::Regex::new(&format!("^{}$", pattern))
            .map(|re| re.is_match(url))
            .unwrap_or(false)
    }

    /// Generate content script injection code
    pub fn wrap_content_script(
        &self,
        extension_id: &str,
        script: &str,
        world: &str,
    ) -> String {
        format!(r#"
(function() {{
    'use strict';
    
    const EXTENSION_ID = "{}";
    const WORLD = "{}";
    
    // Minimal browser API for content scripts
    const browser = {{
        runtime: {{
            id: EXTENSION_ID,
            sendMessage: function(message) {{
                return __spacey_content_sendMessage(EXTENSION_ID, message);
            }},
            onMessage: {{
                addListener: function(callback) {{
                    __spacey_content_addMessageListener(EXTENSION_ID, callback);
                }}
            }},
            getURL: function(path) {{
                return "extension://" + EXTENSION_ID + "/" + path;
            }}
        }},
        storage: {{
            local: {{
                get: function(keys) {{
                    return __spacey_storage_get(EXTENSION_ID, "local", keys);
                }},
                set: function(items) {{
                    return __spacey_storage_set(EXTENSION_ID, "local", items);
                }}
            }}
        }}
    }};
    
    const chrome = browser;
    
    {}
    
}})();
"#, extension_id, world, script)
    }

    /// Unload an extension
    pub fn unload_extension(&self, extension_id: &str) {
        self.background_contexts.write().remove(extension_id);
        self.message_handlers.write().remove(extension_id);
        self.webrequest.remove_extension_listeners(extension_id);
        log::info!("Unloaded extension: {}", extension_id);
    }

    /// Get the storage API
    pub fn storage(&self) -> Arc<ExtensionStorage> {
        Arc::clone(&self.storage)
    }

    /// Get the webRequest API
    pub fn webrequest(&self) -> Arc<WebRequestApi> {
        Arc::clone(&self.webrequest)
    }

    /// Process a network request through extension listeners
    pub fn process_request(&self, details: &RequestDetails) -> Vec<BlockingResponse> {
        let responses = self.webrequest.process_before_request(details);
        responses.into_iter().map(|(_, r)| r).collect()
    }
}

/// Runtime errors
#[derive(Debug)]
pub enum RuntimeError {
    ScriptLoadError(String),
    ScriptExecutionError(String),
    ExtensionNotFound,
    ApiError(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::ScriptLoadError(e) => write!(f, "Script load error: {}", e),
            RuntimeError::ScriptExecutionError(e) => write!(f, "Script execution error: {}", e),
            RuntimeError::ExtensionNotFound => write!(f, "Extension not found"),
            RuntimeError::ApiError(e) => write!(f, "API error: {}", e),
        }
    }
}

impl std::error::Error for RuntimeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_pattern_matching() {
        let runtime = ExtensionRuntime::new(std::path::PathBuf::from("/tmp/test"));
        
        assert!(runtime.url_matches_pattern("https://example.com/page", "<all_urls>"));
        assert!(runtime.url_matches_pattern("http://test.com/", "*://*/*"));
        assert!(!runtime.url_matches_pattern("file:///local", "<all_urls>"));
    }
}
