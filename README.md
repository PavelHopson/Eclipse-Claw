<p align="center">
  <img src=".github/banner.png" alt="Eclipse Claw" width="700" />
</p>

<h3 align="center">
  Самый быстрый веб-скрапер для AI-агентов<br/>
  <sub>На 67% меньше токенов. Извлечение за миллисекунды. Без headless-браузера.</sub>
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

Извлекает чистый структурированный контент из любого URL с помощью TLS-отпечатков уровня Chrome — без headless-браузера, без Selenium, без Puppeteer. Вывод оптимизирован для LLM: **на 67% меньше токенов**, с сохранением метаданных, ссылок и изображений.

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
│                                  │    │ - Инференс в 3 раза быстрее     │
│         4 820 токенов            │    │         1 590 токенов            │
└──────────────────────────────────┘    └──────────────────────────────────┘
```

---

## Быстрый старт (30 секунд)

### Для AI-агентов (Claude, Cursor, Windsurf, VS Code)

```bash
npx create-eclipse-claw
```

Автоматически определяет ваши AI-инструменты, скачивает MCP-сервер и настраивает всё. Одна команда.

### Homebrew (macOS/Linux)

```bash
brew tap PavelHopson/eclipse-claw
brew install eclipse-claw
```

### Готовые бинарники

Скачайте из [GitHub Releases](https://github.com/PavelHopson/Eclipse-Claw/releases) для macOS (arm64, x86_64) и Linux (x86_64, aarch64).

### Cargo (из исходников)

```bash
cargo install --git https://github.com/PavelHopson/Eclipse-Claw.git eclipse-claw-cli
cargo install --git https://github.com/PavelHopson/Eclipse-Claw.git eclipse-claw-mcp
```

### Docker

```bash
docker run --rm ghcr.io/pavelhopson/eclipse-claw https://example.com
```

### Docker Compose (с Ollama для LLM-функций)

```bash
cp env.example .env
docker compose up -d
```

---

## Сравнение с аналогами

| | Eclipse Claw | Firecrawl | Trafilatura | [Apify Skills](https://github.com/apify/agent-skills) |
|---|:---:|:---:|:---:|:---:|
| **Точность извлечения** | **95.1%** | — | 80.6% | Зависит от платформы |
| **Экономия токенов** | **-67%** | — | -55% | Структурированный JSON |
| **Скорость (100 КБ)** | **3.2 мс** | ~500 мс | 18.4 мс | Облачная обработка |
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

## MCP-сервер — 10 инструментов для AI-агентов

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

**8 из 10 инструментов работают локально** — без аккаунта, без API-ключа, полностью приватно.

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

## Бенчмарки

Все результаты получены на реальных тестах с 50 разнообразных страниц. Методология и инструкции по воспроизведению — в [benchmarks/](benchmarks/).

### Качество извлечения

```
Точность      Eclipse Claw ███████████████████ 95.1%
              Readability  ████████████████▋   83.5%
              Trafilatura  ████████████████    80.6%
              Newspaper3k  █████████████▎      66.4%

Фильтрация    Eclipse Claw ███████████████████ 96.1%
шума          Readability  █████████████████▊  89.4%
              Trafilatura  ██████████████████▏ 91.2%
              Newspaper3k  ███████████████▎    76.8%
```

### Скорость (чистое извлечение, без сети)

```
10 КБ         Eclipse Claw ██                   0.8 мс
              Readability  █████                2.1 мс
              Trafilatura  ██████████           4.3 мс

100 КБ        Eclipse Claw ██                   3.2 мс
              Readability  █████                8.7 мс
              Trafilatura  ██████████           18.4 мс
```

### Эффективность токенов (при подаче в Claude/GPT)

| Формат | Токены | vs Сырой HTML |
|--------|:------:|:-------------:|
| Сырой HTML | 4 820 | базовый |
| Readability | 2 340 | -51% |
| Trafilatura | 2 180 | -55% |
| **Eclipse Claw LLM** | **1 590** | **-67%** |

### Скорость краулинга

| Параллелизм | Eclipse Claw | Crawl4AI | Scrapy |
|:-----------:|:-------:|:--------:|:------:|
| 5 | **9.8 стр/с** | 5.2 стр/с | 7.1 стр/с |
| 10 | **18.4 стр/с** | 8.7 стр/с | 12.3 стр/с |
| 20 | **32.1 стр/с** | 14.2 стр/с | 21.8 стр/с |

---

## Уникальные возможности Eclipse Claw

### REST API сервер

В отличие от большинства аналогов, Eclipse Claw включает встроенный HTTP-сервер для интеграции с любым стеком:

```bash
# Запустить сервер
eclipse-claw-server --addr 0.0.0.0:3000

# Извлечь контент
curl -X POST http://localhost:3000/extract \
  -H 'Content-Type: application/json' \
  -d '{"url": "https://example.com"}'

# Суммаризация через LLM (Ollama/DeepSeek/OpenAI — автоцепочка)
curl -X POST http://localhost:3000/summarise \
  -H 'Content-Type: application/json' \
  -d '{"url": "https://news.ycombinator.com"}'

# Batch (до 50 URL параллельно)
curl -X POST http://localhost:3000/batch \
  -H 'Content-Type: application/json' \
  -d '{"urls": ["https://a.com", "https://b.com"]}'
```

### Extraction design tokens через Chrome DevTools Protocol

```bash
# Запустить Chrome с DevTools
google-chrome --remote-debugging-port=9222

# Извлечь точные design tokens через getComputedStyle()
eclipse-claw https://linear.app --design-tokens
# → JSON: цвета, типографика, отступы, тени, CSS-переменные

# Через REST API сервер
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

DeepSeek — самый дешёвый из облачных провайдеров (~3× дешевле GPT-4o). Для активации достаточно:

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
    eclipse-claw-fetch    HTTP-клиент + TLS-отпечатки (wreq/BoringSSL). Краулер. Batch.
    eclipse-claw-llm      Цепочка LLM-провайдеров (Ollama -> DeepSeek -> OpenAI -> Anthropic)
    eclipse-claw-pdf      Извлечение текста из PDF
    eclipse-claw-server   REST API сервер (Axum) — /extract, /summarise, /batch
    eclipse-claw-mcp      MCP-сервер (10 инструментов для AI-агентов)
    eclipse-claw-cli      CLI-утилита
```

`eclipse-claw-core` принимает сырой HTML как `&str` и возвращает структурированный вывод. Без I/O, без сети — может компилироваться в WASM.

---

## Конфигурация

| Переменная | Описание |
|-----------|---------|
| `ECLIPSE_CLAW_API_KEY` | API-ключ облака (обход ботов, JS-рендеринг, поиск, исследования) |
| `OLLAMA_HOST` | URL Ollama для локальных LLM-функций (по умолчанию: `http://localhost:11434`) |
| `DEEPSEEK_API_KEY` | API-ключ DeepSeek — первый облачный провайдер в цепочке (дешевле GPT-4o) |
| `OPENAI_API_KEY` | API-ключ OpenAI для LLM-функций |
| `ANTHROPIC_API_KEY` | API-ключ Anthropic для LLM-функций |
| `ECLIPSE_CLAW_PROXY` | URL одного прокси |
| `ECLIPSE_CLAW_PROXY_FILE` | Путь к файлу с пулом прокси |
| `ECLIPSE_SERVER_ADDR` | Адрес REST API сервера (по умолчанию: `0.0.0.0:3000`) |
| `ECLIPSE_MAX_CONCURRENCY` | Макс. параллельных fetch-соединений в сервере (по умолчанию: `32`) |

---

## Облачный API (опционально)

Для сайтов с защитой от ботов, JS-рендерингом и продвинутыми функциями доступен облачный API.

CLI и MCP-сервер работают локально. Облако используется как фолбэк, когда:
- Сайт имеет защиту от ботов (Cloudflare, DataDome, WAF)
- Страница требует JavaScript-рендеринг
- Вы используете инструменты поиска или исследования

```bash
export ECLIPSE_CLAW_API_KEY=wc_your_key

# Автоматически: сначала локально, облако при обнаружении защиты
eclipse-claw https://protected-site.com

# Принудительно через облако
eclipse-claw --cloud https://spa-site.com
```

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

### Crawl Mode (вдохновлено [Scrapy](https://github.com/scrapy/scrapy))

Сейчас Eclipse Claw извлекает контент из **одного URL**. Планируется полноценный crawl mode:

- [ ] `--crawl` флаг — обход всех ссылок на сайте от стартового URL
- [ ] `--depth N` — ограничение глубины обхода (по умолчанию: 3)
- [ ] `--same-domain` — только ссылки в пределах домена
- [ ] `robots.txt` уважение — автоматическая проверка и соблюдение правил
- [ ] Rate limiting / politeness — настраиваемые задержки между запросами (`--delay 500ms`)
- [ ] Sitemap.xml парсинг — обнаружение страниц через карту сайта
- [ ] Дедупликация URL — canonical URL нормализация
- [ ] Crawl state persistence — продолжение прерванного обхода

```bash
# Планируемый синтаксис
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

- [ ] CSS selector фильтрация (`--select "article.main"`)
- [ ] Sitemap-first crawl mode
- [ ] Webhook уведомления при обнаружении изменений
- [ ] Playwright/Chrome fallback для SPA (без облака)

---

## Участие в разработке

Приветствуются контрибуции! Смотрите [CONTRIBUTING.md](CONTRIBUTING.md) для руководства.

- [Issues](https://github.com/PavelHopson/Eclipse-Claw/issues) — баг-репорты и запросы функций

## Благодарности

TLS и HTTP/2 браузерные отпечатки реализованы на основе [wreq](https://github.com/0x676e67/wreq) и [http2](https://github.com/0x676e67/http2) от [@0x676e67](https://github.com/0x676e67).

## Лицензия

[AGPL-3.0](LICENSE)
