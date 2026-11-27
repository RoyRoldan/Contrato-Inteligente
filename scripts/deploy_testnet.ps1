# Despliega el contrato en testnet y hace una votacion de prueba.
# Uso (desde la raiz del proyecto, en PowerShell):  .\scripts\deploy_testnet.ps1
# Requiere: Rust, el target wasm32v1-none y la Stellar CLI instalada.

$ErrorActionPreference = "Stop"

# Se detiene en el primer comando que falle, para ver cual fue el error real.
function Check($paso) {
    if ($LASTEXITCODE -ne 0) { throw "FALLO el paso: $paso (codigo $LASTEXITCODE). Revisa el mensaje de arriba." }
}

Write-Host "0) Comprobando la Stellar CLI..."
stellar --version
Check "stellar --version (instala la CLI con: winget install --id Stellar.StellarCLI)"

Write-Host "1) Compilando el contrato..."
stellar contract build
Check "stellar contract build"

Write-Host "2) Creando cuentas de prueba con fondos falsos (si ya existen, se reutilizan)..."
$existentes = stellar keys ls
if ($existentes -notcontains "admin") { stellar keys generate admin --network testnet --fund; Check "crear cuenta admin" }
if ($existentes -notcontains "bob")   { stellar keys generate bob   --network testnet --fund; Check "crear cuenta bob" }
$ADMIN = stellar keys address admin
$BOB   = stellar keys address bob
Write-Host "admin: $ADMIN"
Write-Host "bob:   $BOB"

Write-Host "3) Desplegando en testnet..."
$WASM = "target/wasm32v1-none/release/votacion_comunitaria.wasm"
if (-not (Test-Path $WASM)) { throw "No existe $WASM. Revisa el paso de compilacion." }
$ID = stellar contract deploy --wasm $WASM --source-account admin --network testnet --alias votacion
Check "stellar contract deploy"
Write-Host "ID DEL CONTRATO: $ID"

# sha256 de "Acta: se aprueba reparar la tuberia"
$HASH = "bdfe77aa003ab78239fc5c5240697d32fc665dc398a7781707a3b565ddaacca2"
# esos mismos bytes del texto, en hexadecimal
$DOC  = "416374613a20736520617072756562612072657061726172206c612074756265726961"
# llave publica ed25519 de quien firmo el acta, y su firma sobre el hash (hecha fuera de la cadena)
$SIGNER    = "ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c"
$SIGNATURE = "590e42e26bf5cacac6d3f336bf449af05b95952e45cdf966bdd53cf6476eff94a797aea527f6f7743d03b23a4b6265435b4c3f703ba4d3ad7ad17b059c9baa0e"
# la votacion dura 30 dias desde ahora (el ledger usa segundos Unix)
$DEADLINE = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds() + 2592000

Write-Host "4) Inicializando y creando la votacion..."
stellar contract invoke --id $ID --source-account admin --network testnet -- initialize --admin $ADMIN
Check "initialize"
stellar contract invoke --id $ID --source-account admin --network testnet -- add_member --member $BOB
Check "add_member"
stellar contract invoke --id $ID --source-account admin --network testnet -- create_proposal --title "Reparar la tuberia" --doc_hash $HASH --signer $SIGNER --signature $SIGNATURE --deadline $DEADLINE
Check "create_proposal"

Write-Host "5) Bob vota a favor..."
stellar contract invoke --id $ID --source-account bob --network testnet -- vote --proposal_id 1 --voter $BOB --in_favor true
Check "vote"

Write-Host "6) Resultado y verificacion del documento..."
stellar contract invoke --id $ID --source-account admin --network testnet -- get_proposal --proposal_id 1
Check "get_proposal"
stellar contract invoke --id $ID --source-account admin --network testnet -- verify_document --proposal_id 1 --data $DOC
Check "verify_document"

Write-Host "7) Generando los bindings de TypeScript..."
stellar contract bindings typescript --network testnet --contract-id $ID --output-dir packages/votacion --overwrite
Check "bindings typescript"
Push-Location packages/votacion
npm install
npm run build
Pop-Location
Set-Content -Path frontend-app/.env -Value "VITE_CONTRACT_ID=$ID"

Write-Host "Listo. Guarda el ID del contrato en el README: $ID"
