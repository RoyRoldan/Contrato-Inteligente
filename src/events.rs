use soroban_sdk::{symbol_short, Address, Env, String};

// Los eventos permiten que una app o un explorador se entere de lo que pasa
// en el contrato sin tener que leer todo su almacenamiento.

pub fn member_added(env: &Env, member: &Address) {
    env.events()
        .publish((symbol_short!("member"),), member.clone());
}

pub fn proposal_created(env: &Env, id: u32, title: &String) {
    env.events()
        .publish((symbol_short!("proposal"), id), title.clone());
}

pub fn vote_cast(env: &Env, id: u32, voter: &Address, in_favor: bool) {
    env.events()
        .publish((symbol_short!("vote"), id), (voter.clone(), in_favor));
}

pub fn proposal_closed(env: &Env, id: u32) {
    env.events().publish((symbol_short!("closed"), id), true);
}