# Simulador Rust

Simuladores PIX em Rust, equivalentes ao gateboxgo/simulators em Go.

## Estrutura

| Simulador | Porta | Função |
|-----------|-------|--------|
| seventrust | 7010 | Mock do gateway SevenTrust |
| sulcred | 7020 | Mock do gateway Sulcred |
| client-simulator | 7070 | Load tester e simulação PIX |

## Uso local (cargo)

```bash
# Testes
cargo test

# Todos de uma vez
./scripts/run-all.sh

# Ou individualmente
cargo run -p seventrust
PORT=7020 cargo run -p sulcred
GATEWAY_URL=http://localhost:8080 cargo run -p client-simulator
```

## Docker

```bash
# Build das imagens
./scripts/build-docker.sh

# Ou manualmente
docker build --build-arg PACKAGE=seventrust -t seventrust-rust .
docker build --build-arg PACKAGE=sulcred -t sulcred-rust .
docker build --build-arg PACKAGE=client-simulator -t client-simulator-rust .
```

## Workspace `money/` (Docker + apps)

O único Compose do projeto está em **`money/docker-compose.yml`**. A partir desta pasta (`gatebox/simulador_rust`):

```bash
cd ../..
./runinfra.sh
./runapp.sh start gatebox
```

## Variáveis

| Variável | Descrição |
|----------|-----------|
| PORT | Porta (default 7010, 7020, 7070) |
| GATEBOXGO_WEBHOOK_URL | URL da API para webhooks (seventrust/sulcred) |
| GATEWAY_URL | URL da API principal (client-simulator) |
| **MESSAGING_BACKEND** | `pulsar` (default) ou `rabbitmq` (client-simulator) |
| PULSAR_URL | URL do Pulsar (ex.: `pulsar://localhost:6650`) |
| RABBITMQ_URL | URL do RabbitMQ (client-simulator) |
| use_api | `true` = POST direto na API; `false` = publica na fila (Pulsar ou RabbitMQ) |
| **SEVENTRUST_USE_TLS** | `false` (default) = HTTP; `true` = HTTPS com cert PEM |
| SEVENTRUST_TLS_CERT_PATH | Caminho do certificado PEM (seventrust) |
| SEVENTRUST_TLS_KEY_PATH | Caminho da chave privada PEM (seventrust) |
| **SULCRED_USE_TLS** | `false` (default) = HTTP; `true` = HTTPS com cert PEM |
| SULCRED_TLS_CERT_PATH | Caminho do certificado PEM (sulcred) |
| SULCRED_TLS_KEY_PATH | Caminho da chave privada PEM (sulcred) |

**Nota:** client-simulator com Pulsar exige `protoc` (ex.: `apt-get install protobuf-compiler`).
