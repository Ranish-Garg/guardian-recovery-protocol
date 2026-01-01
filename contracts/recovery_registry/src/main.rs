//! Recovery Registry Contract
//!
//! Manages guardian setup, tracking, and validation for accounts.
//! This is a coordination contract only - actual key rotation happens
//! via session WASM deploys signed by guardians.

#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{account::AccountHash, CLValue, Key, PublicKey, URef};

use guardian_types::{
    constants::{runtime_args as args, storage_keys, MIN_GUARDIANS},
    errors::GuardianError,
};

// Key generation helpers
fn guardians_key(account_hash: &AccountHash) -> String {
    format!("{}{}", storage_keys::GUARDIANS_PREFIX, account_hash)
}

fn threshold_key(account_hash: &AccountHash) -> String {
    format!("{}{}", storage_keys::THRESHOLD_PREFIX, account_hash)
}

fn initialized_key(account_hash: &AccountHash) -> String {
    format!("{}{}", storage_keys::INITIALIZED_PREFIX, account_hash)
}

// ============================================================================
// ENTRY POINTS
// ============================================================================

/// Initialize guardians for an account.
/// 
/// # Arguments
/// * `account_hash` - The account to set up guardians for
/// * `guardians` - List of guardian public keys (minimum 2)
/// * `threshold` - Number of guardians required for recovery approval
///
/// # Errors
/// * `InvalidGuardianSetup` - Less than 2 guardians or duplicate guardians
/// * `InvalidThreshold` - Threshold is 0 or greater than guardian count
/// * `AlreadyInitialized` - Guardians already set up for this account
#[no_mangle]
pub extern "C" fn initialize_guardians() {
    let account_hash: AccountHash = runtime::get_named_arg(args::ARG_ACCOUNT_HASH);
    let guardians: Vec<PublicKey> = runtime::get_named_arg(args::ARG_GUARDIANS);
    let threshold: u32 = runtime::get_named_arg(args::ARG_THRESHOLD);

    // Validate guardian setup
    if guardians.len() < MIN_GUARDIANS {
        runtime::revert(GuardianError::InvalidGuardianSetup);
    }

    if threshold == 0 || threshold > guardians.len() as u32 {
        runtime::revert(GuardianError::InvalidThreshold);
    }

    // Check for duplicate guardians
    let mut seen: Vec<&PublicKey> = Vec::new();
    for guardian in &guardians {
        if seen.iter().any(|&g| g == guardian) {
            runtime::revert(GuardianError::InvalidGuardianSetup);
        }
        seen.push(guardian);
    }

    // Check if already initialized
    let init_key = initialized_key(&account_hash);
    let init_uref = get_or_create_uref(&init_key);
    let already_initialized: bool = storage::read(init_uref)
        .unwrap_or_default()
        .unwrap_or(false);

    if already_initialized {
        runtime::revert(GuardianError::AlreadyInitialized);
    }

    // Store guardians list
    let guardians_uref = get_or_create_uref(&guardians_key(&account_hash));
    storage::write(guardians_uref, guardians.clone());

    // Store threshold
    let threshold_uref = get_or_create_uref(&threshold_key(&account_hash));
    storage::write(threshold_uref, threshold);

    // Mark as initialized
    storage::write(init_uref, true);
}

/// Get the list of guardians for an account.
///
/// # Arguments
/// * `account_hash` - The account to query
///
/// # Returns
/// * `Vec<PublicKey>` - List of guardian public keys
#[no_mangle]
pub extern "C" fn get_guardians() {
    let account_hash: AccountHash = runtime::get_named_arg(args::ARG_ACCOUNT_HASH);

    let key = guardians_key(&account_hash);
    let guardians: Vec<PublicKey> = read_from_storage(&key)
        .unwrap_or_revert_with(GuardianError::AccountNotFound);

    runtime::ret(CLValue::from_t(guardians).unwrap_or_revert());
}

/// Get the recovery threshold for an account.
///
/// # Arguments
/// * `account_hash` - The account to query
///
/// # Returns
/// * `u32` - Number of guardians required for approval
#[no_mangle]
pub extern "C" fn get_threshold() {
    let account_hash: AccountHash = runtime::get_named_arg(args::ARG_ACCOUNT_HASH);

    let key = threshold_key(&account_hash);
    let threshold: u32 = read_from_storage(&key)
        .unwrap_or_revert_with(GuardianError::AccountNotFound);

    runtime::ret(CLValue::from_t(threshold).unwrap_or_revert());
}

/// Check if a public key is a guardian for a specific account.
///
/// # Arguments
/// * `account_hash` - The account to check
/// * `public_key` - The public key to verify
///
/// # Returns
/// * `bool` - True if the key is a guardian for this account
#[no_mangle]
pub extern "C" fn is_guardian() {
    let account_hash: AccountHash = runtime::get_named_arg(args::ARG_ACCOUNT_HASH);
    let public_key: PublicKey = runtime::get_named_arg(args::ARG_PUBLIC_KEY);

    let key = guardians_key(&account_hash);
    let guardians: Option<Vec<PublicKey>> = read_from_storage(&key);

    let is_guardian = match guardians {
        Some(list) => list.iter().any(|g| g == &public_key),
        None => false,
    };

    runtime::ret(CLValue::from_t(is_guardian).unwrap_or_revert());
}

/// Check if an account has guardians set up.
///
/// # Arguments
/// * `account_hash` - The account to check
///
/// # Returns
/// * `bool` - True if guardians are initialized
#[no_mangle]
pub extern "C" fn has_guardians() {
    let account_hash: AccountHash = runtime::get_named_arg(args::ARG_ACCOUNT_HASH);

    let key = initialized_key(&account_hash);
    let has_guardians: bool = read_from_storage(&key).unwrap_or(false);

    runtime::ret(CLValue::from_t(has_guardians).unwrap_or_revert());
}

/// Default call entry point (required by Casper)
#[no_mangle]
pub extern "C" fn call() {
    // This entry point is called when the contract is deployed
    // Initialize any contract-level storage here if needed
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get existing URef or create a new one for the given key
fn get_or_create_uref(key_name: &str) -> URef {
    match runtime::get_key(key_name) {
        Some(Key::URef(uref)) => uref,
        _ => {
            let new_uref = storage::new_uref(());
            runtime::put_key(key_name, Key::URef(new_uref));
            new_uref
        }
    }
}

/// Read a value from storage by key name
fn read_from_storage<T: casper_types::CLTyped + casper_types::bytesrepr::FromBytes>(
    key_name: &str,
) -> Option<T> {
    match runtime::get_key(key_name) {
        Some(Key::URef(uref)) => storage::read(uref).ok().flatten(),
        _ => None,
    }
}
