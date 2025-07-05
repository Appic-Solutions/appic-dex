// a proxy canister to fetch untrusted ledger canisters data, and verify them at the time of pool
// creation

use std::fmt::{self, Debug};

use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::call::RejectionCode;
use serde::de::DeserializeOwned;

const PROXY_CANISTER_ID: &str = "epulg-riaaa-aaaaj-a2erq-cai";

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CandidIcpToken {
    pub ledger_id: Principal,
    pub name: String,
    pub decimals: u8,
    pub symbol: String,
    pub token_type: IcpTokenType,
    pub logo: String,
    pub usd_price: String,
    pub fee: Nat,
    pub rank: Option<u32>,
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, CandidType, Deserialize)]
pub enum IcpTokenType {
    ICRC1,
    ICRC2,
    ICRC3,
    DIP20,
    Other(String),
}

pub async fn validate_icrc_ledger(icrc_ledger: Principal) -> Result<CandidIcpToken, CallError> {
    call_canister::<Principal, Result<CandidIcpToken, CallError>>(
        Principal::from_text(PROXY_CANISTER_ID).unwrap(),
        "get_icp_token",
        icrc_ledger,
    )
    .await?
}

async fn call_canister<I, O>(canister_id: Principal, method: &str, args: I) -> Result<O, CallError>
where
    I: CandidType + Debug + Send + 'static,
    O: CandidType + DeserializeOwned + Debug + 'static,
{
    let res: Result<(O,), _> = ic_cdk::api::call::call(canister_id, method, (&args,)).await;

    match res {
        Ok((output,)) => Ok(output),
        Err((code, msg)) => Err(CallError {
            method: method.to_string(),
            reason: Reason::from_reject(code, msg),
        }),
    }
}

/// Represents an error from a management canister call, such as
/// `sign_with_ecdsa`.
#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct CallError {
    pub method: String,
    pub reason: Reason,
}

impl CallError {
    /// Returns the name of the method that resulted in this error.
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Returns the failure reason.
    pub fn reason(&self) -> &Reason {
        &self.reason
    }
}

impl fmt::Display for CallError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "management call '{}' failed: {}",
            self.method, self.reason
        )
    }
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
/// The reason for the management call failure.
pub enum Reason {
    /// The canister does not have enough cycles to submit the request.
    OutOfCycles,
    /// The call failed with an error.
    CanisterError(String),
    /// The management canister rejected the signature request (not enough
    /// cycles, the ECDSA subnet is overloaded, etc.).
    Rejected(String),
    /// The call failed with a transient error. Retrying may help.
    TransientInternalError(String),
    /// The call failed with a non-transient error. Retrying will not help.
    InternalError(String),
}

impl fmt::Display for Reason {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfCycles => write!(fmt, "the canister is out of cycles"),
            Self::CanisterError(msg) => write!(fmt, "canister error: {}", msg),
            Self::Rejected(msg) => {
                write!(fmt, "the management canister rejected the call: {}", msg)
            }
            Reason::TransientInternalError(msg) => write!(fmt, "transient internal error: {}", msg),
            Reason::InternalError(msg) => write!(fmt, "internal error: {}", msg),
        }
    }
}

impl Reason {
    pub fn from_reject(reject_code: RejectionCode, reject_message: String) -> Self {
        match reject_code {
            RejectionCode::SysTransient => Self::TransientInternalError(reject_message),
            RejectionCode::CanisterError => Self::CanisterError(reject_message),
            RejectionCode::CanisterReject => Self::Rejected(reject_message),
            RejectionCode::NoError
            | RejectionCode::SysFatal
            | RejectionCode::DestinationInvalid
            | RejectionCode::Unknown => Self::InternalError(format!(
                "rejection code: {:?}, rejection message: {}",
                reject_code, reject_message
            )),
        }
    }
}
