# clickcheck

**clickcheck** — инструмент для анализа ClickHouse: 
Помогает *DBA* быстро обнаруживать и устранять проблемы.

На текущий момент ищет тяжелые запросы и ошибки.
В потенциале будет также искать неэффективные запросы, аномалии, пики нагрузки, рост хранилища и другие проблемы.

---

## 🚀 Ключевые возможности

- Анализ `query_log`: группировка запросов по fingerprint
- Многоформатный вывод: текст, JSON, YAML
- Управление профилями подключения (контексты)
- Сбор данных со всех нод кластера (или указанных) с последующей агрегацией на стороне `clickcheck`

## 🛠️ Установка

```bash
cargo install clickcheck
```

## ⚙️ Использование

Используйте `--help` для подробной справки по каждой команде:

```bash
clickcheck --help
clickcheck queries --help
clickcheck errors --help
clickcheck context --help
```

Пример

```bash
clickcheck context set profile ch-hello -U 'https://my-ch-hello-node-1:8443' -U 'https://my-ch-hello-node-2:8443' -u 'hello_user' -i
# Вводим ClickHouse hello_user password:
clickcheck context set profile ch-bye -U 'https://my-ch-bye-node-1:8443' -u 'bye_user' -i
# Вводим ClickHouse bye_user password:

# Выставляем context по умолчанию
clickcheck context set current ch-hello

# Смотрим топ 5 тяжелых запросов на кластере ch-hello
clickcheck queries --last 1hour
# Смотрим топ 5 ошибок на ch-hello
clickcheck errors

# Смотрим топ 5 тяжелых запросов на кластере ch-bye
clickcheck queries --last 1hour --context ch-bye
```
