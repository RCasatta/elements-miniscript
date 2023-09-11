// Written in 2019 by Sanket Kanjular and Andrew Poelstra
// SPDX-License-Identifier: CC0-1.0

use std::{error, fmt};

use elements::hashes::hash160;
use elements::hex::ToHex;
use elements::{secp256k1_zkp, taproot};
use {bitcoin, elements};

use super::BitcoinKey;
use crate::extensions::EvalError;

/// Detailed Error type for Interpreter
#[derive(Debug)]
pub enum Error {
    /// Could not satisfy, absolute locktime not met
    AbsoluteLocktimeNotMet(u32),
    /// Could not satisfy, lock time values are different units
    AbsoluteLocktimeComparisonInvalid(u32, u32),
    /// Cannot Infer a taproot descriptor
    /// Key spends cannot infer the internal key of the descriptor
    /// Inferring script spends is possible, but is hidden nodes are currently
    /// not supported in descriptor spec
    CannotInferTrDescriptors,
    /// Error parsing taproot control block
    ControlBlockParse(taproot::TaprootError),
    /// Tap control block(merkle proofs + tweak) verification error
    ControlBlockVerificationError,
    /// General Interpreter error.
    CouldNotEvaluate,
    /// EcdsaSig related error
    EcdsaSig(bitcoin::ecdsa::Error),
    /// We expected a push (including a `OP_1` but no other numeric pushes)
    ExpectedPush,
    /// The preimage to the hash function must be exactly 32 bytes.
    HashPreimageLengthMismatch,
    /// Incorrect scriptPubKey (pay-to-pubkeyhash) for the provided public key
    IncorrectPubkeyHash,
    /// Incorrect scriptPubKey for the provided redeem script
    IncorrectScriptHash,
    /// Incorrect scriptPubKey (pay-to-witness-pubkeyhash) for the provided public key
    IncorrectWPubkeyHash,
    /// Incorrect scriptPubKey for the provided witness script
    IncorrectWScriptHash,
    /// MultiSig missing at least `1` witness elements out of `k + 1` required
    InsufficientSignaturesMultiSig,
    /// Invalid Sighash type
    InvalidSchnorrSighashType(Vec<u8>),
    /// ecdsa Signature failed to verify
    InvalidEcdsaSignature(bitcoin::PublicKey),
    /// Signature failed to verify
    InvalidSchnorrSignature(bitcoin::key::XOnlyPublicKey),
    /// Last byte of this signature isn't a standard sighash type
    NonStandardSighash(Vec<u8>),
    /// Miniscript error
    Miniscript(crate::Error),
    /// MultiSig requires 1 extra zero element apart from the `k` signatures
    MissingExtraZeroMultiSig,
    /// Script abortion because of incorrect dissatisfaction for multisig.
    /// Any input witness apart from sat(0 sig ...) or nsat(0 0 ..) leads to
    /// this error. This is network standardness assumption and miniscript only
    /// supports standard scripts
    MultiSigEvaluationError,
    ///Witness must be empty for pre-segwit transactions
    NonEmptyWitness,
    ///ScriptSig must be empty for pure segwit transactions
    NonEmptyScriptSig,
    /// Script abortion because of incorrect dissatisfaction for Checksig.
    /// Any input witness apart from sat(sig) or nsat(0) leads to
    /// this error. This is network standardness assumption and miniscript only
    /// supports standard scripts
    // note that BitcoinKey is not exported, create a data structure to convey the same
    // information in error
    PkEvaluationError(PkEvalErrInner),
    /// The Public Key hash check for the given pubkey. This occurs in `PkH`
    /// node when the given key does not match to Hash in script.
    PkHashVerifyFail(hash160::Hash),
    /// Parse Error while parsing a `stack::Element::Push` as a Pubkey. Both
    /// 33 byte and 65 bytes are supported.
    PubkeyParseError,
    /// Parse Error while parsing a `stack::Element::Push` as a XOnlyPublicKey (32 bytes)
    XOnlyPublicKeyParseError,
    /// Could not satisfy, relative locktime not met
    RelativeLocktimeNotMet(u32),
    /// Forward-secp related errors
    Secp(secp256k1_zkp::Error),
    /// Miniscript requires the entire top level script to be satisfied.
    ScriptSatisfactionError,
    /// Schnorr Signature error
    SchnorrSig(elements::SchnorrSigError),
    /// Errors in signature hash calculations
    SighashError(elements::sighash::Error),
    /// Taproot Annex Unsupported
    TapAnnexUnsupported,
    /// An uncompressed public key was encountered in a context where it is
    /// disallowed (e.g. in a Segwit script or p2wpkh output)
    UncompressedPubkey,
    /// Got `stack::Element::Satisfied` or `stack::Element::Dissatisfied` when the
    /// interpreter was expecting `stack::Element::Push`
    UnexpectedStackBoolean,
    /// Unexpected Stack End, caused by popping extra elements from stack
    UnexpectedStackEnd,
    /// Unexpected Stack Push `stack::Element::Push` element when the interpreter
    /// was expecting a stack boolean `stack::Element::Satisfied` or
    /// `stack::Element::Dissatisfied`
    UnexpectedStackElementPush,
    /// Verify expects stack top element exactly to be `stack::Element::Satisfied`.
    /// This error is raised even if the stack top is `stack::Element::Push`.
    VerifyFailed,
    /// Incorrect Covenant Witness
    IncorrectCovenantWitness,
    /// Covenant witness size mismatch
    /// eg: supplied a witness at
    /// nVersion with 5 bytes instead of 4
    CovWitnessSizeErr {
        /// Position of the item in sighash Msg
        pos: usize,
        /// Expected size
        expected: usize,
        /// Actual size
        actual: usize,
    },
    /// Errors related to extensions.
    ArithError(EvalError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::AbsoluteLocktimeNotMet(n) => write!(
                f,
                "required absolute locktime CLTV of {} blocks, not met",
                n
            ),
            Error::AbsoluteLocktimeComparisonInvalid(n, lock_time) => write!(
                f,
                "could not satisfy, lock time values are different units n: {} lock_time: {}",
                n, lock_time
            ),
            Error::CannotInferTrDescriptors => write!(f, "Cannot infer taproot descriptors"),
            Error::ControlBlockParse(ref e) => write!(f, "Control block parse error {}", e),
            Error::ControlBlockVerificationError => {
                f.write_str("Control block verification failed")
            }
            Error::EcdsaSig(ref s) => write!(f, "Ecdsa sig error: {}", s),
            Error::ExpectedPush => f.write_str("expected push in script"),
            Error::CouldNotEvaluate => f.write_str("Interpreter Error: Could not evaluate"),
            Error::HashPreimageLengthMismatch => f.write_str("Hash preimage should be 32 bytes"),
            Error::IncorrectPubkeyHash => f.write_str("public key did not match scriptpubkey"),
            Error::IncorrectScriptHash => f.write_str("redeem script did not match scriptpubkey"),
            Error::IncorrectWPubkeyHash => {
                f.write_str("public key did not match scriptpubkey (segwit v0)")
            }
            Error::IncorrectWScriptHash => f.write_str("witness script did not match scriptpubkey"),
            Error::InsufficientSignaturesMultiSig => f.write_str("Insufficient signatures for CMS"),
            Error::InvalidSchnorrSighashType(ref sig) => write!(
                f,
                "Invalid sighash type for schnorr signature '{}'",
                sig.to_hex()
            ),
            Error::InvalidEcdsaSignature(pk) => write!(f, "bad ecdsa signature with pk {}", pk),
            Error::InvalidSchnorrSignature(pk) => write!(f, "bad schnorr signature with pk {}", pk),
            Error::NonStandardSighash(ref sig) => write!(
                f,
                "Non standard sighash type for signature '{}'",
                sig.to_hex()
            ),
            Error::NonEmptyWitness => f.write_str("legacy spend had nonempty witness"),
            Error::NonEmptyScriptSig => f.write_str("segwit spend had nonempty scriptsig"),
            Error::Miniscript(ref e) => write!(f, "parse error: {}", e),
            Error::MissingExtraZeroMultiSig => f.write_str("CMS missing extra zero"),
            Error::MultiSigEvaluationError => {
                f.write_str("CMS script aborted, incorrect satisfaction/dissatisfaction")
            }
            Error::PkEvaluationError(ref key) => write!(f, "Incorrect Signature for pk {}", key),
            Error::PkHashVerifyFail(ref hash) => write!(f, "Pubkey Hash check failed {}", hash),
            Error::PubkeyParseError => f.write_str("could not parse pubkey"),
            Error::XOnlyPublicKeyParseError => f.write_str("could not parse x-only pubkey"),
            Error::RelativeLocktimeNotMet(n) => {
                write!(f, "required relative locktime CSV of {} blocks, not met", n)
            }
            Error::ScriptSatisfactionError => f.write_str("Top level script must be satisfied"),
            Error::Secp(ref e) => fmt::Display::fmt(e, f),
            Error::SchnorrSig(ref s) => write!(f, "Schnorr sig error: {}", s),
            Error::SighashError(ref e) => fmt::Display::fmt(e, f),
            Error::TapAnnexUnsupported => f.write_str("Encountered annex element"),
            Error::UncompressedPubkey => {
                f.write_str("uncompressed pubkey in non-legacy descriptor")
            }
            Error::UnexpectedStackBoolean => {
                f.write_str("Expected Stack Push operation, found stack bool")
            }
            Error::UnexpectedStackElementPush => write!(f, "Got {}, expected Stack Boolean", 1),
            Error::UnexpectedStackEnd => f.write_str("unexpected end of stack"),
            Error::VerifyFailed => {
                f.write_str("Expected Satisfied Boolean at stack top for VERIFY")
            }
            Error::IncorrectCovenantWitness => f.write_str(
                "Covenant witness incorrect, the initial stack supplied for \
                covenant global context is incorrect",
            ),
            Error::CovWitnessSizeErr {
                pos,
                expected,
                actual,
            } => write!(
                f,
                "At script code item position{}: Expected size{}, got size {}",
                pos, expected, actual
            ),
            Error::ArithError(ref e) => write!(f, "{}", e),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        use self::Error::*;

        match self {
            AbsoluteLocktimeNotMet(_)
            | AbsoluteLocktimeComparisonInvalid(_, _)
            | CannotInferTrDescriptors
            | ControlBlockVerificationError
            | CouldNotEvaluate
            | ExpectedPush
            | HashPreimageLengthMismatch
            | IncorrectPubkeyHash
            | IncorrectScriptHash
            | IncorrectWPubkeyHash
            | IncorrectWScriptHash
            | InsufficientSignaturesMultiSig
            | InvalidEcdsaSignature(_)
            | InvalidSchnorrSignature(_)
            | InvalidSchnorrSighashType(_)
            | NonStandardSighash(_)
            | MissingExtraZeroMultiSig
            | MultiSigEvaluationError
            | NonEmptyWitness
            | NonEmptyScriptSig
            | PubkeyParseError
            | XOnlyPublicKeyParseError
            | PkEvaluationError(_)
            | PkHashVerifyFail(_)
            | RelativeLocktimeNotMet(_)
            | ScriptSatisfactionError
            | TapAnnexUnsupported
            | UncompressedPubkey
            | UnexpectedStackBoolean
            | UnexpectedStackEnd
            | UnexpectedStackElementPush
            | VerifyFailed => None,
            ControlBlockParse(e) => Some(e),
            EcdsaSig(e) => Some(e),
            Miniscript(e) => Some(e),
            Secp(e) => Some(e),
            SchnorrSig(e) => Some(e),
            SighashError(e) => Some(e),
            IncorrectCovenantWitness => None,
            CovWitnessSizeErr { .. } => None,
            ArithError(..) => None,
        }
    }
}

#[doc(hidden)]
impl From<secp256k1_zkp::Error> for Error {
    fn from(e: secp256k1_zkp::Error) -> Error {
        Error::Secp(e)
    }
}

#[doc(hidden)]
impl From<elements::SchnorrSigError> for Error {
    fn from(e: elements::SchnorrSigError) -> Error {
        Error::SchnorrSig(e)
    }
}

/// A type of representing which keys errored during interpreter checksig evaluation
// Note that we can't use BitcoinKey because it is not public
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PkEvalErrInner {
    /// Full Key
    FullKey(bitcoin::PublicKey),
    /// XOnly Key
    XOnlyKey(bitcoin::key::XOnlyPublicKey),
}

impl From<BitcoinKey> for PkEvalErrInner {
    fn from(pk: BitcoinKey) -> Self {
        match pk {
            BitcoinKey::Fullkey(pk) => PkEvalErrInner::FullKey(pk),
            BitcoinKey::XOnlyPublicKey(xpk) => PkEvalErrInner::XOnlyKey(xpk),
        }
    }
}

impl fmt::Display for PkEvalErrInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PkEvalErrInner::FullKey(pk) => pk.fmt(f),
            PkEvalErrInner::XOnlyKey(xpk) => xpk.fmt(f),
        }
    }
}

#[doc(hidden)]
impl From<elements::sighash::Error> for Error {
    fn from(e: elements::sighash::Error) -> Error {
        Error::SighashError(e)
    }
}

impl From<elements::secp256k1_zkp::UpstreamError> for Error {
    fn from(e: elements::secp256k1_zkp::UpstreamError) -> Error {
        Error::Secp(elements::secp256k1_zkp::Error::Upstream(e))
    }
}

#[doc(hidden)]
impl From<crate::Error> for Error {
    fn from(e: crate::Error) -> Error {
        Error::Miniscript(e)
    }
}
