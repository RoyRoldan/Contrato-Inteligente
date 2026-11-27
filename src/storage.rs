use soroban_sdk::{contracttype, Address, Env};

use crate::types::Proposal;

// Un ledger dura unos 5 segundos: 17_280 ledgers son aproximadamente un día.
const DAY_IN_LEDGERS: u32 = 17_280;
const TTL_EXTEND_TO: u32 = 30 * DAY_IN_LEDGERS;
const TTL_THRESHOLD: u32 = TTL_EXTEND_TO - DAY_IN_LEDGERS;

#[contracttype]
pub enum DataKey {
    Admin,
    Count,
    Member(Address),
    Proposal(u32),
    Voted(u32, Address),
}

/// Mantiene vivos el administrador y el contador (almacenamiento de instancia).
pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

fn extend_persistent(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, TTL_THRESHOLD, TTL_EXTEND_TO);
}

// ---- Administrador ----

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

// ---- Contador de propuestas ----

pub fn get_count(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::Count).unwrap_or(0)
}

pub fn set_count(env: &Env, count: u32) {
    env.storage().instance().set(&DataKey::Count, &count);
}

// ---- Padrón de miembros ----

pub fn is_member(env: &Env, member: &Address) -> bool {
    let key = DataKey::Member(member.clone());
    let found: bool = env.storage().persistent().get(&key).unwrap_or(false);
    if found {
        extend_persistent(env, &key);
    }
    found
}

pub fn set_member(env: &Env, member: &Address) {
    let key = DataKey::Member(member.clone());
    env.storage().persistent().set(&key, &true);
    extend_persistent(env, &key);
}

// ---- Propuestas ----

pub fn get_proposal(env: &Env, id: u32) -> Option<Proposal> {
    let key = DataKey::Proposal(id);
    let proposal: Option<Proposal> = env.storage().persistent().get(&key);
    if proposal.is_some() {
        extend_persistent(env, &key);
    }
    proposal
}

pub fn set_proposal(env: &Env, id: u32, proposal: &Proposal) {
    let key = DataKey::Proposal(id);
    env.storage().persistent().set(&key, proposal);
    extend_persistent(env, &key);
}

// ---- Votos ya emitidos ----

pub fn has_voted(env: &Env, id: u32, voter: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Voted(id, voter.clone()))
}

pub fn set_voted(env: &Env, id: u32, voter: &Address) {
    let key = DataKey::Voted(id, voter.clone());
    env.storage().persistent().set(&key, &true);
    extend_persistent(env, &key);
}
