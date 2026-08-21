// Some tests for the Chain builder
#[cfg(test)]
mod test {

    // use super::*;
    use std::time::Duration;

    use crate::SigningKey;
    use crate::{Capability, ChainBuilder};
    use serde::{Deserialize, Serialize};
    use testresult::TestResult;

    #[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
    enum Rpc {
        #[default]
        Read,
        ReadWrite,
        /// Read, ReadWrite, and any "future ones" that we might not have thought of yet.
        All,
    }

    impl Capability for Rpc {
        const KIND: &'static str = "testing hash string thing";

        fn permits(&self, other: &Self) -> bool {
            match (self, other) {
                // `All` permits all RPC operations, by definition
                (Rpc::All, _) => true,
                // `ReadWrite` permits `Read` and `ReadWrite`, but not `All` (which may be extended later to include more caps)
                (Rpc::ReadWrite, Rpc::ReadWrite | Rpc::Read) => true,
                (Rpc::ReadWrite, _) => false,
                // `Read` only permits `Read`
                (Rpc::Read, Rpc::Read) => true,
                (Rpc::Read, _) => false,
            }
        }
    }

    type TestBuilder = ChainBuilder<Rpc>;

    // make some keys and and a builder for the tests.
    fn builder() -> (SigningKey, SigningKey, ChainBuilder<Rpc>) {
        // Some keys
        let issuer = SigningKey::from_bytes(&[0u8; 32]);
        let audience = SigningKey::from_bytes(&[1u8; 32]);
        let cb = TestBuilder::new(issuer.verifying_key());
        (issuer, audience, cb)
    }

    #[test]
    fn test_chain_builder() -> TestResult {
        // Some keys
        let (issuer, _audience, mut cb) = builder();

        cb.start(
            issuer.clone(),
            issuer.verifying_key(),
            Rpc::All,
            Duration::from_secs(20),
        )?;
        cb.show();
        cb.check_cap(Rpc::Read)?;
        Ok(())
    }
}
