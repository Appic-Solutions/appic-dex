use candid::Principal;

use crate::minter_client::{
    minter_types::{DexOrderArgs, DexOrderError},
    runtime::MinterRuntime,
};

pub mod minter_types;
pub mod runtime;

use runtime::Runtime;

pub struct MinterClient {
    minter_id: Principal,
    runtime: MinterRuntime,
}

impl MinterClient {
    pub fn new(minter_id: Principal) -> Self {
        Self {
            minter_id,
            runtime: MinterRuntime,
        }
    }

    pub async fn dex_order(
        &self,
        args: &DexOrderArgs,
    ) -> Result<Result<(), DexOrderError>, (i32, String)> {
        let result: Result<(), DexOrderError> = self
            .runtime
            .call(self.minter_id, "dex_order", (args,))
            .await
            .map(untuple)?;
        Ok(result)
    }
}

// extract the element from an unary tuple
fn untuple<T>(t: (T,)) -> T {
    t.0
}
