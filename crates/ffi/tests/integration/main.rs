// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Integration test support for the NeMo Flow FFI crate.

use libc::c_char;
use nemo_flow::api::event::Event;
use nemo_flow::api::llm::{LlmAttributes, LlmHandle, LlmRequest};
use nemo_flow::api::runtime::{LlmExecutionNextFn, LlmStreamExecutionNextFn, ToolExecutionNextFn};
use nemo_flow::api::scope::{ScopeAttributes, ScopeHandle, ScopeType};
use nemo_flow::api::tool::{ToolAttributes, ToolHandle};
use nemo_flow::codec::request::AnnotatedLlmRequest as AnnotatedLLMRequest;
use nemo_flow::error::{FlowError, Result};
use nemo_flow_ffi::api::*;
use nemo_flow_ffi::callable::*;
use nemo_flow_ffi::convert::*;
use nemo_flow_ffi::error::*;
use nemo_flow_ffi::types::*;
use nemo_flow_ffi::{api, convert, error};
use serde_json::{Value as Json, json};
use std::ffi::{CStr, CString};
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::Stream;

unsafe fn nemo_flow_string_free_internal(ptr: *mut c_char) {
    unsafe { nemo_flow_string_free(ptr) };
}

mod api_tests;
mod callable_extra_tests;
#[path = "../unit/callable_tests.rs"]
mod callable_tests;
#[path = "../coverage/convert_tests.rs"]
mod convert_coverage_tests;
#[path = "../coverage/error_tests.rs"]
mod error_coverage_tests;
#[path = "../unit/types_tests.rs"]
mod types_tests;
