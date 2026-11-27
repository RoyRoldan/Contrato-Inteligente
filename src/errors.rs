use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAMember = 3,
    ProposalNotFound = 4,
    ProposalClosed = 5,
    AlreadyVoted = 6,
    ProposalExpired = 7,
    InvalidDeadline = 8,
}
