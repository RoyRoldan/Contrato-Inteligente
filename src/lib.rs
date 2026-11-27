#![no_std]

//! Votación comunitaria en Stellar (Soroban).
//!
//! Pensado para asociaciones de desarrollo, ASADAs o condominios:
//! el administrador registra a los miembros (el "padrón"), crea propuestas,
//! cada miembro vota una sola vez y cualquiera puede ver el resultado.
//!
//! Cada propuesta guarda la huella (sha256) de su documento, por ejemplo el acta,
//! y comprueba con ed25519_verify que alguien firmó esa huella. El documento se
//! queda fuera de la cadena; cualquiera puede comprobar después que un documento
//! es el original recalculando el hash. Cada propuesta tiene una fecha límite.

mod errors;
mod events;
mod storage;
mod types;
#[cfg(test)]
mod test;

use errors::Error;
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String};
use types::Proposal;

#[contract]
pub struct VotingContract;

/// Devuelve al administrador y exige que haya autorizado la llamada.
fn require_admin(env: &Env) -> Result<Address, Error> {
    let admin = storage::get_admin(env).ok_or(Error::NotInitialized)?;
    admin.require_auth();
    storage::extend_instance(env);
    Ok(admin)
}

#[contractimpl]
impl VotingContract {
    /// Se llama una sola vez para definir quién administra la votación.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if storage::get_admin(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        storage::set_count(&env, 0);
        storage::extend_instance(&env);
        Ok(())
    }

    /// El administrador agrega a un miembro al padrón.
    pub fn add_member(env: Env, member: Address) -> Result<(), Error> {
        require_admin(&env)?;
        storage::set_member(&env, &member);
        events::member_added(&env, &member);
        Ok(())
    }

    /// El administrador crea una propuesta nueva y recibe su id.
    /// - `doc_hash`: sha256 del documento que respalda la propuesta.
    /// - `signer` y `signature`: llave pública ed25519 y su firma (hecha fuera de la
    ///   cadena) sobre `doc_hash`. Si la firma no corresponde, la llamada falla.
    /// - `deadline`: momento en que termina la votación (debe estar en el futuro).
    pub fn create_proposal(
        env: Env,
        title: String,
        doc_hash: BytesN<32>,
        signer: BytesN<32>,
        signature: BytesN<64>,
        deadline: u64,
    ) -> Result<u32, Error> {
        require_admin(&env)?;
        if deadline <= env.ledger().timestamp() {
            return Err(Error::InvalidDeadline);
        }
        let msg: Bytes = doc_hash.clone().into();
        env.crypto().ed25519_verify(&signer, &msg, &signature);

        let id = storage::get_count(&env) + 1;
        let proposal = Proposal {
            title: title.clone(),
            doc_hash,
            signer,
            deadline,
            yes: 0,
            no: 0,
            open: true,
        };
        storage::set_proposal(&env, id, &proposal);
        storage::set_count(&env, id);
        events::proposal_created(&env, id, &title);
        Ok(id)
    }

    /// Un miembro vota a favor (true) o en contra (false). Solo una vez por propuesta.
    pub fn vote(env: Env, proposal_id: u32, voter: Address, in_favor: bool) -> Result<(), Error> {
        voter.require_auth();
        storage::extend_instance(&env);

        if !storage::is_member(&env, &voter) {
            return Err(Error::NotAMember);
        }

        let mut proposal =
            storage::get_proposal(&env, proposal_id).ok_or(Error::ProposalNotFound)?;
        if !proposal.open {
            return Err(Error::ProposalClosed);
        }
        if env.ledger().timestamp() > proposal.deadline {
            return Err(Error::ProposalExpired);
        }
        if storage::has_voted(&env, proposal_id, &voter) {
            return Err(Error::AlreadyVoted);
        }

        if in_favor {
            proposal.yes += 1;
        } else {
            proposal.no += 1;
        }

        storage::set_voted(&env, proposal_id, &voter);
        storage::set_proposal(&env, proposal_id, &proposal);
        events::vote_cast(&env, proposal_id, &voter, in_favor);
        Ok(())
    }

    /// El administrador cierra la votación; ya no se aceptan más votos.
    pub fn close_proposal(env: Env, proposal_id: u32) -> Result<(), Error> {
        require_admin(&env)?;
        let mut proposal =
            storage::get_proposal(&env, proposal_id).ok_or(Error::ProposalNotFound)?;
        proposal.open = false;
        storage::set_proposal(&env, proposal_id, &proposal);
        events::proposal_closed(&env, proposal_id);
        Ok(())
    }

    /// Consulta pública del estado y resultado de una propuesta.
    pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, Error> {
        storage::get_proposal(&env, proposal_id).ok_or(Error::ProposalNotFound)
    }

    /// Recalcula el sha256 de `data` y dice si coincide con el documento original.
    /// Si cambia un solo byte, el hash es otro y devuelve false.
    pub fn verify_document(env: Env, proposal_id: u32, data: Bytes) -> Result<bool, Error> {
        let proposal = storage::get_proposal(&env, proposal_id).ok_or(Error::ProposalNotFound)?;
        let hash: BytesN<32> = env.crypto().sha256(&data).to_bytes();
        Ok(hash == proposal.doc_hash)
    }

    /// Consulta si una cuenta ya votó en una propuesta.
    pub fn has_voted(env: Env, proposal_id: u32, voter: Address) -> bool {
        storage::has_voted(&env, proposal_id, &voter)
    }
}
