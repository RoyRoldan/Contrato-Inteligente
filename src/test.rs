#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, BytesN, Env, String,
};

// Documento de prueba y su firma ed25519 (hecha fuera de la cadena sobre su sha256).
const DOC: &[u8] = b"Acta: se aprueba reparar la tuberia";
const SIGNER: [u8; 32] = [0xea, 0x4a, 0x6c, 0x63, 0xe2, 0x9c, 0x52, 0x0a, 0xbe, 0xf5, 0x50, 0x7b, 0x13, 0x2e, 0xc5, 0xf9, 0x95, 0x47, 0x76, 0xae, 0xbe, 0xbe, 0x7b, 0x92, 0x42, 0x1e, 0xea, 0x69, 0x14, 0x46, 0xd2, 0x2c];
const SIG: [u8; 64] = [0x59, 0x0e, 0x42, 0xe2, 0x6b, 0xf5, 0xca, 0xca, 0xc6, 0xd3, 0xf3, 0x36, 0xbf, 0x44, 0x9a, 0xf0, 0x5b, 0x95, 0x95, 0x2e, 0x45, 0xcd, 0xf9, 0x66, 0xbd, 0xd5, 0x3c, 0xf6, 0x47, 0x6e, 0xff, 0x94, 0xa7, 0x97, 0xae, 0xa5, 0x27, 0xf6, 0xf7, 0x74, 0x3d, 0x03, 0xb2, 0x3a, 0x4b, 0x62, 0x65, 0x43, 0x5b, 0x4c, 0x3f, 0x70, 0x3b, 0xa4, 0xd3, 0xad, 0x7a, 0xd1, 0x7b, 0x05, 0x9c, 0x9b, 0xaa, 0x0e];

const DAY: u64 = 86_400;
const DAY_LEDGERS: u32 = 17_280;
const PLAZO: u64 = 10 * DAY;

fn hash(env: &Env, texto: &[u8]) -> BytesN<32> {
    env.crypto()
        .sha256(&Bytes::from_slice(env, texto))
        .to_bytes()
}

fn setup() -> (Env, VotingContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VotingContract, ());
    let client = VotingContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

fn nueva(env: &Env, client: &VotingContractClient, titulo: &str, plazo: u64) -> u32 {
    client.create_proposal(
        &String::from_str(env, titulo),
        &hash(env, DOC),
        &BytesN::from_array(env, &SIGNER),
        &BytesN::from_array(env, &SIG),
        &plazo,
    )
}

// Avanza el reloj del ledger: cuenta de ledgers (TTL) y segundos (plazos).
fn avanzar_dias(env: &Env, dias: u32) {
    env.ledger().with_mut(|li| {
        li.sequence_number += dias * DAY_LEDGERS;
        li.timestamp += dias as u64 * DAY;
    });
}

#[test]
fn votacion_completa() {
    let (env, client, _admin) = setup();
    let ana = Address::generate(&env);
    let beto = Address::generate(&env);
    let carla = Address::generate(&env);
    client.add_member(&ana);
    client.add_member(&beto);
    client.add_member(&carla);

    let id = nueva(&env, &client, "Arreglar la tuberia principal", PLAZO);
    assert_eq!(id, 1);

    client.vote(&id, &ana, &true);
    client.vote(&id, &beto, &true);
    client.vote(&id, &carla, &false);

    let p = client.get_proposal(&id);
    assert_eq!(p.yes, 2);
    assert_eq!(p.no, 1);
    assert!(p.open);
    assert_eq!(p.deadline, PLAZO);
    assert!(client.has_voted(&id, &ana));
}

#[test]
fn no_se_puede_votar_dos_veces() {
    let (env, client, _admin) = setup();
    let ana = Address::generate(&env);
    client.add_member(&ana);
    let id = nueva(&env, &client, "Comprar cisterna", PLAZO);

    client.vote(&id, &ana, &true);
    let res = client.try_vote(&id, &ana, &false);
    assert_eq!(res, Err(Ok(Error::AlreadyVoted)));
}

#[test]
fn quien_no_es_miembro_no_puede_votar() {
    let (env, client, _admin) = setup();
    let intruso = Address::generate(&env);
    let id = nueva(&env, &client, "Pintar el salon comunal", PLAZO);

    let res = client.try_vote(&id, &intruso, &true);
    assert_eq!(res, Err(Ok(Error::NotAMember)));
}

#[test]
fn no_se_vota_en_propuesta_cerrada() {
    let (env, client, _admin) = setup();
    let ana = Address::generate(&env);
    client.add_member(&ana);
    let id = nueva(&env, &client, "Cambiar horario de la feria", PLAZO);

    client.close_proposal(&id);
    let res = client.try_vote(&id, &ana, &true);
    assert_eq!(res, Err(Ok(Error::ProposalClosed)));
}

#[test]
fn propuesta_inexistente() {
    let (env, client, _admin) = setup();
    let ana = Address::generate(&env);
    client.add_member(&ana);

    let res = client.try_vote(&99, &ana, &true);
    assert_eq!(res, Err(Ok(Error::ProposalNotFound)));
}

#[test]
fn no_se_inicializa_dos_veces() {
    let (env, client, _admin) = setup();
    let otro = Address::generate(&env);
    let res = client.try_initialize(&otro);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn sin_inicializar_da_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VotingContract, ());
    let client = VotingContractClient::new(&env, &contract_id);
    let ana = Address::generate(&env);

    let res = client.try_add_member(&ana);
    assert_eq!(res, Err(Ok(Error::NotInitialized)));
}

#[test]
fn documento_original_se_verifica() {
    let (env, client, _admin) = setup();
    let id = nueva(&env, &client, "Reparar tuberia", PLAZO);

    let original = Bytes::from_slice(&env, DOC);
    assert!(client.verify_document(&id, &original));
}

#[test]
fn documento_alterado_no_se_verifica() {
    let (env, client, _admin) = setup();
    let id = nueva(&env, &client, "Reparar tuberia", PLAZO);

    let alterado = Bytes::from_slice(&env, b"Acta: se aprueba reparar la tuberia!");
    assert!(!client.verify_document(&id, &alterado));
}

#[test]
fn verificar_documento_de_propuesta_inexistente() {
    let (env, client, _admin) = setup();
    let data = Bytes::from_slice(&env, b"lo que sea");
    let res = client.try_verify_document(&42, &data);
    assert_eq!(res, Err(Ok(Error::ProposalNotFound)));
}

#[test]
fn firma_invalida_no_crea_la_propuesta() {
    let (env, client, _admin) = setup();
    let mut mala = SIG;
    mala[0] ^= 1; // cambiamos un solo bit de la firma

    let res = client.try_create_proposal(
        &String::from_str(&env, "Con firma falsa"),
        &hash(&env, DOC),
        &BytesN::from_array(&env, &SIGNER),
        &BytesN::from_array(&env, &mala),
        &PLAZO,
    );
    assert!(res.is_err());
}

#[test]
fn firma_de_otro_documento_no_sirve() {
    let (env, client, _admin) = setup();

    // La firma es valida, pero para otro documento distinto al que se declara.
    let res = client.try_create_proposal(
        &String::from_str(&env, "Documento cambiado"),
        &hash(&env, b"otro documento"),
        &BytesN::from_array(&env, &SIGNER),
        &BytesN::from_array(&env, &SIG),
        &PLAZO,
    );
    assert!(res.is_err());
}

#[test]
fn plazo_en_el_pasado_es_invalido() {
    let (env, client, _admin) = setup();
    avanzar_dias(&env, 5);

    let res = client.try_create_proposal(
        &String::from_str(&env, "Plazo vencido"),
        &hash(&env, DOC),
        &BytesN::from_array(&env, &SIGNER),
        &BytesN::from_array(&env, &SIG),
        &DAY, // ya paso: el reloj va por el dia 5
    );
    assert_eq!(res, Err(Ok(Error::InvalidDeadline)));
}

#[test]
fn no_se_vota_despues_del_plazo() {
    let (env, client, _admin) = setup();
    let ana = Address::generate(&env);
    let beto = Address::generate(&env);
    client.add_member(&ana);
    client.add_member(&beto);
    let id = nueva(&env, &client, "Con fecha limite", PLAZO);

    // Antes del plazo se puede votar.
    client.vote(&id, &ana, &true);

    // Movemos el reloj mas alla del plazo.
    avanzar_dias(&env, 11);
    let res = client.try_vote(&id, &beto, &true);
    assert_eq!(res, Err(Ok(Error::ProposalExpired)));

    // El resultado anterior se conserva.
    assert_eq!(client.get_proposal(&id).yes, 1);
}

#[test]
fn los_datos_siguen_vivos_gracias_al_ttl() {
    let (env, client, _admin) = setup();
    let ana = Address::generate(&env);
    client.add_member(&ana);
    let id = nueva(&env, &client, "Propuesta de larga duracion", 100 * DAY);

    // Pasan 20 dias y alguien vota: el contrato extiende la vigencia de los datos.
    avanzar_dias(&env, 20);
    client.vote(&id, &ana, &true);

    // Pasan otros 20 dias (40 en total, mas que los 30 iniciales).
    avanzar_dias(&env, 20);
    assert_eq!(client.get_proposal(&id).yes, 1);
    assert!(client.has_voted(&id, &ana));
}