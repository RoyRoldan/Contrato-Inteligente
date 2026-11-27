import { Buffer } from "buffer";
(window as any).Buffer = Buffer;

import { StellarWalletsKit } from "@jsr/creit-tech__stellar-wallets-kit/sdk";
import { defaultModules } from "@jsr/creit-tech__stellar-wallets-kit/modules/utils";
import { Client, networks } from "votacion"; // bindings generados con stellar contract bindings typescript

const $ = (id: string) => document.getElementById(id) as HTMLElement;
const say = (t: string) => ($("status").textContent = t);

// Mismos códigos que src/errors.rs
const ERRORES: Record<string, string> = {
  "1": "El contrato ya estaba inicializado.",
  "2": "El contrato todavía no se ha inicializado.",
  "3": "Tu cuenta no está en el padrón, así que no puede votar.",
  "4": "Esa propuesta no existe.",
  "5": "La propuesta está cerrada.",
  "6": "Esta cuenta ya votó en esta propuesta.",
  "7": "El plazo de la votación ya terminó.",
  "8": "La fecha límite debe estar en el futuro.",
};
const explicar = (e: unknown) => {
  const m = String((e as Error)?.message ?? e).match(/#(\d+)/);
  return m && ERRORES[m[1]] ? ERRORES[m[1]] : "Error: " + String((e as Error)?.message ?? e);
};

// Stellar Wallets Kit: un solo selector para Freighter, xBull, Lobstr, Albedo y más
StellarWalletsKit.init({
  modules: defaultModules(),
  network: networks.testnet.networkPassphrase as any,
});

let address = "";
let client: Client | null = null;

// Los bindings usan los nombres del contrato; por si salen en camelCase, probamos ambos.
const camel = (s: string) => s.replace(/_([a-z])/g, (_, c) => c.toUpperCase());
const call = (name: string, args: object): Promise<any> => {
  const c = client as any;
  return (c[name] ?? c[camel(name)]).call(c, args);
};
// Las funciones que devuelven Result traen el valor dentro de .unwrap()
const valor = (tx: any) => (tx.result?.unwrap ? tx.result.unwrap() : tx.result);

$("connect").onclick = async () => {
  try {
    // Abre el selector donde la persona elige su wallet
    const { address: cuenta } = await StellarWalletsKit.authModal();
    address = cuenta;
    client = new Client({
      ...networks.testnet,
      rpcUrl: "https://soroban-testnet.stellar.org",
      publicKey: address,
      // de la wallet elegida
      signTransaction: (xdr: string) =>
        StellarWalletsKit.signTransaction(xdr, {
          networkPassphrase: networks.testnet.networkPassphrase,
          address,
        }),
    });
    $("account").textContent = " " + address.slice(0, 6) + "…" + address.slice(-4);
    say("Wallet conectada.");
  } catch (e) { say(explicar(e)); }
};

const idActual = () => Number(($("pid") as HTMLInputElement).value);

$("load").onclick = async () => {
  if (!client) return say("Conecta tu wallet primero.");
  try {
    // Lectura: con simular alcanza, no se firma nada
    const p: any = valor(await call("get_proposal", { proposal_id: idActual() }));
    $("proposal").textContent =
      `${p.title}\nA favor: ${p.yes}   En contra: ${p.no}\n` +
      `Estado: ${p.open ? "abierta" : "cerrada"}\nPlazo (unix): ${p.deadline}\n` +
      `sha256: ${Buffer.from(p.doc_hash).toString("hex")}\n` +
      `firmó: ${Buffer.from(p.signer).toString("hex")}`;
    say("Propuesta cargada.");
  } catch (e) { say(explicar(e)); }
};

const votar = (a_favor: boolean) => async () => {
  if (!client) return say("Conecta tu wallet primero.");
  try {
    // Escritura: firmar y enviar
    const tx = await call("vote", { proposal_id: idActual(), voter: address, in_favor: a_favor });
    await tx.signAndSend();
    say("Voto registrado en testnet.");
    ($("load") as HTMLButtonElement).click();
  } catch (e) { say(explicar(e)); }
};
$("yes").onclick = votar(true);
$("no").onclick = votar(false);

$("verify").onclick = async () => {
  if (!client) return say("Conecta tu wallet primero.");
  try {
    const texto = ($("doc") as HTMLTextAreaElement).value;
    const ok = valor(await call("verify_document", { proposal_id: idActual(), data: Buffer.from(texto) }));
    say(ok ? "El documento coincide con el original." : "El documento NO coincide: fue alterado o no es el original.");
  } catch (e) { say(explicar(e)); }
};
