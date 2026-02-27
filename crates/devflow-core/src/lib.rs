//! Core logic and abstractions for the Devflow system.
//!
//! This crate defines the project configuration, command structures,
//! extension registry, and runtime profiles used across the Devflow workspace.

pub mod command;
pub mod config;
pub mod extension;
pub mod runtime;

pub use command::{CommandRef, PrimaryCommand};
pub use config::{DevflowConfig, ExtensionSource, TargetsConfig};
pub use extension::{ExecutionAction, Extension, ExtensionRegistry};
pub use runtime::RuntimeProfile;
