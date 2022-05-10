// Miniscript
// Written in 2019 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! Example: Verifying a signed transaction.

use bitcoin::consensus::Decodable;
use bitcoin::secp256k1::{self, Secp256k1};
use bitcoin::util::sighash;
use miniscript::interpreter::KeySigPair;
use std::str::FromStr;

fn main() {
    //
    // Setup
    //

    let tx = hard_coded_transaction();
    let spk_input_1 = hard_coded_script_pubkey();

    let interpreter = miniscript::Interpreter::from_txdata(
        &spk_input_1,
        &tx.input[0].script_sig,
        &tx.input[0].witness,
        0,
        0,
    )
    .unwrap();

    let desc_string = interpreter.inferred_descriptor_string();
    println!("Descriptor: {}", desc_string);

    // To do sanity checks on the transaction using the interpreter parse the
    // descriptor with `from_str`.
    let _ = miniscript::Descriptor::<bitcoin::PublicKey>::from_str(&desc_string)
        .expect("sanity checks to pass");
    // Alternately, use `inferred_descriptor` which does sanity checks for us also.
    let _ = interpreter.inferred_descriptor().expect("same as from_str");

    // Example one
    //
    // Learn which keys were used, not bothering to verify the signatures
    // (trusting that if they're on the blockchain, standardness would've
    // required they be either valid or 0-length.

    println!("\n\nExample one:\n");

    for elem in interpreter.iter_assume_sigs() {
        // Don't bother checking signatures.
        match elem.expect("no evaluation error") {
            miniscript::interpreter::SatisfiedConstraint::PublicKey { key_sig } => {
                let (key, sig) = key_sig
                    .as_ecdsa()
                    .expect("expected ecdsa sig, found schnorr sig");

                println!("Signed with:\n key: {}\n sig: {}", key, sig);
            }
            _ => {}
        }
    }

    // Example two
    //
    // Verify the signatures to ensure that invalid signatures are not treated
    // as having participated in the script

    println!("\n\nExample two:\n");
    let secp = Secp256k1::new();

    // We can set prevouts to be empty list because this is a legacy transaction
    // and this information is not required for sighash computation.
    let prevouts = sighash::Prevouts::All::<bitcoin::TxOut>(&[]);

    for elem in interpreter.iter(&secp, &tx, 0, &prevouts) {
        match elem.expect("no evaluation error") {
            miniscript::interpreter::SatisfiedConstraint::PublicKey { key_sig } => {
                let (key, sig) = key_sig.as_ecdsa().unwrap();
                println!("Signed with:\n key: {}\n sig: {}", key, sig);
            }
            _ => {}
        }
    }

    // Example three
    //
    // Same, but with the wrong signature hash, to demonstrate what happens
    // given an apparently invalid script.
    let secp = Secp256k1::new();
    let message = secp256k1::Message::from_slice(&[0x01; 32][..]).expect("32-byte hash");

    let iter = interpreter.iter_custom(Box::new(|key_sig: &KeySigPair| {
        let (pk, ecdsa_sig) = key_sig.as_ecdsa().expect("Ecdsa Sig");
        ecdsa_sig.hash_ty == bitcoin::EcdsaSighashType::All
            && secp
                .verify_ecdsa(&message, &ecdsa_sig.sig, &pk.inner)
                .is_ok()
    }));

    println!("\n\nExample three:\n");

    for elem in iter {
        let error = elem.expect_err("evaluation error");
        println!("Evaluation error: {}", error);
    }
}

/// Returns an arbitrary transaction.
fn hard_coded_transaction() -> bitcoin::Transaction {
    // tx `f27eba163c38ad3f34971198687a3f1882b7ec818599ffe469a8440d82261c98`
    #[cfg_attr(feature="cargo-fmt", rustfmt_skip)]
    let tx_bytes = vec![
        0x01, 0x00, 0x00, 0x00, 0x02, 0xc5, 0x11, 0x1d, 0xb7, 0x93, 0x50, 0xc1,
        0x70, 0x28, 0x41, 0x39, 0xe8, 0xe3, 0x4e, 0xb0, 0xed, 0xba, 0x64, 0x7b,
        0x6c, 0x88, 0x7e, 0x9f, 0x92, 0x8f, 0xfd, 0x9b, 0x5c, 0x4a, 0x4b, 0x52,
        0xd0, 0x01, 0x00, 0x00, 0x00, 0xda, 0x00, 0x47, 0x30, 0x44, 0x02, 0x20,
        0x1c, 0xcc, 0x1b, 0xe9, 0xaf, 0x73, 0x4a, 0x10, 0x9f, 0x66, 0xfb, 0xed,
        0xeb, 0x77, 0xb7, 0xa1, 0xf4, 0xb3, 0xc5, 0xff, 0x3d, 0x7f, 0x46, 0xf6,
        0xde, 0x50, 0x69, 0xbb, 0x52, 0x7f, 0x26, 0x9d, 0x02, 0x20, 0x75, 0x37,
        0x2f, 0x6b, 0xd7, 0x0c, 0xf6, 0x45, 0x7a, 0xc7, 0x0e, 0x82, 0x6f, 0xc6,
        0xa7, 0x5b, 0xf7, 0xcf, 0x10, 0x8c, 0x92, 0xea, 0xcf, 0xfc, 0xb5, 0xd9,
        0xfd, 0x77, 0x66, 0xa3, 0x58, 0xa9, 0x01, 0x48, 0x30, 0x45, 0x02, 0x21,
        0x00, 0xfe, 0x82, 0x5b, 0xe1, 0xd5, 0xfd, 0x71, 0x67, 0x83, 0xf4, 0x55,
        0xef, 0xe6, 0x6d, 0x61, 0x58, 0xff, 0xf8, 0xc3, 0x2b, 0x93, 0x1c, 0x5f,
        0x3f, 0xf9, 0x8e, 0x06, 0x65, 0xa9, 0xfd, 0x8e, 0x64, 0x02, 0x20, 0x22,
        0x01, 0x0f, 0xdb, 0x53, 0x8d, 0x0f, 0xa6, 0x8b, 0xd7, 0xf5, 0x20, 0x5d,
        0xc1, 0xdf, 0xa6, 0xc4, 0x28, 0x1b, 0x7b, 0xb7, 0x6f, 0xc2, 0x53, 0xf7,
        0x51, 0x4d, 0x83, 0x48, 0x52, 0x5f, 0x0d, 0x01, 0x47, 0x52, 0x21, 0x03,
        0xd0, 0xbf, 0x26, 0x7c, 0x93, 0x78, 0xb3, 0x18, 0xb5, 0x80, 0xc2, 0x10,
        0xa6, 0x78, 0xc4, 0xbb, 0x60, 0xd8, 0x44, 0x8b, 0x52, 0x0d, 0x21, 0x25,
        0xa1, 0xbd, 0x37, 0x2b, 0x23, 0xae, 0xa6, 0x49, 0x21, 0x02, 0x11, 0xa8,
        0x2a, 0xa6, 0x94, 0x63, 0x99, 0x0a, 0x6c, 0xdd, 0x48, 0x36, 0x76, 0x36,
        0x6a, 0x44, 0xac, 0x3c, 0x98, 0xe7, 0x68, 0x54, 0x69, 0x84, 0x0b, 0xf2,
        0x7a, 0x72, 0x4e, 0x40, 0x5a, 0x7e, 0x52, 0xae, 0xfd, 0xff, 0xff, 0xff,
        0xea, 0x51, 0x1f, 0x33, 0x7a, 0xf5, 0x72, 0xbb, 0xad, 0xcd, 0x2e, 0x03,
        0x07, 0x71, 0x62, 0x3a, 0x60, 0xcc, 0x71, 0x82, 0xad, 0x74, 0x53, 0x3e,
        0xa3, 0x2f, 0xc8, 0xaa, 0x47, 0xd2, 0x0e, 0x71, 0x01, 0x00, 0x00, 0x00,
        0xda, 0x00, 0x48, 0x30, 0x45, 0x02, 0x21, 0x00, 0xfa, 0x2b, 0xfb, 0x4d,
        0x49, 0xb7, 0x6d, 0x9f, 0xb4, 0xc6, 0x9c, 0xc7, 0x8c, 0x36, 0xd2, 0x66,
        0x92, 0x40, 0xe4, 0x57, 0x14, 0xc7, 0x19, 0x06, 0x85, 0xf7, 0xe5, 0x13,
        0x94, 0xac, 0x4e, 0x37, 0x02, 0x20, 0x04, 0x95, 0x2c, 0xf7, 0x75, 0x1c,
        0x45, 0x9d, 0x8a, 0x8b, 0x64, 0x76, 0x76, 0xce, 0x86, 0xf3, 0xbd, 0x69,
        0xff, 0x39, 0x17, 0xcb, 0x99, 0x85, 0x14, 0xbd, 0x73, 0xb7, 0xfc, 0x04,
        0xf6, 0x4c, 0x01, 0x47, 0x30, 0x44, 0x02, 0x20, 0x31, 0xae, 0x81, 0x1e,
        0x35, 0x7e, 0x80, 0x00, 0x01, 0xc7, 0x57, 0x27, 0x7a, 0x22, 0x44, 0xa7,
        0x2b, 0xd5, 0x9d, 0x0a, 0x00, 0xbe, 0xde, 0x49, 0x0a, 0x96, 0x12, 0x3e,
        0x54, 0xce, 0x03, 0x4c, 0x02, 0x20, 0x05, 0xa2, 0x9f, 0x14, 0x30, 0x1e,
        0x5e, 0x2f, 0xdc, 0x7c, 0xee, 0x49, 0x43, 0xec, 0x78, 0x78, 0xdf, 0x73,
        0xde, 0x96, 0x27, 0x00, 0xa4, 0xd9, 0x43, 0x6b, 0xce, 0x24, 0xd6, 0xc3,
        0xa3, 0x57, 0x01, 0x47, 0x52, 0x21, 0x03, 0x4e, 0x74, 0xde, 0x0b, 0x84,
        0x3f, 0xaa, 0x60, 0x44, 0x3d, 0xf4, 0x76, 0xf1, 0xf6, 0x14, 0x4a, 0x5b,
        0x0e, 0x76, 0x49, 0x9e, 0x8a, 0x26, 0x71, 0x07, 0x36, 0x5b, 0x32, 0xfa,
        0xd5, 0xd0, 0xfd, 0x21, 0x03, 0xb4, 0xa6, 0x82, 0xc8, 0x6a, 0xd9, 0x06,
        0x38, 0x8f, 0x99, 0x52, 0x76, 0xf0, 0x84, 0x92, 0x72, 0x3a, 0x8c, 0x5f,
        0x32, 0x3c, 0x6a, 0xf6, 0x92, 0x97, 0x17, 0x40, 0x5d, 0x2e, 0x1b, 0x2f,
        0x70, 0x52, 0xae, 0xfd, 0xff, 0xff, 0xff, 0x02, 0xa7, 0x32, 0x75, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xa9, 0x14, 0xfb, 0xf7, 0x76, 0xff,
        0xeb, 0x3b, 0xb8, 0x89, 0xb2, 0x01, 0xa5, 0x3f, 0x5f, 0xb0, 0x55, 0x4f,
        0x6e, 0x6f, 0xa2, 0x56, 0x88, 0xac, 0x19, 0x88, 0x56, 0x01, 0x00, 0x00,
        0x00, 0x00, 0x17, 0xa9, 0x14, 0xd3, 0xb6, 0x1d, 0x34, 0xf6, 0x33, 0x7c,
        0xd7, 0xc0, 0x28, 0xb7, 0x90, 0xb0, 0xcf, 0x43, 0xe0, 0x27, 0xd9, 0x1d,
        0xe7, 0x87, 0x09, 0x5d, 0x07, 0x00,
    ];

    bitcoin::Transaction::consensus_decode(&mut &tx_bytes[..]).expect("decode transaction")
}

fn hard_coded_script_pubkey() -> bitcoin::Script {
    bitcoin::Script::from(vec![
        0xa9, 0x14, 0x92, 0x09, 0xa8, 0xf9, 0x0c, 0x58, 0x4b, 0xb5, 0x97, 0x4d, 0x58, 0x68, 0x72,
        0x49, 0xe5, 0x32, 0xde, 0x59, 0xf4, 0xbc, 0x87,
    ])
}
