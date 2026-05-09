# Testing Guidelines

Этот документ задает единый стиль тестов для `exchange-engine`.

## Где писать тесты

1. Domain tests:
`crates/domain/tests/*.rs`  
Пример: `crates/domain/tests/trading_tests.rs`

2. Application tests:
`crates/application/tests/*.rs`

3. Сквозные интеграционные тесты (несколько crate/service):
корневой `tests/`.

## Базовые правила

1. Проверяем инварианты, а не детали реализации.
2. Один тест = один бизнес-сценарий.
3. Названия тестов в формате behavior:
`partially_fills_and_leaves_resting_order`.
4. Не используем `f64/f32` для денег и объема.
5. Для повторяющихся данных используем локальные helper-функции (`buy(...)`, `sell(...)`).

## Что обязательно тестировать для matching engine

1. `price priority` (лучшая цена исполняется первой).
2. `time priority` внутри одного ценового уровня (FIFO).
3. Частичное исполнение и остаток в стакане.
4. Полное исполнение без остатка.
5. Невалидные заявки (`qty=0`, `price=0`, пустые `id`/`user_id`).
6. Рынок не совпадает с market engine.

## Шаблон unit-теста

```rust
#[test]
fn scenario_name() {
    // arrange
    // act
    // assert
}
```

## Запуск тестов

- Все workspace тесты:
`cargo test --workspace`

- Только domain:
`cargo test -p domain`

- Один тест по имени:
`cargo test -p domain partially_fills_and_leaves_resting_order`

## Минимальный quality gate перед merge

1. `cargo fmt --check`
2. `cargo clippy --workspace -- -D warnings`
3. `cargo test --workspace`
