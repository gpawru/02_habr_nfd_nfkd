# NF(K)D нормализация строк Unicode

примеры для статьи на Хабре: _вставить ссылку_

### структура репозитория:

- [**benches**](benches) - бенчмарки нормализации
- [**tests**](tests) - тесты нормализации
- **data** - "запечённые" данные декомпозиции
- [**decomposing**](decomposing) - нормализация строк
- **test_data** - данные для тестирования и бенчмарков

### подготовка данных:

- парсинг UCD: https://github.com/gpawru/unicode_data
- запекание данных: https://github.com/gpawru/unicode_bakery

### запуск тестов и бенчмарков:

```
make test
```

```
make bench
```
*(результат - в виде CSV)*
