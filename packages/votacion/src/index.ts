import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}


export const networks = {
  testnet: {
    networkPassphrase: "Test SDF Network ; September 2015",
    contractId: "CC5O6FJ63CF53ZZM4DFZ7CI5FXVRDZJEUB7EQFCBTZ5ZT5CB4C4BXW5V",
  }
} as const


/**
 * Una propuesta que la comunidad vota.
 */
export interface Proposal {
  /**
 * Momento (segundos, hora del ledger) en que termina la votación.
 */
deadline: u64;
  /**
 * Huella sha256 del documento de respaldo (por ejemplo, el acta).
 */
doc_hash: Buffer;
  no: u32;
  open: boolean;
  /**
 * Llave pública ed25519 de quien firmó el documento (por ejemplo, la secretaría).
 */
signer: Buffer;
  title: string;
  yes: u32;
}

export const Errors = {
  1: {message:"AlreadyInitialized"},
  2: {message:"NotInitialized"},
  3: {message:"NotAMember"},
  4: {message:"ProposalNotFound"},
  5: {message:"ProposalClosed"},
  6: {message:"AlreadyVoted"},
  7: {message:"ProposalExpired"},
  8: {message:"InvalidDeadline"}
}

export type DataKey = {tag: "Admin", values: void} | {tag: "Count", values: void} | {tag: "Member", values: readonly [string]} | {tag: "Proposal", values: readonly [u32]} | {tag: "Voted", values: readonly [u32, string]};

export interface Client {
  /**
   * Construct and simulate a vote transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Un miembro vota a favor (true) o en contra (false). Solo una vez por propuesta.
   */
  vote: ({proposal_id, voter, in_favor}: {proposal_id: u32, voter: string, in_favor: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a has_voted transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Consulta si una cuenta ya votó en una propuesta.
   */
  has_voted: ({proposal_id, voter}: {proposal_id: u32, voter: string}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a add_member transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * El administrador agrega a un miembro al padrón.
   */
  add_member: ({member}: {member: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Se llama una sola vez para definir quién administra la votación.
   */
  initialize: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_proposal transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Consulta pública del estado y resultado de una propuesta.
   */
  get_proposal: ({proposal_id}: {proposal_id: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Proposal>>>

  /**
   * Construct and simulate a close_proposal transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * El administrador cierra la votación; ya no se aceptan más votos.
   */
  close_proposal: ({proposal_id}: {proposal_id: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a create_proposal transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * El administrador crea una propuesta nueva y recibe su id.
   * - `doc_hash`: sha256 del documento que respalda la propuesta.
   * - `signer` y `signature`: llave pública ed25519 y su firma (hecha fuera de la
   * cadena) sobre `doc_hash`. Si la firma no corresponde, la llamada falla.
   * - `deadline`: momento en que termina la votación (debe estar en el futuro).
   */
  create_proposal: ({title, doc_hash, signer, signature, deadline}: {title: string, doc_hash: Buffer, signer: Buffer, signature: Buffer, deadline: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u32>>>

  /**
   * Construct and simulate a verify_document transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Recalcula el sha256 de `data` y dice si coincide con el documento original.
   * Si cambia un solo byte, el hash es otro y devuelve false.
   */
  verify_document: ({proposal_id, data}: {proposal_id: u32, data: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<boolean>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAE9VbiBtaWVtYnJvIHZvdGEgYSBmYXZvciAodHJ1ZSkgbyBlbiBjb250cmEgKGZhbHNlKS4gU29sbyB1bmEgdmV6IHBvciBwcm9wdWVzdGEuAAAAAAR2b3RlAAAAAwAAAAAAAAALcHJvcG9zYWxfaWQAAAAABAAAAAAAAAAFdm90ZXIAAAAAAAATAAAAAAAAAAhpbl9mYXZvcgAAAAEAAAABAAAD6QAAA+0AAAAAAAAAAw==",
        "AAAAAAAAADFDb25zdWx0YSBzaSB1bmEgY3VlbnRhIHlhIHZvdMOzIGVuIHVuYSBwcm9wdWVzdGEuAAAAAAAACWhhc192b3RlZAAAAAAAAAIAAAAAAAAAC3Byb3Bvc2FsX2lkAAAAAAQAAAAAAAAABXZvdGVyAAAAAAAAEwAAAAEAAAAB",
        "AAAAAAAAADBFbCBhZG1pbmlzdHJhZG9yIGFncmVnYSBhIHVuIG1pZW1icm8gYWwgcGFkcsOzbi4AAAAKYWRkX21lbWJlcgAAAAAAAQAAAAAAAAAGbWVtYmVyAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAAAM=",
        "AAAAAAAAAEJTZSBsbGFtYSB1bmEgc29sYSB2ZXogcGFyYSBkZWZpbmlyIHF1acOpbiBhZG1pbmlzdHJhIGxhIHZvdGFjacOzbi4AAAAAAAppbml0aWFsaXplAAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAD6QAAA+0AAAAAAAAAAw==",
        "AAAAAAAAADpDb25zdWx0YSBww7pibGljYSBkZWwgZXN0YWRvIHkgcmVzdWx0YWRvIGRlIHVuYSBwcm9wdWVzdGEuAAAAAAAMZ2V0X3Byb3Bvc2FsAAAAAQAAAAAAAAALcHJvcG9zYWxfaWQAAAAABAAAAAEAAAPpAAAH0AAAAAhQcm9wb3NhbAAAAAM=",
        "AAAAAAAAAEJFbCBhZG1pbmlzdHJhZG9yIGNpZXJyYSBsYSB2b3RhY2nDs247IHlhIG5vIHNlIGFjZXB0YW4gbcOhcyB2b3Rvcy4AAAAAAA5jbG9zZV9wcm9wb3NhbAAAAAAAAQAAAAAAAAALcHJvcG9zYWxfaWQAAAAABAAAAAEAAAPpAAAD7QAAAAAAAAAD",
        "AAAAAAAAAVtFbCBhZG1pbmlzdHJhZG9yIGNyZWEgdW5hIHByb3B1ZXN0YSBudWV2YSB5IHJlY2liZSBzdSBpZC4KLSBgZG9jX2hhc2hgOiBzaGEyNTYgZGVsIGRvY3VtZW50byBxdWUgcmVzcGFsZGEgbGEgcHJvcHVlc3RhLgotIGBzaWduZXJgIHkgYHNpZ25hdHVyZWA6IGxsYXZlIHDDumJsaWNhIGVkMjU1MTkgeSBzdSBmaXJtYSAoaGVjaGEgZnVlcmEgZGUgbGEKY2FkZW5hKSBzb2JyZSBgZG9jX2hhc2hgLiBTaSBsYSBmaXJtYSBubyBjb3JyZXNwb25kZSwgbGEgbGxhbWFkYSBmYWxsYS4KLSBgZGVhZGxpbmVgOiBtb21lbnRvIGVuIHF1ZSB0ZXJtaW5hIGxhIHZvdGFjacOzbiAoZGViZSBlc3RhciBlbiBlbCBmdXR1cm8pLgAAAAAPY3JlYXRlX3Byb3Bvc2FsAAAAAAUAAAAAAAAABXRpdGxlAAAAAAAAEAAAAAAAAAAIZG9jX2hhc2gAAAPuAAAAIAAAAAAAAAAGc2lnbmVyAAAAAAPuAAAAIAAAAAAAAAAJc2lnbmF0dXJlAAAAAAAD7gAAAEAAAAAAAAAACGRlYWRsaW5lAAAABgAAAAEAAAPpAAAABAAAAAM=",
        "AAAAAAAAAIVSZWNhbGN1bGEgZWwgc2hhMjU2IGRlIGBkYXRhYCB5IGRpY2Ugc2kgY29pbmNpZGUgY29uIGVsIGRvY3VtZW50byBvcmlnaW5hbC4KU2kgY2FtYmlhIHVuIHNvbG8gYnl0ZSwgZWwgaGFzaCBlcyBvdHJvIHkgZGV2dWVsdmUgZmFsc2UuAAAAAAAAD3ZlcmlmeV9kb2N1bWVudAAAAAACAAAAAAAAAAtwcm9wb3NhbF9pZAAAAAAEAAAAAAAAAARkYXRhAAAADgAAAAEAAAPpAAAAAQAAAAM=",
        "AAAAAQAAACRVbmEgcHJvcHVlc3RhIHF1ZSBsYSBjb211bmlkYWQgdm90YS4AAAAAAAAACFByb3Bvc2FsAAAABwAAAEBNb21lbnRvIChzZWd1bmRvcywgaG9yYSBkZWwgbGVkZ2VyKSBlbiBxdWUgdGVybWluYSBsYSB2b3RhY2nDs24uAAAACGRlYWRsaW5lAAAABgAAAD9IdWVsbGEgc2hhMjU2IGRlbCBkb2N1bWVudG8gZGUgcmVzcGFsZG8gKHBvciBlamVtcGxvLCBlbCBhY3RhKS4AAAAACGRvY19oYXNoAAAD7gAAACAAAAAAAAAAAm5vAAAAAAAEAAAAAAAAAARvcGVuAAAAAQAAAFJMbGF2ZSBww7pibGljYSBlZDI1NTE5IGRlIHF1aWVuIGZpcm3DsyBlbCBkb2N1bWVudG8gKHBvciBlamVtcGxvLCBsYSBzZWNyZXRhcsOtYSkuAAAAAAAGc2lnbmVyAAAAAAPuAAAAIAAAAAAAAAAFdGl0bGUAAAAAAAAQAAAAAAAAAAN5ZXMAAAAABA==",
        "AAAABAAAAAAAAAAAAAAABUVycm9yAAAAAAAACAAAAAAAAAASQWxyZWFkeUluaXRpYWxpemVkAAAAAAABAAAAAAAAAA5Ob3RJbml0aWFsaXplZAAAAAAAAgAAAAAAAAAKTm90QU1lbWJlcgAAAAAAAwAAAAAAAAAQUHJvcG9zYWxOb3RGb3VuZAAAAAQAAAAAAAAADlByb3Bvc2FsQ2xvc2VkAAAAAAAFAAAAAAAAAAxBbHJlYWR5Vm90ZWQAAAAGAAAAAAAAAA9Qcm9wb3NhbEV4cGlyZWQAAAAABwAAAAAAAAAPSW52YWxpZERlYWRsaW5lAAAAAAg=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABQAAAAAAAAAAAAAABUFkbWluAAAAAAAAAAAAAAAAAAAFQ291bnQAAAAAAAABAAAAAAAAAAZNZW1iZXIAAAAAAAEAAAATAAAAAQAAAAAAAAAIUHJvcG9zYWwAAAABAAAABAAAAAEAAAAAAAAABVZvdGVkAAAAAAAAAgAAAAQAAAAT" ]),
      options
    )
  }
  public readonly fromJSON = {
    vote: this.txFromJSON<Result<void>>,
        has_voted: this.txFromJSON<boolean>,
        add_member: this.txFromJSON<Result<void>>,
        initialize: this.txFromJSON<Result<void>>,
        get_proposal: this.txFromJSON<Result<Proposal>>,
        close_proposal: this.txFromJSON<Result<void>>,
        create_proposal: this.txFromJSON<Result<u32>>,
        verify_document: this.txFromJSON<Result<boolean>>
  }
}