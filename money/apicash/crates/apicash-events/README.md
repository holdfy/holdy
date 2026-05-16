# apicash-events

Mensageria assíncrona para APICash usando **Apache Pulsar** (crate [`pulsar`](https://crates.io/crates/pulsar), upstream [streamnative/pulsar-rs](https://github.com/streamnative/pulsar-rs)).

## Conteúdo

- **Modelos** — enum [`ApicashEvent`](src/models/events.rs) com payload tipado (pedido, pagamento, score, custódia, liberação, disputa, etc.).
- **Producer** — [`EventProducer`](src/producer/event_producer.rs) com métodos `publish_*` por tipo.
- **Consumers** — [`run_custody_consumer`](src/consumer/custody_consumer.rs), [`run_antifraude_consumer`](src/consumer/antifraude_consumer.rs), [`run_release_consumer`](src/consumer/release_consumer.rs); cada um usa uma *subscription* distinta no mesmo tópico.
- **Config** — [`PulsarConfig`](src/config/pulsar_config.rs): URL do broker, tenant, namespace e nome do tópico.

## Variáveis de ambiente

| Variável | Descrição |
|----------|-----------|
| `APICASH_PULSAR__SERVICE_URL` | Ex.: `pulsar://127.0.0.1:6650` |
| `APICASH_PULSAR__TENANT` | Default `public` |
| `APICASH_PULSAR__NAMESPACE` | Default `default` |
| `APICASH_PULSAR__TOPIC_NAME` | Default `apicash-events` |

Tópico completo: `persistent://{tenant}/{namespace}/{topic_name}`.

## Integração

Implemente os traits `CustodyLockPort`, `AntifraudeEventPort` e `ReleaseEventPort` no serviço que possui `CustodyService` / antifraude / release, injete em `Arc<dyn ...>` e execute os `run_*_consumer` em `tokio::spawn`.

## Testes

```bash
cargo test -p apicash-events
```
