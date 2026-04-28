// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! LLM codec types, traits, and built-in implementations.
//!
//! This module provides the type system and traits for bidirectional
//! request codecs ([`traits::LlmCodec`] / [`request::AnnotatedLlmRequest`]) and
//! the decode-only response codec
//! ([`traits::LlmResponseCodec`] / [`response::AnnotatedLlmResponse`]).

pub mod anthropic;
pub mod openai_chat;
pub mod openai_responses;
pub mod request;
pub mod response;
pub mod traits;
