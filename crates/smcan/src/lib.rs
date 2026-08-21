// Outright stolen from https://github.com/n0-computer/rcan/commit/6605540873c44df5feb3408194421311d40ffbfd
// Updated for entertainment value

// local modules
mod builder;
mod chain;
mod resolver;

#[cfg(test)]
mod tests;

//  Exported
pub use builder::ChainBuilder;
pub use chain::Chain;
pub use resolver::Resolver;

use std::ops::Add;

use anyhow::{bail, ensure, Context, Result};
use blake3::Hash;

use ed25519_dalek::{ed25519::signature::Signer, Signature, SIGNATURE_LENGTH};
pub use ed25519_dalek::{SigningKey, VerifyingKey};
use n0_future::time::{Duration, SystemTime};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const VERSION: u8 = 2;

/// Domain separation tag
pub const DST: &[u8] = b"smcan-1-delegation";

/// Stable serde for [`VerifyingKey`]: length-prefixed bytes in binary
/// formats, lowercase hex in human-readable ones. Goes through
/// [`serdect`] for its constant-time hex codec, and pins the wire
/// format independent of [`ed25519_dalek`]'s own serde impl.
mod verifying_key_serde {
    use ed25519_dalek::VerifyingKey;
    use serde::{de::Error, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(
        key: &VerifyingKey,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serdect::array::serialize_hex_lower_or_bin(key.as_bytes(), serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<VerifyingKey, D::Error> {
        let mut buf = [0u8; 32];
        serdect::array::deserialize_hex_or_bin(&mut buf, deserializer)?;
        VerifyingKey::from_bytes(&buf).map_err(D::Error::custom)
    }
}

/// Wire-format wrapper around an ed25519 [`Signature`] that serializes as
/// a fixed-length tuple of `SIGNATURE_LENGTH` bytes (no length prefix in
/// binary formats like postcard), and as a lowercase hex string in
/// human-readable formats.
struct SignatureWire([u8; SIGNATURE_LENGTH]);

impl Serialize for SignatureWire {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.collect_str(&format_args!("{}", hex::encode(self.0)))
        } else {
            use serde::ser::SerializeTuple;
            let mut tup = serializer.serialize_tuple(SIGNATURE_LENGTH)?;
            for b in &self.0 {
                tup.serialize_element(b)?;
            }
            tup.end()
        }
    }
}

impl<'de> Deserialize<'de> for SignatureWire {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = SignatureWire;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "an ed25519 signature ({} bytes)", SIGNATURE_LENGTH)
            }

            fn visit_str<E: serde::de::Error>(
                self,
                v: &str,
            ) -> std::result::Result<Self::Value, E> {
                let mut bytes = [0u8; SIGNATURE_LENGTH];
                hex::decode_to_slice(v, &mut bytes).map_err(E::custom)?;
                Ok(SignatureWire(bytes))
            }

            fn visit_bytes<E: serde::de::Error>(
                self,
                v: &[u8],
            ) -> std::result::Result<Self::Value, E> {
                if v.len() != SIGNATURE_LENGTH {
                    return Err(E::invalid_length(v.len(), &self));
                }
                let mut bytes = [0u8; SIGNATURE_LENGTH];
                bytes.copy_from_slice(v);
                Ok(SignatureWire(bytes))
            }

            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut bytes = [0u8; SIGNATURE_LENGTH];
                for (i, slot) in bytes.iter_mut().enumerate() {
                    *slot = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                }
                Ok(SignatureWire(bytes))
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(V)
        } else {
            deserializer.deserialize_tuple(SIGNATURE_LENGTH, V)
        }
    }
}

/// A trait for types that define a capability.
///
/// Capabilities can be compared using [`Capability::permits`], which determines
/// whether one capability grants permission to perform another.
///
/// A common implementation of this trait might be an enum representing different
/// RPC request types.
///
/// The `Capability` type must be serializable so it can be included in the signature
/// payload in an [`Rcan`].
pub trait Capability: Serialize {
    // Store  the hash.
    const KIND: &'static str;
    /// Determines if `self` permits `other`.
    ///
    /// Returns `true` if `self` grants permission to perform the `other` capability,
    /// otherwise returns `false`.
    fn permits(&self, other: &Self) -> bool;
}

/// A token for attenuated capability delegations
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Smcan<C> {
    /// The actual content.
    pub payload: Payload<C>,
    /// Signature over the serialized payload.
    pub signature: Signature,
}

impl<C: Serialize> Serialize for Smcan<C> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut tup = serializer.serialize_tuple(2)?;
        tup.serialize_element(&self.payload)?;
        tup.serialize_element(&SignatureWire(self.signature.to_bytes()))?;
        tup.end()
    }
}

impl<'de, C: Deserialize<'de> + Serialize> Deserialize<'de> for Smcan<C> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RcanVisitor<C>(std::marker::PhantomData<C>);

        impl<'de, C: Deserialize<'de> + Serialize> serde::de::Visitor<'de> for RcanVisitor<C> {
            type Value = Smcan<C>;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("an rcan token (payload, signature)")
            }

            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let payload: Payload<C> = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let SignatureWire(sig_bytes) = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let rcan = Smcan {
                    payload,
                    signature: Signature::from_bytes(&sig_bytes),
                };

                // Verify before yielding, so a deserialized `Rcan` is
                // always signature checked. Without this, serde wire
                // formats hand back an unverified token while only
                // `decode` checks the signature.
                rcan.verify_signature().map_err(serde::de::Error::custom)?;

                Ok(rcan)
            }
        }

        deserializer.deserialize_tuple(2, RcanVisitor::<C>(std::marker::PhantomData))
    }
}

#[derive(Clone, Serialize, Deserialize, derive_more::Debug, PartialEq, Eq)]
pub struct Payload<C> {
    /// The issuer
    #[debug("{}", hex::encode(issuer))]
    #[serde(with = "verifying_key_serde")]
    issuer: VerifyingKey,
    /// The intended audience
    #[debug("{}", hex::encode(audience))]
    #[serde(with = "verifying_key_serde")]
    audience: VerifyingKey,
    // A hash of the kind of capability
    kind: Hash,
    /// The origin of the capability
    capability_origin: CapabilityOrigin,
    /// The capability
    capability: C,
    /// Valid until unix timestamp in seconds.
    valid_until: Expires,
}

impl<C> Payload<C> {
    pub fn capability(&self) -> &C {
        &self.capability
    }

    pub fn capability_origin(&self) -> &CapabilityOrigin {
        &self.capability_origin
    }
}

#[derive(Clone, Serialize, Deserialize, derive_more::Debug, PartialEq, Eq)]
pub struct SourcePair {
    #[debug("{}", hex::encode(hash.as_bytes()))]
    hash: Hash,
    #[debug("{}", hex::encode(delegate))]
    #[serde(with = "verifying_key_serde")]
    delegate: VerifyingKey,
}

/// The potential origins of a capability.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum CapabilityOrigin {
    /// The origin is the issuer itself
    Issuer,
    IssuerTerminal,
    /// This is a delegation, with this key being the root of the delegation chain.
    Delegation(SourcePair),
    DelegationTerminal(SourcePair),
}

/// When an rcan expires
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, derive_more::Display)]
pub enum Expires {
    /// Never expires
    #[display("never")]
    Never,
    /// Valid until given unix timestamp in seconds
    #[display("{_0}")]
    At(u64),
}

pub struct RcanBuilder<'s, C> {
    issuer: &'s SigningKey,
    audience: VerifyingKey,
    capability_origin: CapabilityOrigin,
    capability: C,
}

impl<C> Smcan<C> {
    pub fn issuing_builder(
        issuer: &SigningKey,
        audience: VerifyingKey,
        capability: C,
    ) -> RcanBuilder<'_, C> {
        RcanBuilder {
            issuer,
            audience,
            capability_origin: CapabilityOrigin::Issuer,
            capability,
        }
    }

    pub fn issuing_terminal(
        issuer: &SigningKey,
        audience: VerifyingKey,
        capability: C,
    ) -> RcanBuilder<'_, C> {
        RcanBuilder {
            issuer,
            audience,
            capability_origin: CapabilityOrigin::IssuerTerminal,
            capability,
        }
    }

    pub fn delegating_builder(
        issuer: &SigningKey,
        audience: VerifyingKey,
        owner: VerifyingKey,
        hash: Hash,
        capability: C,
    ) -> RcanBuilder<'_, C> {
        RcanBuilder {
            issuer,
            audience,
            capability_origin: CapabilityOrigin::Delegation(SourcePair {
                hash: hash,
                delegate: owner,
            }),
            capability,
        }
    }

    pub fn delegating_terminal(
        issuer: &SigningKey,
        audience: VerifyingKey,
        owner: VerifyingKey,
        hash: Hash,
        capability: C,
    ) -> RcanBuilder<'_, C> {
        RcanBuilder {
            issuer,
            audience,
            capability_origin: CapabilityOrigin::DelegationTerminal(SourcePair {
                hash: hash,
                delegate: owner,
            }),
            capability,
        }
    }

    pub fn encode(&self) -> Vec<u8>
    where
        C: Serialize,
    {
        postcard::to_extend(self, vec![VERSION]).expect("vec")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self>
    where
        C: DeserializeOwned + Serialize,
    {
        let Some(version) = bytes.first() else {
            bail!("cannot decode, token is empty");
        };
        ensure!(*version == VERSION, "invalid version: {}", version);
        // `Rcan`'s `Deserialize` verifies the signature, so a successful
        // decode is already signature-checked.
        let rcan: Self = postcard::from_bytes(&bytes[1..]).context("decoding")?;
        Ok(rcan)
    }

    /// Verify the signature over the payload. The signed bytes are
    /// `DST ++ postcard(payload)`, matching [`RcanBuilder::sign`].
    fn verify_signature(&self) -> Result<()>
    where
        C: Serialize,
    {
        let signed = postcard::to_extend(&self.payload, DST.to_vec())?;
        self.payload
            .issuer
            .verify_strict(&signed, &self.signature)?;
        Ok(())
    }

    pub fn get_kind(&self) -> &Hash {
        &self.payload.kind
    }

    pub fn audience(&self) -> &VerifyingKey {
        &self.payload.audience
    }

    pub fn issuer(&self) -> &VerifyingKey {
        &self.payload.issuer
    }

    pub fn capability(&self) -> &C {
        self.payload.capability()
    }

    pub fn capability_origin(&self) -> &CapabilityOrigin {
        self.payload.capability_origin()
    }

    pub fn capability_issuer(&self) -> &VerifyingKey {
        match self.payload.capability_origin() {
            CapabilityOrigin::Issuer => &self.payload.issuer,
            CapabilityOrigin::IssuerTerminal => &self.payload.issuer,
            CapabilityOrigin::Delegation(ref root) => &root.delegate,
            CapabilityOrigin::DelegationTerminal(ref root) => &root.delegate,
        }
    }

    pub fn issuer_hash(&self) -> Option<Hash> {
        match self.payload.capability_origin() {
            CapabilityOrigin::Issuer => None,
            CapabilityOrigin::IssuerTerminal => None,
            CapabilityOrigin::Delegation(ref root) => Some(root.hash.clone()),
            CapabilityOrigin::DelegationTerminal(ref root) => Some(root.hash.clone()),
        }
    }

    pub fn is_terminal(&self) -> bool {
        match self.payload.capability_origin() {
            CapabilityOrigin::Issuer => false,
            CapabilityOrigin::IssuerTerminal => true,
            CapabilityOrigin::Delegation(_) => false,
            CapabilityOrigin::DelegationTerminal(_) => true,
        }
    }

    pub fn expires(&self) -> &Expires {
        &self.payload.valid_until
    }
}

impl<C> RcanBuilder<'_, C> {
    pub fn sign(self, valid_until: Expires) -> Smcan<C>
    where
        C: Serialize + Capability,
    {
        let payload = Payload {
            issuer: self.issuer.verifying_key(),
            audience: self.audience,
            kind: blake3::hash(C::KIND.as_bytes()),
            capability_origin: self.capability_origin,
            capability: self.capability,
            valid_until,
        };

        let to_sign = postcard::to_extend(&payload, DST.to_vec()).expect("vec");
        let signature = self.issuer.sign(&to_sign);

        Smcan { signature, payload }
    }
}

impl Expires {
    pub fn valid_for(duration: Duration) -> Self {
        Self::At(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("now is after UNIX_EPOCH")
                .add(duration)
                .as_secs(),
        )
    }

    pub fn is_valid_at(&self, time: SystemTime) -> bool {
        let time = time
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("time must be after UNIX_EPOCH")
            .as_secs();
        match self {
            Expires::Never => true,
            Expires::At(expiry) => *expiry >= time,
        }
    }
}

#[cfg(test)]
mod test {
    use testresult::TestResult;

    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    enum Rpc {
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

    #[test]
    fn test_simple_capabilitys() {
        assert!(Rpc::Read.permits(&Rpc::Read));
        assert!(Rpc::ReadWrite.permits(&Rpc::Read));
        assert!(Rpc::ReadWrite.permits(&Rpc::ReadWrite),);
        assert!(!Rpc::Read.permits(&Rpc::ReadWrite));
        assert!(!Rpc::Read.permits(&Rpc::All));
        assert!(Rpc::All.permits(&Rpc::All));
        assert!(Rpc::All.permits(&Rpc::Read));
        assert!(Rpc::All.permits(&Rpc::ReadWrite));
    }

    #[test]
    fn test_rcan_encoding() -> TestResult {
        let issuer = SigningKey::from_bytes(&[0u8; 32]);
        let audience = SigningKey::from_bytes(&[1u8; 32]);
        let rcan = Smcan::issuing_builder(&issuer, audience.verifying_key(), Rpc::ReadWrite)
            .sign(Expires::Never);

        println!("{}", hex::encode(rcan.encode()));
        println!(
            "{}",
            hex::encode(postcard::to_allocvec(&rcan.signature).unwrap())
        );
        println!("{:#?}", &rcan);
        assert_eq!(Smcan::decode(&rcan.encode())?, rcan);
        Ok(())
    }

    #[test]
    fn deserialize_rejects_forged_signature() {
        let issuer = SigningKey::from_bytes(&[0u8; 32]);
        let audience = SigningKey::from_bytes(&[1u8; 32]);
        let rcan = Smcan::issuing_builder(&issuer, audience.verifying_key(), Rpc::ReadWrite)
            .sign(Expires::Never);

        // A genuine token round-trips through serde.
        let mut wire = postcard::to_stdvec(&rcan).unwrap();
        assert_eq!(postcard::from_bytes::<Smcan<Rpc>>(&wire).unwrap(), rcan);

        // The trailing bytes are the signature. Zeroing them must make
        // deserialization fail rather than yield an unverified token.
        let n = wire.len();
        wire[n - SIGNATURE_LENGTH..].fill(0);
        assert!(postcard::from_bytes::<Smcan<Rpc>>(&wire).is_err());
    }

    #[test]
    fn test_expiry() {
        let issuer = SigningKey::from_bytes(&[0u8; 32]);
        let audience = SigningKey::from_bytes(&[1u8; 32]).verifying_key();
        let rcan = Smcan::issuing_builder(&issuer, audience, Rpc::All)
            .sign(Expires::valid_for(Duration::from_secs(60)));
        assert!(rcan.expires().is_valid_at(SystemTime::UNIX_EPOCH));
        let now = SystemTime::now();
        assert!(rcan.expires().is_valid_at(now));
        let future = now + Duration::from_secs(61);
        assert!(!rcan.expires().is_valid_at(future));
    }
}
