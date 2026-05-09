# Руководство по разработке: Rust + DDD + Hexagonal Architecture

## 1) Цель документа

Этот документ фиксирует, как писать код биржи так, чтобы:
- доменная логика была явной и проверяемой;
- инфраструктурные детали не протекали в бизнес-правила;
- система масштабировалась по командам и сервисам без деградации качества.

## 2) Архитектурные принципы

1. `Domain first`  
Бизнес-инварианты живут в домене, а не в контроллерах/хэндлерах.

2. `Dependency rule`  
Внутренние слои не зависят от внешних.  
`domain` не знает про БД, Kafka, HTTP, Redis.

3. `Explicit boundaries`  
Каждый bounded context имеет собственный язык, типы и use cases.

4. `No anemic domain`  
Сущности содержат поведение и инварианты, а не только поля.

## 3) Bounded Contexts (для текущей биржи)

- `trading`: ордера, matching, lifecycle.
- `risk`: margin, leverage checks, liquidation triggers.
- `ledger`: double-entry postings, account movements.
- `wallet`: депозиты/выводы, on-chain state machine.
- `market-data`: order book projections, streams.
- `identity`: users, API keys, scopes.
- `reference-data`: markets, tick/lot size, статус пар.

Правило:
- пересечение контекстов только через контракты (events/API), не через shared mutable model.

## 4) Рекомендуемая структура репозитория (workspace)

```text
exchange-engine/
  crates/
    domain/
      trading/
      risk/
      ledger/
      wallet/
      reference_data/
    application/
      trading_app/
      risk_app/
      ledger_app/
      wallet_app/
    ports/
      persistence/
      messaging/
      time/
      idempotency/
    adapters/
      http/
      grpc/
      postgres/
      kafka/
      redis/
      blockchain/
    platform/
      observability/
      config/
      error/
    binaries/
      trading_api/
      market_data_api/
      wallet_worker/
  docs/
  tests/
```

Минимальная альтернатива для старта:
- `crates/domain`
- `crates/application`
- `crates/adapters`
- `crates/binaries`

## 5) Hexagonal слои и правила зависимостей

## Domain Layer

Содержит:
- сущности (`Order`, `Position`, `LedgerAccount`);
- value objects (`Price`, `Qty`, `Money`, `Leverage`);
- доменные сервисы;
- доменные события;
- инварианты и ошибки домена.

Не содержит:
- `sqlx`, `rdkafka`, `axum`, `serde_json::Value`, SQL, HTTP DTO.

## Application Layer (Use Cases)

Содержит:
- orchestration сценариев (`PlaceOrder`, `CancelOrder`, `ConfirmDeposit`);
- транзакционные границы;
- вызовы портов;
- policy-level idempotency.

Не содержит:
- бизнес-инвариантов, которые должны жить в domain entities.

## Ports (Interfaces)

Примеры:
- `OrderRepository`
- `LedgerRepository`
- `EventPublisher`
- `Clock`
- `IdempotencyStore`

Правило:
- порты объявляются внутри внутреннего слоя (обычно `application`), реализации в `adapters`.

## Adapters

Содержит:
- входящие адаптеры (`HTTP/gRPC`, consumers, schedulers);
- исходящие адаптеры (`Postgres`, `Kafka`, `Redis`, blockchain RPC).

Правило:
- адаптеры только маппят протокол <-> use case, без доменной логики.

## 6) Правила моделирования домена

1. `Невалидное состояние недостижимо`
- конструкторы `new(...) -> Result<T, DomainError>`
- приватные поля + smart constructors.

2. `Value Objects вместо примитивов`
- не `f64 price`, а `Price(Decimal)`
- не `String market`, а `MarketId`.

3. `Явная арифметика денег`
- использовать `rust_decimal`, запрет `float` для денег/объемов.

4. `Детерминизм`
- matching/risk функции детерминированы и тестируются на replay.

5. `Доменные события как факт`
- `OrderAccepted`, `TradeExecuted`, `MarginCallTriggered`.

## 7) Ошибки и Result-стратегия

- В domain: строго типизированные ошибки (`thiserror`).
- В application: композиция доменных и инфраструктурных ошибок.
- На границе API: явное маппирование в transport-коды (HTTP/gRPC), без утечки внутренних деталей.

Рекомендация:
- `anyhow` использовать только в executable composition root и tooling-коде.

## 8) Конфигурация и Composition Root

- Каждый бинарь имеет явный `main` как composition root:
  - загрузка config;
  - создание адаптеров;
  - wiring use cases;
  - запуск сервера/воркера.

Запрет:
- глобальные singleton с неявным состоянием.

## 9) Конкурентность и производительность (Rust)

1. Async I/O:
- `tokio` для network/IO-bound задач.

2. CPU-bound:
- выносить тяжелые вычисления из async executors (dedicated pools/threads).

3. Backpressure:
- bounded channels и лимиты очередей.

4. Lock discipline:
- минимизировать shared mutable state;
- предпочитать single-writer ownership per market shard.

5. Allocation awareness:
- избегать лишних копий в hot path;
- использовать borrow/refs, где уместно.

## 10) Тестовая стратегия

1. Unit tests (domain):
- инварианты сущностей и value objects.

2. Property tests:
- matching/ledger/risk инварианты (`proptest`).

3. Application tests:
- сценарии use cases с mock/fake портами.

4. Integration tests:
- Postgres/Kafka/Redis adapters.

5. Contract tests:
- event schema compatibility и API contracts.

Минимальный quality gate в CI:
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`

## 11) Код-стайл и практики Rust

- Модули маленькие, связные, с явными границами.
- Публичный API crate минимальный (`pub(crate)` по умолчанию).
- Новые типы (`newtype`) для ключевых идентификаторов.
- `Arc<dyn Trait>` только там, где нужна runtime-полиморфность.
- Избегать premature generic complexity; сначала простая читаемая модель.
- Документировать инварианты рядом с кодом, а не только в wiki.

## 12) Антипаттерны (чего избегать)

- Толстые handlers/controllers с бизнес-логикой.
- Общая shared-модель между контекстами.
- Прямой доступ из API слоя в SQL/Kafka без use case.
- Денежные расчеты через `f32/f64`.
- "Удобные" bypass-инвариантов ради быстрого фичефлага.

## 13) Definition of Done для новой фичи

Фича считается завершенной, если:
- реализована в терминах domain + use case;
- адаптеры не содержат бизнес-правил;
- покрыта unit/integration тестами по риску изменения;
- добавлены метрики/логи/трейсы для эксплуатации;
- обновлены ADR/доки при изменении архитектурных решений.

## 14) План внедрения по шагам

1. Зафиксировать целевую workspace-структуру и naming conventions.
2. Вынести текущие доменные типы в `crates/domain`.
3. Добавить `application` use cases поверх домена.
4. Обернуть текущие API/хранилища в порты и адаптеры.
5. Добавить тестовые пирамиды и CI quality gates.
6. Проводить каждую новую фичу только через DDD+Hexagonal шаблон.

