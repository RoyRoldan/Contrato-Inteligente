use soroban_sdk::{contracttype, BytesN, String};

/// Una propuesta que la comunidad vota.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub title: String,
    /// Huella sha256 del documento de respaldo (por ejemplo, el acta).
    pub doc_hash: BytesN<32>,
    /// Llave pública ed25519 de quien firmó el documento (por ejemplo, la secretaría).
    pub signer: BytesN<32>,
    /// Momento (segundos, hora del ledger) en que termina la votación.
    pub deadline: u64,
    pub yes: u32,
    pub no: u32,
    pub open: bool,
}
