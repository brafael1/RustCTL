# rustctl

Uma interface de terminal (TUI) para gerenciar unidades do systemd, escrita em
Rust com [ratatui](https://crates.io/crates/ratatui) e
[crossterm](https://crates.io/crates/crossterm).

Ele envolve o `systemctl` (list-units, is-enabled, status, start/stop/restart/
enable/disable/reload) e expõe as informações em uma TUI controlada por teclado.

![Licença: MIT](https://img.shields.io/badge/licen%C3%A7a-MIT-blue.svg)

## Funcionalidades

- Lista todas as unidades carregadas (de todos os tipos) com um indicador
  colorido de estado (`●` ativa, `○` inativa, `✖` falhou, `◔` em transição).
- Abas de filtro por tipo de unidade:
  **Todas / service / socket / timer / target / mount**.
- Painel de detalhes com descrição, estados load / active / sub, status de
  enable e a saída ao vivo de `systemctl status`.
- Ações de uma tecla sobre a unidade selecionada:

  | Tecla        | Ação                  |
  |--------------|-----------------------|
  | `s`          | start (iniciar)       |
  | `S`          | stop (parar)          |
  | `r`          | restart (reiniciar)    |
  | `l`          | reload (recarregar)    |
  | `e`          | enable (habilitar)     |
  | `E`          | disable (desabilitar)  |
  | `R` (Shift)  | atualizar lista       |
  | `Tab`        | alternar filtro        |
  | `↑` `↓` / `j` `k` | mover seleção     |
  | `PageUp` / `PageDown` | pular 10 linhas |
  | `Home` / `End` | primeira / última unidade |
  | `q` / `Ctrl-C` | sair                |

- Barra de status com ajuda contextual e mensagens temporárias de feedback.

## Requisitos

- Linux com systemd
- Rust 1.74+ (edition 2021)
- `systemctl` no `PATH`

A maioria das ações (start/stop/restart/enable/disable) exige **root** ou uma
regra do polkit. A listagem somente leitura funciona como usuário comum.

## Build

```bash
cargo build --release
```

O binário otimizado fica em `target/release/rustctl`.

## Execução

```bash
# somente leitura como usuário atual
cargo run --release

# ações que modificam o estado do systemd
sudo ./target/release/rustctl
```

Na inicialização, se você não for root, o `rustctl` imprime um lembrete de uma
linha informando que operações de start/init falharão a menos que o polkit
esteja configurado, e aguarda Enter antes de entrar na TUI.

## Estrutura do projeto

```
src/
├── main.rs              setup do terminal, loop de eventos, teardown
├── systemd/             wrapper do systemctl (tipos + comandos)
│   ├── mod.rs
│   ├── types.rs         Unit, Enabled
│   └── commands.rs      list_units, is_enabled, status, verbos
├── app/                 estado da aplicação e lógica de domínio
│   ├── mod.rs           struct App, seleção, act(), refresh()
│   └── filters.rs       tabela FILTERS + matches_filter
├── ui/                  somente renderização
│   ├── mod.rs           dispatcher draw() + layout
│   └── render.rs        título, lista de unidades, detalhes, barra de status
└── event/               teclado → métodos de App
    └── mod.rs
```

As camadas são intencionalmente desacopladas:

- **systemd/** faz toda a E/S via subprocessos (testável isoladamente).
- **app/** detém o estado mutável; a UI e o input chamam métodos de `App` em
  vez de tocar no `systemd` diretamente.
- **ui/** apenas lê `&App` e constrói widgets do ratatui.
- **event/** apenas traduz pressionamentos de tecla em chamadas de métodos de
  `App`.

## Testes

```bash
cargo test --release
```

Testes unitários cobrem o parser da saída de `systemctl list-units`, incluindo
colunas alinhadas com múltiplos espaços (o formato `--plain`) e descrições que
contêm espaços.

## Licença

MIT — veja [LICENSE](LICENSE).