<p align="center">
  <img src=".github/banner.png" alt="Eclipse Claw" width="700" />
</p>

<h3 align="center">
  Локальный web extraction toolkit для AI-агентов<br/>
  <sub>CLI, MCP и REST. Public-only egress, изолированные workers и проверяемые fixtures.</sub>
</h3>

<p align="center">
  <a href="https://github.com/PavelHopson/Eclipse-Claw/stargazers"><img src="https://img.shields.io/github/stars/PavelHopson/Eclipse-Claw?style=for-the-badge&logo=github&logoColor=white&label=Stars&color=181717" alt="Stars" /></a>
  <a href="https://github.com/PavelHopson/Eclipse-Claw/releases"><img src="https://img.shields.io/github/v/release/PavelHopson/Eclipse-Claw?style=for-the-badge&logo=rust&logoColor=white&label=Version&color=B7410E" alt="Version" /></a>
  <a href="https://github.com/PavelHopson/Eclipse-Claw/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-AGPL--3.0-10B981?style=for-the-badge" alt="License" /></a>
  <a href="https://www.npmjs.com/package/create-eclipse-claw"><img src="https://img.shields.io/npm/dt/create-eclipse-claw?style=for-the-badge&logo=npm&logoColor=white&label=Installs&color=CB3837" alt="npm installs" /></a>
</p>

---

## Проблема

Ваш AI-агент вызывает `fetch()` и получает **403 Forbidden**. Или 142 КБ сырого HTML, который сжигает токены. **Eclipse Claw решает обе проблемы.**

Извлекает структурированный контент из публичных HTTP(S)-страниц с помощью TLS-отпечатков уровня Chrome — обычно без headless-браузера, Selenium или Puppeteer. Вывод для LLM убирает навигационный шум и сохраняет полезные метаданные, ссылки и изображения. Точный результат зависит от конкретной страницы и проверяется воспроизводимыми fixtures.

```
              Сырой HTML                          Eclipse Claw
┌──────────────────────────────────┐    ┌──────────────────────────────────┐
│ <div class="ad-wrapper">         │    │ # Прорыв в AI                    │
│ <nav class="global-nav">         │    │                                  │
│ <script>window.__NEXT_DATA__     │    │ Исследователи достигли 94%       │
│ ={...8KB JSON...}</script>       │    │ точности на бенчмарках           │
│ <div class="social-share">       │    │ кросс-доменных рассуждений.      │
│ <footer class="site-footer">     │    │                                  │
│ <!-- 142 847 символов -->        │    │ ## Ключевые выводы               │
│                                  │    │ - Структура для LLM/RAG         │
│         Сырой документ           │    │         Очищенный вывод          │
└──────────────────────────────────┘    └──────────────────────────────────┘
```

---

## Быстрый старт (30 секунд)

### Для AI-агентов (Claude, Cursor, Windsurf, VS Code)

```bash
npx create-eclipse-claw
```

Автоматически определяет ваши AI-инструменты, скачивает MCP-сервер и настраивает всё. Одна команда.

### Проверенные готовые бинарники

Скачайте архив для macOS (arm64, x86_64) или Linux (x86_64, aarch64) из
[GitHub Releases](https://github.com/PavelHopson/Eclipse-Claw/releases). Перед запуском
сверьте SHA-256 архива с опубликованным `SHA256SUMS`. Публичный Homebrew tap пока не выпущен.

### Cargo (из исходников)

```bash
cargo install --locked --git https://github.com/PavelHopson/Eclipse-Claw.git --tag v0.4.1 eclipse-claw-cli
cargo install --locked --git https://github.com/PavelHopson/Eclipse-Claw.git --tag v0.4.1 eclipse-claw-mcp
```

### Docker

```bash
docker run --rm ghcr.io/pavelhopson/eclipse-claw https://example.com
```

### Docker Compose (с Ollama для LLM-функций)

```bash
cp env.example .env
# Замените три token placeholder и задайте проверенный OLLAMA_IMAGE@sha256.
docker compose up -d
```

---

## Сравнение с аналогами

| | Eclipse Claw | Firecrawl | Trafilatura | [Apify Skills](https://github.com/apify/agent-skills) |
|---|:---:|:---:|:---:|:---:|
| **Воспроизводимый fixture gate** | **Да** | См. проект | См. проект | См. платформу |
| **LLM-ориентированный вывод** | **Да** | Да | Текст | Структурированный JSON |
| **Локальное выполнение** | **Да** | Опционально | Да | Нет |
| **TLS-отпечатки** | Да | Нет | Нет | Не нужно (API) |
| **Self-hosted** | **Да** | Нет | Да | Нет (облако) |
| **REST API сервер** | **Да** | Да | Нет | Да (Apify API) |
| **Design token extraction (CDP)** | **Да** | Нет | Нет | Нет |
| **MCP-сервер** | **Да** | Нет | Нет | Нет |
| **DeepSeek поддержка** | **Да** | Нет | Нет | Нет |
| **JSONL-вывод** | **Да** | Нет | Нет | JSON |
| **Без браузера** | Да | Нет | Да | Облако |
| **Платформ-парсеры** | Любой URL | Любой URL | Любой URL | **55+ (Twitter, TikTok, YouTube...)** |
| **Стоимость** | Бесплатно | $$$$ | Бесплатно | Free tier, затем pay-per-use |

> **Когда что использовать:** Eclipse Claw — для быстрого извлечения контента из любого URL (локально, бесплатно). Apify Skills — для специализированных платформ (соцсети, e-commerce, Google Maps) где нужны API-обёртки.

---

## Примеры использования

### Извлечение контента

```bash
$ eclipse-claw https://stripe.com -f llm

> URL: https://stripe.com
> Title: Stripe | Financial Infrastructure for the Internet
> Language: en
> Word count: 847

# Stripe | Financial Infrastructure for the Internet

Stripe is a suite of APIs powering online payment processing
and commerce solutions for internet businesses of all sizes.

## Products
- Payments — Accept payments online and in person
- Billing — Manage subscriptions and invoicing
...
```

### Извлечение бренда

```bash
$ eclipse-claw https://github.com --brand

{
  "name": "GitHub",
  "colors": [{"hex": "#59636E", "usage": "Primary"}, ...],
  "fonts": ["Mona Sans", "ui-monospace"],
  "logos": [{"url": "https://github.githubassets.com/...", "kind": "svg"}]
}
```

### Краулинг сайта

```bash
$ eclipse-claw https://docs.rust-lang.org --crawl --depth 2 --max-pages 50

Crawling... 50/50 pages extracted
```

---

## MCP-сервер — 11 инструментов для AI-агентов

Eclipse Claw работает как MCP-сервер для Claude Desktop, Claude Code, Cursor, Windsurf, OpenCode и любого MCP-совместимого клиента.

```bash
npx create-eclipse-claw    # автоопределение и настройка
```

Ручная настройка — добавьте в конфиг Claude Desktop:

```json
{
  "mcpServers": {
    "eclipse-claw": {
      "command": "~/.eclipse-claw/eclipse-claw-mcp"
    }
  }
}
```

### Доступные инструменты

| Инструмент | Описание | Нужен API-ключ? |
|-----------|---------|:-:|
| `scrape` | Извлечение контента из любого URL | Нет |
| `crawl` | Рекурсивный обход сайта | Нет |
| `map` | Обнаружение URL через sitemap | Нет |
| `batch` | Параллельное извлечение из нескольких URL | Нет |
| `extract` | Структурированное извлечение через LLM | Нет (нужен Ollama) |
| `summarize` | Суммаризация страницы | Нет (нужен Ollama) |
| `diff` | Обнаружение изменений контента | Нет |
| `brand` | Извлечение айдентики бренда | Нет |
| `search` | Веб-поиск + скрапинг результатов | Да |
| `research` | Глубокое исследование из нескольких источников | Да |
| `doctor` | Показывает доступные connectors, data boundary и порядок fallback без сетевых проверок и чтения секретов | Нет |

**9 из 11 инструментов могут работать без Eclipse Cloud.** Перед исследованием вызовите
`doctor`: он простыми словами покажет, какой connector готов, куда могут уйти данные и почему
fallback отключён. Команда не проверяет credentials по сети, не открывает browser profile и
не устанавливает сторонние программы.

---

## Возможности

### Извлечение контента

- **Оценка читаемости** — многосигнальное определение контента (плотность текста, семантические теги, соотношение ссылок)
- **Фильтрация шума** — удаление навигации, подвалов, рекламы, модалов, баннеров cookies
- **Data island extraction** — извлечение React/Next.js JSON-данных, JSON-LD, данных гидрации
- **YouTube-метаданные** — структурированные данные из любого видео
- **PDF-извлечение** — автоопределение по Content-Type
- **5 форматов вывода** — Markdown, текст, JSON, LLM-оптимизированный, HTML

### Управление контентом

```bash
eclipse-claw URL --include "article, .content"       # CSS-селекторы для включения
eclipse-claw URL --exclude "nav, footer, .sidebar"    # CSS-селекторы для исключения
eclipse-claw URL --only-main-content                  # Автоопределение основного контента
```

### Краулинг

```bash
eclipse-claw URL --crawl --depth 3 --max-pages 100   # BFS-обход одного домена
eclipse-claw URL --crawl --sitemap                    # Посев из sitemap
eclipse-claw URL --map                                # Только обнаружение URL
```

### LLM-функции (Ollama / OpenAI / Anthropic)

```bash
eclipse-claw URL --summarize                          # Краткое содержание страницы
eclipse-claw URL --extract-prompt "Получи все цены"   # Извлечение на естественном языке
eclipse-claw URL --extract-json '{"type":"object"}'   # Извлечение по JSON-схеме
```

### Отслеживание изменений

```bash
eclipse-claw URL -f json > snap.json                  # Сохранить снимок
eclipse-claw URL --diff-with snap.json                # Сравнить позже
```

### Извлечение бренда

```bash
eclipse-claw URL --brand                              # Цвета, шрифты, логотипы, OG-изображение
```

### Ротация прокси

```bash
eclipse-claw URL --proxy http://user:pass@host:port   # Один прокси
eclipse-claw URLs --proxy-file proxies.txt            # Пул с ротацией
```

---

## Проверяемый benchmark gate

В репозитории закреплены четыре небольших public-page fixtures: article, documentation,
product и SPA/data-island. Для каждого файла сохранены источник, дата фиксации и SHA-256.
CI проверяет целостность fixtures, ожидаемые extraction signals и устойчивость LLM boundary
к инструкциям, встроенным в HTML.

```bash
node scripts/verify-benchmark-fixtures.mjs
cargo test -p eclipse-claw-core --test fixed_public_benchmark
cargo test -p eclipse-claw-llm --test fixed_security_benchmark
```

Это regression gate, а не заявление о лидерстве по скорости или точности. Старые сравнительные
цифры не используются как release-критерий, пока для них нет публичного runner, fixtures и
машиночитаемого отчёта. Подробности — в [benchmarks/](benchmarks/).

---

## Уникальные возможности Eclipse Claw

### REST API сервер

В отличие от большинства аналогов, Eclipse Claw включает встроенный HTTP-сервер для интеграции с любым стеком:

```bash
# Безопасный локальный запуск (по умолчанию 127.0.0.1:3000)
eclipse-claw-server

# Извлечь контент
curl -X POST http://localhost:3000/extract \
  -H 'Content-Type: application/json' \
  -d '{"url": "https://example.com"}'

# Суммаризация через отдельный LLM worker
curl -X POST http://localhost:3000/summarise \
  -H 'Content-Type: application/json' \
  -d '{"url": "https://news.ycombinator.com"}'

# Batch (до 50 URL параллельно)
curl -X POST http://localhost:3000/batch \
  -H 'Content-Type: application/json' \
  -d '{"urls": ["https://a.com", "https://b.com"]}'
```

Сервер не открывается во внешнюю сеть без аутентификации. Для осознанного внешнего bind
нужен Bearer token длиной от 32 символов:

```bash
export ECLIPSE_SERVER_ADDR=0.0.0.0:3000
export ECLIPSE_SERVER_TOKEN='replace-with-a-random-secret-of-32-plus-chars'
eclipse-claw-server

curl -X POST http://server:3000/extract \
  -H "Authorization: Bearer $ECLIPSE_SERVER_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"url": "https://example.com"}'
```

По умолчанию разрешены только публичные HTTP(S)-адреса: localhost, private/link-local сети,
cloud metadata endpoints и redirects на них блокируются. CLI может обратиться к доверенному
локальному сервису только с явным `--allow-private-network`. Размер ответа ограничен 20 MiB,
а server concurrency — `ECLIPSE_MAX_CONCURRENCY`.

### Изолированные LLM и browser workers

Production REST работает fail-closed: provider API keys и Chromium не находятся в процессе
`eclipse-claw-server`. LLM-запросы уходят в аутентифицированный `llm-worker`, а CDP включается
только через отдельный `cdp-worker`. Простой безопасный запуск описан в `docker-compose.yml`:

```bash
cp env.example .env
# Замените три token placeholder и закрепите OLLAMA_IMAGE по sha256.
docker compose up -d server llm-worker ollama
# Browser worker запускается только при реальной необходимости:
docker compose --profile browser up -d cdp-worker
```

Для ручного запуска server задайте `ECLIPSE_LLM_WORKER_URL` и отдельный
`ECLIPSE_LLM_WORKER_TOKEN`. Если `ECLIPSE_REQUIRE_ISOLATED_WORKERS=1`, отсутствие корректного
worker приводит к явной ошибке, а не к скрытому direct-provider fallback.

#### Extraction design tokens через Chrome DevTools Protocol

```bash
# Запустить Chrome с DevTools
google-chrome --remote-debugging-port=9222

# Извлечь точные design tokens через getComputedStyle()
eclipse-claw https://linear.app --design-tokens
# → JSON: цвета, типографика, отступы, тени, CSS-переменные

# Через REST API server + отдельный worker
export ECLIPSE_ENABLE_CDP=1
export ECLIPSE_CDP_WORKER_URL=http://127.0.0.1:3100
export ECLIPSE_CDP_WORKER_TOKEN='different-random-secret-of-32-plus-chars'
curl -X POST http://localhost:3000/design-tokens \
  -H 'Content-Type: application/json' \
  -d '{"url": "https://vercel.com"}'
```

Вывод:
```json
{
  "colors": { "backgrounds": [...], "foregrounds": [...], "accents": [...] },
  "typography": { "families": ["Inter", "JetBrains Mono"], "sizes": ["12px","14px",...] },
  "spacing": { "gaps": ["8px","16px","24px"], "max_widths": ["1200px"] },
  "css_variables": [{ "name": "--color-primary", "value": "#6366f1" }],
  "color_scheme": "dark",
  "scroll_library": "lenis"
}
```

### DeepSeek в LLM-цепочке

LLM-провайдеры выстроены в порядке стоимости: сначала бесплатный локальный Ollama, затем облако:

```
Ollama (локально, бесплатно) → DeepSeek → OpenAI → Anthropic
```

Стоимость и условия облачных провайдеров меняются; сверяйте текущие тарифы перед включением.
Для активации DeepSeek задайте ключ только в изолированном LLM worker:

```bash
export DEEPSEEK_API_KEY=sk-...
```

### JSONL-вывод для пайплайнов

```bash
# Один JSON-объект на строку — удобно для jq, Loki, Elasticsearch
eclipse-claw --urls-file urls.txt --jsonl | jq '.metadata.title'

# Потоковая обработка больших батчей
eclipse-claw --urls-file 10000_urls.txt --jsonl --concurrency 20 > results.jsonl
```

---

## Архитектура

```
eclipse-claw/
  crates/
    eclipse-claw-core     Движок извлечения. Без I/O. WASM-совместим.
    eclipse-claw-audit    Privacy-preserving JSONL audit с rotation и retention.
    eclipse-claw-fetch    HTTP-клиент + TLS-отпечатки (wreq/BoringSSL). Краулер. Batch.
    eclipse-claw-llm      Цепочка LLM-провайдеров (Ollama -> DeepSeek -> OpenAI -> Anthropic)
    eclipse-claw-pdf      Извлечение текста из PDF
    eclipse-claw-server   REST API сервер (Axum) — /extract, /summarise, /batch
    eclipse-claw-worker   Изолированный authenticated LLM/CDP worker.
    eclipse-claw-mcp      MCP-сервер (11 инструментов для AI-агентов + read-only doctor)
    eclipse-claw-cli      CLI-утилита
```

`eclipse-claw-core` принимает сырой HTML как `&str` и возвращает структурированный вывод. Без I/O, без сети — может компилироваться в WASM.

---

## Конфигурация

| Переменная | Описание |
|-----------|---------|
| `ECLIPSE_CLAW_API_KEY` | API-ключ облака (обход ботов, JS-рендеринг, поиск, исследования) |
| `ECLIPSE_CLAW_CLOUD_FALLBACK` | Явное согласие на автоматический local → cloud fallback (`1` включает; по умолчанию выключено) |
| `OLLAMA_HOST` | URL Ollama для локальных LLM-функций (по умолчанию: `http://localhost:11434`) |
| `DEEPSEEK_API_KEY` | API-ключ DeepSeek — первый облачный провайдер в цепочке (дешевле GPT-4o) |
| `OPENAI_API_KEY` | API-ключ OpenAI для LLM-функций |
| `ANTHROPIC_API_KEY` | API-ключ Anthropic для LLM-функций |
| `ECLIPSE_CLAW_PROXY` | URL одного прокси |
| `ECLIPSE_CLAW_PROXY_FILE` | Путь к файлу с пулом прокси |
| `ECLIPSE_CLAW_ALLOW_PROXY_DNS` | Явное согласие MCP доверить DNS настроенному proxy (`1`; иначе MCP fail-closed) |
| `ECLIPSE_CLAW_ALLOW_SESSION_COOKIES` | Явное локальное разрешение MCP передавать cookies (`1`; по умолчанию запрещено) |
| `ECLIPSE_SERVER_ADDR` | Адрес REST API сервера (по умолчанию: `127.0.0.1:3000`) |
| `ECLIPSE_SERVER_TOKEN` | Bearer token (минимум 32 символа), обязательный для любого non-loopback bind |
| `ECLIPSE_REQUIRE_ISOLATED_WORKERS` | Запрещает direct LLM providers в server-процессе; production должен использовать `1` |
| `ECLIPSE_LLM_WORKER_URL` / `ECLIPSE_LLM_WORKER_TOKEN` | URL и отдельный Bearer token изолированного LLM worker |
| `ECLIPSE_ENABLE_CDP` | Явно включает `/design-tokens`; без worker endpoint server завершается с ошибкой |
| `ECLIPSE_CDP_WORKER_URL` / `ECLIPSE_CDP_WORKER_TOKEN` | URL и отдельный Bearer token browser worker |
| `ECLIPSE_MAX_CONCURRENCY` | Макс. параллельных fetch-соединений в сервере (по умолчанию: `32`) |
| `ECLIPSE_AUDIT_DIR` | Каталог durable JSONL audit; обязателен при `ECLIPSE_AUDIT_REQUIRED=1` |
| `ECLIPSE_AUDIT_RETENTION_DAYS` | Retention audit-файлов: 14 дней по умолчанию, максимум 90 |
| `ECLIPSE_AUDIT_READ_ENABLED` | Отдельный opt-in для Bearer-protected audit read endpoint |

## Безопасная работа с AI-агентами

- MCP помечает ответы сайтов и поисковые snippets как `untrusted`: это данные, а не команды агенту.
- LLM extraction и summarization получают web-контент в отдельной границе и обязаны игнорировать
  инструкции со страницы. Это снижает риск prompt injection, но не заменяет human review для
  действий с секретами, аккаунтами или production.
- MCP cookies выключены по умолчанию. Не передавайте production sessions и личные аккаунты;
  включайте opt-in только в отдельном локальном процессе с минимальными правами.
- Proxy выполняет DNS вне локальной egress-политики. MCP требует отдельный proxy-DNS opt-in;
  `proxies.txt` больше не подхватывается автоматически из текущей директории.
- Crawler по умолчанию учитывает `robots.txt`, `Allow`/`Disallow` и `Crawl-delay`, сохраняет
  same-origin scope и ограничивает concurrency.
- Durable audit хранит только фиксированные `service`, `operation`, `outcome`, status и timing.
  URL, IP, query, headers, cookies, API keys, prompt и содержимое страниц в записи не попадают.
- Audit write может быть обязательным: при `ECLIPSE_AUDIT_REQUIRED=1` ошибка записи останавливает
  запуск или завершает запрос fail-closed. Read endpoint выключен отдельно по умолчанию.
- REST CORS не разрешён по умолчанию. Для browser-клиента задайте конкретный origin на
  reverse proxy вместе с TLS, rate limit и своим auth/authorization слоем.

---

## Облачный API (опционально)

Для сайтов с защитой от ботов, JS-рендерингом и продвинутыми функциями доступен облачный API.

CLI и MCP-сервер сначала работают локально. Наличие API key само по себе **не разрешает**
автоматическую передачу URL и контента в cloud. Облако используется как фолбэк, только когда
пользователь отдельно включил `--cloud-fallback` или `ECLIPSE_CLAW_CLOUD_FALLBACK=1`, а локальный
ответ показывает одну из этих проблем:
- Сайт имеет защиту от ботов (Cloudflare, DataDome, WAF)
- Страница требует JavaScript-рендеринг

`search`, `research` и `--cloud` — отдельные явные cloud-действия. Для них нужен API key,
но дополнительный fallback-флаг не нужен.

```bash
export ECLIPSE_CLAW_API_KEY=wc_your_key
export ECLIPSE_CLAW_CLOUD_FALLBACK=1

# Явно разрешённый fallback: сначала локально, затем cloud при обнаружении защиты
eclipse-claw --cloud-fallback https://protected-site.com

# Принудительно через облако
eclipse-claw --cloud https://spa-site.com
```

Для server-side диагностики доступны `GET /connectors` и `GET /connectors/doctor`. Они
возвращают статический allowlist, readiness, provenance и безопасный следующий шаг, не
показывая значения ключей и не выполняя network probes.

### SDK

```bash
npm install @eclipse-claw/sdk                  # TypeScript/JavaScript
pip install eclipse-claw                        # Python
go get github.com/PavelHopson/eclipse-claw-go   # Go
```

---

## Сценарии применения

- **AI-агенты** — предоставьте Claude/Cursor/GPT доступ к вебу через MCP
- **Исследования** — краулинг документации, сайтов конкурентов, архивов новостей
- **Мониторинг цен** — отслеживание изменений через `--diff-with` снимки
- **Обучающие данные** — подготовка веб-контента для файн-тюнинга с оптимизацией токенов
- **Контент-пайплайны** — пакетное извлечение + суммаризация в CI/CD
- **Бренд-аналитика** — извлечение визуальной айдентики любого сайта

---

## Roadmap

### Crawl Mode

Основной безопасный crawl mode уже реализован:

- [x] `--crawl`, `--depth`, `--max-pages` и same-origin обход
- [x] `robots.txt` `Allow`/`Disallow` и `Crawl-delay` по умолчанию
- [x] настраиваемые delay и concurrency
- [x] sitemap seeding и дедупликация URL
- [ ] persistence/resume для прерванного обхода

```bash
# Рабочий синтаксис
eclipse-claw --crawl --depth 3 --same-domain https://docs.example.com --jsonl > docs.jsonl

# С rate limiting
eclipse-claw --crawl --delay 500ms --respect-robots https://blog.example.com
```

### Telegram Parsing (вдохновлено [TGSpyder](https://github.com/Darksight-Analytics/tgspyder))

Новый crate `eclipse-claw-telegram` для извлечения данных из Telegram:

- [ ] Подключение через Telegram API (TDLib или grammers — Rust-native Telegram client)
- [ ] Парсинг участников чатов/каналов — ID, username, имя, статус
- [ ] Выгрузка истории сообщений — текст, дата, автор, reply chains
- [ ] Извлечение медиа-метаданных (фото, видео, документы) без скачивания
- [ ] Поиск пользователей по ID и username
- [ ] Парсинг инвайт-ссылок и пересылок
- [ ] Экспорт в JSONL / CSV (единый формат с основным парсером)
- [ ] MCP-инструмент `telegram_extract` для AI-агентов

```bash
# Планируемый синтаксис
eclipse-claw --telegram --chat @channel_name --messages --jsonl > messages.jsonl
eclipse-claw --telegram --chat @group_name --members --csv > members.csv
eclipse-claw --telegram --user 123456789 --info
```

### Другие планы

- [x] Allowlisted connector registry + read-only `doctor` для REST/MCP; automatic cloud fallback
      требует отдельного opt-in и никогда не включается только из-за сохранённого API key
- [x] CSS selector фильтрация (`--include "article.main"` / `--exclude "nav,footer"`)
- [x] Sitemap-first crawl mode
- [ ] Webhook уведомления при обнаружении изменений
- [x] Опциональный CDP worker для SPA/design tokens, изолированный от REST process

---

## Участие в разработке

Приветствуются контрибуции! Смотрите [CONTRIBUTING.md](CONTRIBUTING.md) для руководства.

- [Issues](https://github.com/PavelHopson/Eclipse-Claw/issues) — баг-репорты и запросы функций

## Благодарности

TLS и HTTP/2 браузерные отпечатки реализованы на основе [wreq](https://github.com/0x676e67/wreq) и [http2](https://github.com/0x676e67/http2) от [@0x676e67](https://github.com/0x676e67).

## Лицензия

[AGPL-3.0](LICENSE)
