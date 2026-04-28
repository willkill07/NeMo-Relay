// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::api::runtime::EventSubscriberFn;
use crate::api::runtime::current_scope_stack;
use crate::api::runtime::global_context;
use crate::api::shared::ensure_runtime_owner;
use crate::error::{FlowError, Result};

/// Register a global lifecycle event subscriber.
///
/// The subscriber is added to the process-wide registry and receives every
/// emitted scope, tool, LLM, and mark event until it is deregistered.
///
/// # Parameters
/// - `name`: Unique subscriber name in the global registry.
/// - `callback`: Subscriber callback invoked for each emitted event.
///
/// # Returns
/// A [`Result`] that is `Ok(())` when the subscriber was registered.
///
/// # Errors
/// Returns [`FlowError::AlreadyExists`] when another global subscriber is
/// already registered under the same name.
///
/// # Notes
/// Global subscribers remain active across scopes until explicitly removed.
pub fn register_subscriber(name: &str, callback: EventSubscriberFn) -> Result<()> {
    ensure_runtime_owner()?;
    let context = global_context();
    let mut state = context
        .write()
        .map_err(|error| FlowError::Internal(error.to_string()))?;
    if state.event_subscribers.contains_key(name) {
        return Err(FlowError::AlreadyExists(format!(
            "{name} subscriber already exists"
        )));
    }
    state.event_subscribers.insert(name.to_string(), callback);
    Ok(())
}

/// Deregister a global lifecycle event subscriber.
///
/// This removes the named subscriber from the process-wide registry.
///
/// # Parameters
/// - `name`: Global subscriber name to remove.
///
/// # Returns
/// A [`Result`] containing `true` when a subscriber was removed and `false`
/// when the name was not registered.
///
/// # Errors
/// Returns an error when the global registry lock cannot be acquired safely.
///
/// # Notes
/// Deregistration affects only future event delivery.
pub fn deregister_subscriber(name: &str) -> Result<bool> {
    ensure_runtime_owner()?;
    let context = global_context();
    let mut state = context
        .write()
        .map_err(|error| FlowError::Internal(error.to_string()))?;
    Ok(state.event_subscribers.remove(name).is_some())
}

/// Register a scope-local lifecycle event subscriber.
///
/// The subscriber remains active only while the target scope is still present
/// on the active scope stack.
///
/// # Parameters
/// - `scope_uuid`: UUID of the owning scope.
/// - `name`: Unique subscriber name within the owning scope.
/// - `callback`: Subscriber callback invoked for events emitted under that
///   scope hierarchy.
///
/// # Returns
/// A [`Result`] that is `Ok(())` when the subscriber was registered.
///
/// # Errors
/// Returns [`FlowError::NotFound`] when the scope does not exist on the active
/// stack and [`FlowError::AlreadyExists`] when the scope already owns a
/// subscriber with the same name.
///
/// # Notes
/// Scope-local subscribers are removed automatically when the owning scope is
/// popped.
pub fn scope_register_subscriber(
    scope_uuid: &uuid::Uuid,
    name: &str,
    callback: EventSubscriberFn,
) -> Result<()> {
    ensure_runtime_owner()?;
    let scope_stack = current_scope_stack();
    let mut guard = scope_stack.write().expect("scope stack lock poisoned");
    let registries = guard
        .local_registries_mut(scope_uuid)
        .ok_or_else(|| FlowError::NotFound(format!("scope {scope_uuid} not found")))?;
    if registries.event_subscribers.contains_key(name) {
        return Err(FlowError::AlreadyExists(format!(
            "{name} subscriber already exists"
        )));
    }
    registries
        .event_subscribers
        .insert(name.to_string(), callback);
    Ok(())
}

/// Deregister a scope-local lifecycle event subscriber.
///
/// This removes the named subscriber from the registry attached to a specific
/// active scope.
///
/// # Parameters
/// - `scope_uuid`: UUID of the owning scope.
/// - `name`: Scope-local subscriber name to remove.
///
/// # Returns
/// A [`Result`] containing `true` when a subscriber was removed and `false`
/// when the name was not registered on that scope.
///
/// # Errors
/// Returns [`FlowError::NotFound`] when the scope does not exist on the active
/// stack.
///
/// # Notes
/// Deregistration affects only future event delivery for that scope.
pub fn scope_deregister_subscriber(scope_uuid: &uuid::Uuid, name: &str) -> Result<bool> {
    ensure_runtime_owner()?;
    let scope_stack = current_scope_stack();
    let mut guard = scope_stack.write().expect("scope stack lock poisoned");
    let registries = guard
        .local_registries_mut(scope_uuid)
        .ok_or_else(|| FlowError::NotFound(format!("scope {scope_uuid} not found")))?;
    Ok(registries.event_subscribers.remove(name).is_some())
}
