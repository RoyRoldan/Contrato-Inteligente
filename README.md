# Votación comunitaria en Stellar

Primer contrato inteligente en Stellar (Soroban, Rust) 

> Proyecto individual de práctica. 

## El problema

En Costa Rica, muchas decisiones de barrio (asociaciones de desarrollo, ASADAs,
condominios, comités de la escuela) se toman levantando la mano, con papelitos
o en un grupo de WhatsApp. Esto genera problemas simples pero reales:

- Alguien puede votar dos veces o votar sin ser parte del grupo.
- Nadie puede verificar el resultado después.
- Si hay una disputa, no hay un registro confiable.
- El acta que respalda la decisión se puede cambiar después sin que nadie lo note.
- Las votaciones no tienen una fecha límite clara.

## La solución

Un contrato inteligente que guarda el padrón de miembros, las propuestas y los
votos en la blockchain de Stellar. Las reglas se cumplen solas:

1. Solo el **administrador** registra miembros y crea o cierra propuestas.
2. Solo los **miembros** del padrón pueden votar.
3. Cada miembro vota **una sola vez** por propuesta.
4. Cada propuesta tiene una **fecha límite**: pasado el plazo, ya no se aceptan votos.
5. Cuando la propuesta se cierra, tampoco se aceptan votos.
6. Cualquier persona puede **consultar el resultado**.
7. Cada propuesta guarda la **huella sha256** de su documento (por ejemplo, el acta).
   El documento no se sube a la cadena; solo su huella. Cualquiera puede **verificar**
   que un documento es el original: si cambia un solo byte, el hash es distinto.
8. Al crear la propuesta, el contrato comprueba con **`ed25519_verify`** que una llave
   (por ejemplo, la secretaría) **firmó** esa huella. La firma se hace fuera de la cadena;
   si no corresponde, la llamada falla y no se guarda nada.

## Por qué elegí este problema

Es pequeño, se entiende en una frase y no necesita tokens ni pagos, así que me
permite enfocarme en lo visto en clase: tipos (`Bytes`, `BytesN<32>`, `BytesN<64>`),
`sha256`, `ed25519_verify`, errores con código, eventos, vigencia con TTL, tests que
mueven el tiempo, módulos ordenados y bindings de TypeScript para llamarlo desde una app.

## Estructura del proyecto

```
src/
├── lib.rs      // #[contract] y funciones públicas
├── storage.rs  // DataKey, get / set y extend_ttl
├── errors.rs   // #[contracterror] con código
├── events.rs   // eventos: member, proposal, vote, closed
├── types.rs    // Proposal
└── test.rs     // pruebas (incluye tests que mueven el tiempo)
scripts/deploy_testnet.ps1   // despliegue, prueba en testnet y bindings
frontend-app/                // app en TypeScript que usa los bindings y Stellar Wallets Kit
frontend/index.html          // demo visual simple (simulación, sin blockchain)
```

## Funciones del contrato

| Función | Quién la usa | Qué hace |
|---|---|---|
| `initialize(admin)` | Una vez | Define al administrador |
| `add_member(member)` | Admin | Agrega una cuenta al padrón |
| `create_proposal(title, doc_hash, signer, signature, deadline)` | Admin | Verifica la firma del documento, crea la propuesta y devuelve su id |
| `vote(proposal_id, voter, in_favor)` | Miembro | Vota a favor (`true`) o en contra (`false`) |
| `close_proposal(proposal_id)` | Admin | Cierra la votación |
| `get_proposal(proposal_id)` | Cualquiera | Devuelve título, huella, firmante, plazo, votos y estado |
| `verify_document(proposal_id, data)` | Cualquiera | Recalcula el sha256 de `data` y devuelve `true` si coincide |
| `has_voted(proposal_id, voter)` | Cualquiera | Indica si una cuenta ya votó |

### Errores

| Código | Error | Cuándo |
|---|---|---|
| 1 | `AlreadyInitialized` | Se inicializa dos veces |
| 2 | `NotInitialized` | Se usa antes de inicializar |
| 3 | `NotAMember` | Vota alguien fuera del padrón |
| 4 | `ProposalNotFound` | La propuesta no existe |
| 5 | `ProposalClosed` | La propuesta está cerrada |
| 6 | `AlreadyVoted` | La cuenta ya votó |
| 7 | `ProposalExpired` | Ya pasó la fecha límite |
| 8 | `InvalidDeadline` | La fecha límite no está en el futuro |

### Eventos

`member` (miembro agregado), `proposal` (propuesta creada), `vote` (voto emitido) y
`closed` (propuesta cerrada).

### Vigencia (TTL)

Los datos de Soroban expiran si nadie los renueva. `storage.rs` extiende la vigencia
(unos 30 días) cada vez que se lee o escribe un dato, así que las propuestas con
actividad se mantienen vivas.

## Cómo correrlo

Requisitos: [Rust](https://www.rust-lang.org/tools/install), el target
`wasm32v1-none` y la [Stellar CLI](https://developers.stellar.org/docs/tools/cli).
En Windows también hacen falta las C++ Build Tools de Visual Studio.

```bash
# Correr las pruebas
cargo test

# Compilar el contrato
stellar contract build
```

Nota: si `cargo test` falla con un error de `ed25519-dalek`, corre
`cargo update -p ed25519-dalek@3.0.0 --precise 2.2.0` y vuelve a probar.

## Pruebas

Las pruebas están en `src/test.rs` (15 en total) y cubren:

- Una votación completa con tres miembros (2 a favor, 1 en contra).
- Que no se pueda votar dos veces, ni fuera del padrón, ni en una propuesta cerrada o inexistente.
- Que el contrato no se pueda inicializar dos veces ni usarse sin inicializar.
- Que el documento original se verifique y uno alterado no.
- Que una firma ed25519 inválida, o la firma de otro documento, impida crear la propuesta.
- **Tests que mueven el tiempo:** una fecha límite en el pasado es inválida, no se vota
  después del plazo, y los datos siguen vivos pasados 40 días gracias al TTL.

## Despliegue en testnet

Con la Stellar CLI instalada, desde la raíz del proyecto en PowerShell:

```
.\scripts\deploy_testnet.ps1
```

El script compila el contrato, crea dos cuentas de prueba (`admin` y `bob`) con fondos
falsos, despliega el contrato, crea una propuesta firmada, hace votar a Bob, verifica
el documento y genera los bindings de TypeScript en `packages/votacion`.

**ID del contrato en testnet:** `PEGAR_AQUI_EL_ID`

## Llamarlo desde una app (TypeScript)

Los bindings se generan con:

```
stellar contract bindings typescript --network testnet --contract-id votacion --output-dir packages/votacion
```

La app de `frontend-app/` los usa junto con [Stellar Wallets Kit](https://stellarwalletskit.dev/),
que permite conectar varias wallets (Freighter, xBull, Lobstr, Albedo y otras) desde un
mismo selector. La wallet debe estar en red testnet. Las lecturas solo simulan la llamada; las escrituras
(`vote`) se firman con la wallet y se envían.

```
cd frontend-app
npm install
npx jsr add @creit-tech/stellar-wallets-kit
npm run dev
```

La cuenta que vota debe estar en el padrón (`add_member` hecho por el administrador).

## Demo visual simple

`frontend/index.html` se abre con doble clic en el navegador. Es una **simulación**
de las reglas (no se conecta a la blockchain, y no incluye la firma ni la fecha límite):
sirve para ver cómo funciona el flujo de votación y la verificación de documentos.

## Posibles mejoras

- Que los miembros puedan proponer, no solo el administrador.
- Voto ponderado (por ejemplo, un voto por vivienda).
- Pantalla de administrador en la app para crear propuestas con firma.
