mod errors;
pub mod secret;
mod token;
use errors::Error;
use num_traits::CheckedAdd;
use secret::Secret;
use serde::{Deserialize, Serialize};

use crate::{Amount, dhke::hash_to_curve, nut01::PublicKey, nut02::KeysetId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashuError {
    code: u16,
    detail: String,
}

impl CashuError {
    pub fn new(code: u16, detail: String) -> Self {
        Self { code, detail }
    }

    pub fn code(&self) -> u16 {
        self.code
    }
    pub fn detail(&self) -> &String {
        &self.detail
    }
}

/// List of [Proof]
pub type Proofs = Vec<Proof>;

/// Utility methods for [Proofs]
pub trait ProofsMethods {
    /// Try to sum up the amounts of all [Proof]s
    fn total_amount(&self) -> Result<Amount, Error>;

    /// Try to fetch the pubkeys of all [Proof]s
    fn ys(&self) -> Result<Vec<PublicKey>, Error>;
}

impl ProofsMethods for Proofs {
    fn total_amount(&self) -> Result<Amount, Error> {
        self.iter().try_fold(Amount::ZERO, |acc, v| {
            acc.checked_add(&v.amount).ok_or(Error::Overflow)
        })
    }

    fn ys(&self) -> Result<Vec<PublicKey>, Error> {
        self.iter()
            .map(|p| p.y())
            .collect::<Result<Vec<PublicKey>, _>>()
    }
}

/// Proofs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof {
    /// Amount
    pub amount: Amount,
    /// `Keyset id`
    #[serde(rename = "id")]
    pub keyset_id: KeysetId,
    /// Secret message
    pub secret: Secret,
    /// Unblind signature
    #[serde(rename = "C")]
    pub c: PublicKey,
}

impl Proof {
    /// Get y from proof
    ///
    /// Where y is `hash_to_curve(secret)`
    pub fn y(&self) -> Result<PublicKey, Error> {
        Ok(hash_to_curve(self.secret.as_ref())?)
    }
}

/// Blind Signature (also called `promise`)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlindSignature {
    /// Amount
    ///
    /// The value of the blind token.
    pub amount: Amount,
    /// Keyset ID
    ///
    /// ID of the mint keys that signed the token.
    #[serde(rename = "id")]
    pub keyset_id: KeysetId,
    /// Blind signature (C_)
    ///
    /// The blind signature on the secret message `B_` of [BlindMessage].
    #[serde(rename = "C_")]
    pub c: PublicKey,
}

/// Blind Message (also called `output`)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlindedMessage {
    /// Amount
    ///
    /// The value for the requested [BlindSignature]
    pub amount: Amount,
    /// Keyset ID
    ///
    /// ID from which we expect a signature.
    #[serde(rename = "id")]
    pub keyset_id: KeysetId,
    /// Blinded secret message (B_)
    ///
    /// The blinded secret message generated by the sender.
    #[serde(rename = "B_")]
    pub blinded_secret: PublicKey,
}
