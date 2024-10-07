//! Utilities for creating and verifying proof that a given canister is a part of YRAL Backend canisters
mod types;

use std::{cell::RefCell, thread::LocalKey};

use candid::{CandidType, Principal};
use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use ic_stable_structures::{StableBTreeMap, Memory};
use types::{ManagementCanisterSchnorrPublicKeyReply, ManagementCanisterSchnorrPublicKeyRequest, ManagementCanisterSignatureReply, ManagementCanisterSignatureRequest, SchnorrAlgorithm, SchnorrKeyId};
use serde::{Serialize, Deserialize};

pub(crate) const THRESHOLD_SCHNORR_KEY: &str = {
    #[cfg(feature = "local")]
    {
        "dfx_test_key"
    }
    #[cfg(not(feature = "local"))]
    {
        "key_1"
    }
};

pub(crate) type LocalPoPStore<Store> = LocalKey<RefCell<Store>>;

pub struct PubKeyCache<M: Memory>(StableBTreeMap<Principal, Vec<u8>, M>);

impl<M: Memory> PubKeyCache<M> {
    pub fn init(memory: M) -> Self {
        Self(StableBTreeMap::init(memory))
    }

    async fn get_or_init_public_key<Store: PoPStore<M>>(store: &'static LocalPoPStore<Store>, principal: Principal) -> Result<VerifyingKey, String> {
        let maybe_pk = store.with_borrow(|store| {
            store.pubkey_cache().0.get(&principal)
        });
        if let Some(pk) = maybe_pk {
            return VerifyingKey::try_from(pk.as_slice())
                .map_err(|_| "invalid public key".to_string())
        }

        let derive_args = ManagementCanisterSchnorrPublicKeyRequest {
            derivation_path: vec![],
            canister_id: Some(principal),
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: THRESHOLD_SCHNORR_KEY.to_string(),
            }
        };
        let (key_res,): (ManagementCanisterSchnorrPublicKeyReply,) = ic_cdk::call(
            Principal::management_canister(),
            "schnorr_public_key",
            (derive_args,)
        )
            .await
            .map_err(|(_, msg)| {
                format!("unable to get public key: {msg}")
            })?;

        let key = key_res.public_key;
        let vk = VerifyingKey::try_from(key.as_slice())
            .map_err(|_| "invalid public key".to_string())?;
        store.with_borrow_mut(|store| {
            store.pubkey_cache_mut().0.insert(principal, key.clone());
        });

        Ok(vk)
    }
}

#[derive(Serialize)]
struct PoaMessage {
    prefix: &'static [u8],
    pub child: Principal,
}

impl PoaMessage {
    pub fn new(child: Principal) -> Self {
        Self {
            prefix: b"CHILD",
            child,
        }
    }

    pub fn serialize_cbor(&self) -> Vec<u8> {
        let mut bytes = vec![];
        ciborium::into_writer(self, &mut bytes)
            .expect("PoaMessage should serialize succesfully");

        bytes
    }
}

/// Proof that this canister id is a child of the parent canister 
#[derive(Clone, CandidType, Serialize, Deserialize)]
struct ProofOfChild {
    // Principal of the child
    principal: Principal,
    signature: Vec<u8>,
}

impl ProofOfChild {
    async fn new(child: Principal) -> Result<Self, String> {
        let message = PoaMessage::new(child);
        let sign_args = ManagementCanisterSignatureRequest {
            message: message.serialize_cbor(),
            derivation_path: vec![],
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: THRESHOLD_SCHNORR_KEY.to_string()
            }, 
        };

        let (sig_res,): (ManagementCanisterSignatureReply,) = ic_cdk::api::call::call_with_payment(
            Principal::management_canister(),
            "sign_with_schnorr",
            (sign_args,),
            25_000_000_000,
        )
        .await
        .map_err(|(_, msg)| format!("unable to sign: {msg}"))?;

        Ok(Self {
            principal: child,
            signature: sig_res.signature,
        })
    }

    pub fn verify(&self, parent_key: &VerifyingKey) -> Result<(), String> {
        let message = PoaMessage::new(self.principal);
        let message_raw = message.serialize_cbor();

        let sig = Signature::from_slice(&self.signature).map_err(|_| "invalid proof".to_string())?;

        parent_key.verify(&message_raw, &sig).map_err(|_| "invalid proof".to_string())?;

        Ok(())
    }
}

#[derive(Clone, CandidType, Serialize, Deserialize)]
pub struct ProofOfParticipation {
    chain: Vec<ProofOfChild>,
}

impl ProofOfParticipation {
    /// New PoP for platform orchestrator
    pub fn new_for_root() -> Self {
        Self {
            chain: vec![],
        }
    } 

    pub async fn derive_for_child(self, child: Principal) -> Result<Self, String> {
        let mut chain = self.chain;
        let proof = ProofOfChild::new(child).await?;
        chain.push(proof);
        Ok(Self {
            chain,
        })
    }

    /// Verify that the caller is a YRAL canister
    pub async fn verify_caller_is_participant<M: Memory, Store: PoPStore<M>>(&self, store: &'static LocalPoPStore<Store>) -> Result<(), String> {
        #[cfg(feature = "local")]
        {
            // Hack: Always pass on local testing node
            // a proper implementation requires deploying platform orchestrator locally
            Ok(())
        }
        #[cfg(not(feature = "local"))]
        {
            let platform_orchestrator = store.with_borrow(|s| s.platform_orchestrator());
            let canister = ic_cdk::caller();

            let mut parent = PubKeyCache::get_or_init_public_key(store, platform_orchestrator).await?;
            for proof in &self.chain {
                proof.verify(&parent)?;
                if proof.principal == canister {
                    return Ok(())
                }
                parent = PubKeyCache::get_or_init_public_key(store, proof.principal).await?;
            }

            Err("invalid proof".to_string())
        }
    }
}

pub trait PoPStore<M: Memory> {
    fn pubkey_cache(&self) -> &PubKeyCache<M>;

    fn pubkey_cache_mut(&mut self) -> &mut PubKeyCache<M>;

    fn platform_orchestrator(&self) -> Principal;
}