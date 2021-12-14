// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::string::CefString;
use crate::LogSeverity;
use std::os::raw::c_int;
use wrflib_cef_sys::cef_settings_t;

#[derive(Debug, Copy, Clone)]
pub struct Settings<'a> {
    pub no_sandbox: bool,
    pub browser_subprocess_path: Option<&'a str>,
    pub framework_dir_path: Option<&'a str>,
    pub multi_threaded_message_loop: bool,
    pub external_message_pump: bool,
    pub windowless_rendering_enabled: bool,
    pub command_line_args_disabled: bool,
    pub cache_path: Option<&'a str>,
    pub user_data_path: Option<&'a str>,
    pub persist_session_cookies: bool,
    pub persist_user_preferences: bool,
    pub user_agent: Option<&'a str>,
    pub locale: Option<&'a str>,
    pub log_file: Option<&'a str>,
    pub log_severity: LogSeverity,
    pub javascript_flags: Option<&'a str>,
    pub resources_dir_path: Option<&'a str>,
    pub locales_dir_path: Option<&'a str>,
    pub pack_loading_disabled: bool,
    pub remote_debugging_port: Option<i32>,
    pub uncaught_exception_stack_size: Option<i32>,
    pub ignore_certificate_errors: bool,
    pub background_color: u32,
    pub accept_language_list: Option<&'a str>,
    pub main_bundle_path: Option<&'a str>,
    pub chrome_runtime: bool,
    pub root_cache_path: Option<&'a str>,
    pub user_agent_product: Option<&'a str>,
    pub cookieable_schemes_list: Option<&'a str>,
    pub cookieable_schemes_exclude_defaults: bool,
    pub application_client_id_for_file_scanning: Option<&'a str>,
}
impl<'a> Default for Settings<'a> {
    fn default() -> Settings<'a> {
        Settings {
            no_sandbox: true,
            browser_subprocess_path: None,
            framework_dir_path: None,
            multi_threaded_message_loop: false,
            external_message_pump: false,
            windowless_rendering_enabled: false,
            command_line_args_disabled: false,
            cache_path: None,
            user_data_path: None,
            persist_session_cookies: false,
            persist_user_preferences: false,
            user_agent: None,
            locale: None,
            log_file: None,
            log_severity: LogSeverity::LOGSEVERITY_DEFAULT,
            javascript_flags: None,
            resources_dir_path: None,
            locales_dir_path: None,
            pack_loading_disabled: false,
            remote_debugging_port: None,
            uncaught_exception_stack_size: None,
            ignore_certificate_errors: false,
            background_color: 0x0000_0000,
            accept_language_list: None,
            main_bundle_path: None,
            chrome_runtime: false,
            root_cache_path: None,
            user_agent_product: None,
            cookieable_schemes_list: None,
            cookieable_schemes_exclude_defaults: false,
            application_client_id_for_file_scanning: None,
        }
    }
}
impl<'a> Settings<'a> {
    pub(crate) fn to_cef(&self) -> cef_settings_t {
        cef_settings_t {
            size: std::mem::size_of::<cef_settings_t>() as u64,
            no_sandbox: self.no_sandbox as c_int,
            browser_subprocess_path: CefString::convert_str_to_cef(self.browser_subprocess_path),
            framework_dir_path: CefString::convert_str_to_cef(self.framework_dir_path),
            multi_threaded_message_loop: self.multi_threaded_message_loop as c_int,
            external_message_pump: self.external_message_pump as c_int,
            windowless_rendering_enabled: self.windowless_rendering_enabled as c_int,
            command_line_args_disabled: self.command_line_args_disabled as c_int,
            cache_path: CefString::convert_str_to_cef(self.cache_path),
            user_data_path: CefString::convert_str_to_cef(self.user_data_path),
            persist_session_cookies: self.persist_session_cookies as c_int,
            persist_user_preferences: self.persist_user_preferences as c_int,
            user_agent: CefString::convert_str_to_cef(self.user_agent),
            locale: CefString::convert_str_to_cef(self.locale),
            log_file: CefString::convert_str_to_cef(self.log_file),
            log_severity: self.log_severity,
            javascript_flags: CefString::convert_str_to_cef(self.javascript_flags),
            resources_dir_path: CefString::convert_str_to_cef(self.resources_dir_path),
            locales_dir_path: CefString::convert_str_to_cef(self.locales_dir_path),
            pack_loading_disabled: self.pack_loading_disabled as c_int,
            remote_debugging_port: self.remote_debugging_port.unwrap_or(0),
            uncaught_exception_stack_size: self.uncaught_exception_stack_size.unwrap_or(0),
            ignore_certificate_errors: self.ignore_certificate_errors as c_int,
            background_color: self.background_color,
            accept_language_list: CefString::convert_str_to_cef(self.accept_language_list),
            main_bundle_path: CefString::convert_str_to_cef(self.main_bundle_path),
            chrome_runtime: self.chrome_runtime as c_int,
            root_cache_path: CefString::convert_str_to_cef(self.root_cache_path),
            user_agent_product: CefString::convert_str_to_cef(self.user_agent_product),
            cookieable_schemes_list: CefString::convert_str_to_cef(self.cookieable_schemes_list),
            cookieable_schemes_exclude_defaults: self.cookieable_schemes_exclude_defaults as c_int,
            application_client_id_for_file_scanning: CefString::convert_str_to_cef(self.application_client_id_for_file_scanning),
        }
    }
}
