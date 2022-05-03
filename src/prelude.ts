// Exports common NEAR Rust SDK types based on https://docs.rs/near-sdk/4.0.0-pre.4/near_sdk/.

/// Represents an 64 bits unsigned integer encoded as a `string`.
/// See https://docs.rs/near-sdk/4.0.0-pre.4/near_sdk/json_types/struct.U64.html.
export type U64 = string;

/// Represents an 64 bits signed integer encoded as a `string`.
/// See https://docs.rs/near-sdk/4.0.0-pre.4/near_sdk/json_types/struct.I64.html.
export type I64 = string;

/// Represents an 128 bits unsigned integer encoded as a `string`.
/// See https://docs.rs/near-sdk/4.0.0-pre.4/near_sdk/json_types/struct.U128.html.
export type U128 = string;

/// Represents an 128 bits signed integer encoded as a `string`.
/// See https://docs.rs/near-sdk/4.0.0-pre.4/near_sdk/json_types/struct.I128.html.
export type I128 = string;

/// Represents an encoded array of bytes into a `string`.
/// See https://docs.rs/near-sdk/4.0.0-pre.4/near_sdk/json_types/struct.Base64VecU8.html.
export type Base64VecU8 = string;

/// Balance is a type for storing amounts of tokens, specified in yoctoNEAR.
/// See https://docs.rs/near-sdk/4.0.0-pre.4/near_sdk/type.Balance.html.
export type Balance = U128;

/// Account identifier. This is the human readable UTF8 string which is used internally to index accounts on the network and their respective state.
/// See https://docs.rs/near-sdk/4.0.0-pre.4/near_sdk/struct.AccountId.html.
export type AccountId = string;

/// DEPRECATED since 4.0.0.
/// See https://docs.rs/near-sdk/4.0.0-pre.4/near_sdk/json_types/type.ValidAccountId.html.
export type ValidAccountId = string;
