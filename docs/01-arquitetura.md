# 01 — Arquitetura

## 1. Visão em camadas

```
┌──────────────────────────────────────────────────────────────┐
│ app (binário)                                                │
│  máquina de estados da aplicação, loop de mensagens,         │
│  tradução Evento → Input do jogo, agendamento de animação    │
├───────────────┬───────────────────────┬──────────────────────┤
│ game          │ render                │ platform             │
│ lógica pura   │ rasterização em       │ Win32: janela,       │
│ do jogo       │ buffer BGRA           │ taskbar, bandeja,    │
│ (sem Windows) │ (sem Windows)         │ DPI, sessão, energia │
│ forbid(unsafe)│ forbid(unsafe)        │ único crate c/ unsafe│
├───────────────┴───────────┬───────────┴──────────────────────┤
│ assets                    │ sim (binário headless)           │
│ sprites/fonte embutidos   │ balanceamento: milhares de lutas │
└───────────────────────────┴──────────────────────────────────┘
```

Regras de dependência (verificadas pela própria estrutura do `Cargo.toml` de cada crate):

| Crate | Pode depender de | Não pode depender de |
|---|---|---|
| `game` | `serde` | `render`, `platform`, `windows-sys`, qualquer I/O |
| `render` | `assets` | `game`, `platform`, `windows-sys` |
| `assets` | `png` (ou QOI) | `platform`, `windows-sys` |
| `platform` | `windows-sys`, `log` | `game`, `render` |
| `sim` | `game` | `platform`, `render` |
| `app` | todos acima | — |

Consequência: `game`, `render`, `assets` e `sim` compilam e são testados no Linux, o que deixa o CI mais
barato e permite fuzzing e testes de imagem sem Windows.

## 2. Estrutura de diretórios

```
TaskBarFGladiator/
├─ Cargo.toml                 workspace, [workspace.lints], [profile.*]
├─ Cargo.lock                 commitado (binário final → build reproduzível)
├─ rust-toolchain.toml        versão exata do Rust
├─ rustfmt.toml
├─ deny.toml                  cargo-deny: licenças, advisories, bans, fontes
├─ crates/
│  ├─ game/
│  │  └─ src/ {lib.rs, state.rs, combat.rs, ai.rs, items.rs, progression.rs,
│  │           rng.rs, fixed.rs, save/{mod.rs, schema.rs, migrate.rs, validate.rs}}
│  ├─ render/
│  │  └─ src/ {lib.rs, surface.rs, blit.rs, font.rs, scale.rs, widgets.rs, scene.rs}
│  ├─ assets/
│  │  ├─ build.rs             (opcional) empacota PNGs em atlas no build
│  │  └─ src/ {lib.rs, atlas.rs}
│  ├─ platform/
│  │  └─ src/ {lib.rs, window.rs, layered.rs, taskbar.rs, zorder.rs, tray.rs,
│  │           dpi.rs, session.rs, power.rs, fullscreen.rs, paths.rs, single_instance.rs,
│  │           shell_classes.rs, raii.rs}
│  └─ sim/
│     └─ src/main.rs
├─ app/
│  ├─ build.rs                embute manifesto e recurso de versão
│  ├─ app.manifest
│  └─ src/ {main.rs, events.rs, controller.rs, animation.rs, settings.rs, i18n.rs, logging.rs}
├─ assets-src/                arte em formato editável + CREDITS.md
├─ fuzz/                      cargo-fuzz (alvo: parser de save)
├─ docs/
└─ .github/ {workflows/, dependabot.yml, ISSUE_TEMPLATE/}
```

## 3. Renderização

### 3.1 Decisão: software + `UpdateLayeredWindow`

| Opção | Memória | Complexidade | Decisão |
|---|---|---|---|
| Direct3D 11 / Direct2D | +30 a +80 MB de working set (DLLs do driver de vídeo, heaps do driver) | Tratar *device lost*, troca de GPU, drivers com bug | Rejeitada |
| GDI puro (`BitBlt`, `TextOut`) | Baixa | GDI não preserva canal alpha em janela layered | Rejeitada para desenho |
| **Software em `Vec<u32>` + `UpdateLayeredWindow`** | Buffer de ~77 KB (400×48×4) | Baixa; testável no Linux | **Escolhida** |

### 3.2 Fluxo de um quadro

1. `render` desenha a cena em `Surface { width, height, pixels: Vec<u32> }`, formato BGRA com **alpha
   premultiplicado** (cada canal de cor já multiplicado pelo alpha).
2. `platform` copia o buffer para uma DIB section criada com `CreateDIBSection`, `BITMAPINFOHEADER` com
   `biBitCount = 32`, `biCompression = BI_RGB` e `biHeight` **negativo** (origem no topo).
   Otimização: renderizar direto na memória da DIB section (o ponteiro `ppvBits`) e eliminar a cópia.
3. `UpdateLayeredWindow(hwnd, hdcScreen, &pos, &size, hdcMem, &ptSrc, 0, &blend, ULW_ALPHA)` com
   `BLENDFUNCTION { BlendOp: AC_SRC_OVER, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA }`.

Detalhes importantes:

- Se os pixels não estiverem premultiplicados, aparecem halos claros nas bordas e cores erradas.
- Pixels com alpha 0 não recebem eventos de mouse: o clique passa para a barra de tarefas embaixo. Áreas
  clicáveis do jogo precisam de alpha ≥ 1.
- Em janelas `WS_EX_LAYERED` atualizadas com `UpdateLayeredWindow`, **não** existe `WM_PAINT`; o redesenho
  é sempre explícito.

### 3.3 Escala e DPI

- Arte desenhada em **altura base de 24 px lógicos**.
- Fator de escala inteiro: `escala = max(1, altura_da_barra_em_px / 24)` com arredondamento para baixo.
  Exemplos: 40 px → ×1 (com 16 px de margem) ou arte alternativa de 40; 48 px → ×2; 60 px → ×2; 72 px → ×3.
- Escala por vizinho mais próximo (sem filtro), para manter a pixel art nítida.
- Sobra vertical é distribuída como margem (centralizado).
- Fator não inteiro (ex.: ×2,5) produz pixels de tamanhos diferentes e é proibido.

### 3.4 Texto

- Fonte bitmap própria (pixel font, ex.: 5×7 com 1 px de contorno), embutida no executável.
- Motivo: `TextOut`/`DrawText` da GDI escrevem alpha 0 na DIB section; o texto fica invisível ou com
  bordas erradas na janela layered. Além disso, a fonte bitmap tem aparência idêntica em qualquer sistema.
- Suporte a caracteres do português: `á à â ã é ê í ó ô õ ú ç` e maiúsculas correspondentes.

## 4. Bindings Win32

- Crate `windows-sys`: bindings brutos gerados a partir dos metadados oficiais da Microsoft; compila mais
  rápido e gera binário menor que o crate `windows` (que adiciona wrappers COM e tipos de alto nível).
- Camada própria em `platform/src/raii.rs` com tipos que liberam o recurso no `Drop`:

| Tipo | Recurso | Liberação |
|---|---|---|
| `OwnedHwnd` | janela | `DestroyWindow` |
| `MemDc` | `CreateCompatibleDC` | restaura objeto selecionado + `DeleteDC` |
| `DibSection` | `CreateDIBSection` | `DeleteObject` |
| `ScreenDc` | `GetDC(NULL)` | `ReleaseDC` |
| `OwnedIcon` | `HICON` | `DestroyIcon` |
| `OwnedMenu` | `HMENU` | `DestroyMenu` |
| `WinEventHook` | `SetWinEventHook` | `UnhookWinEvent` |
| `TrayIcon` | `Shell_NotifyIconW(NIM_ADD)` | `Shell_NotifyIconW(NIM_DELETE)` |
| `NamedMutex` | `CreateMutexW` | `CloseHandle` |
| `SessionNotification` | `WTSRegisterSessionNotification` | `WTSUnRegisterSessionNotification` |

- Lints obrigatórios no crate `platform`:
  - `unsafe_op_in_unsafe_fn = "deny"`
  - `clippy::undocumented_unsafe_blocks = "deny"` (todo `unsafe { }` precisa de `// SAFETY:` explicando
    por que as pré-condições são satisfeitas)
  - `clippy::multiple_unsafe_ops_per_block = "warn"`

## 5. Loop de eventos

### 5.1 Modelo

```
GetMessageW (bloqueia; 0 % de CPU sem eventos)
   │
   ├─ WndProc recebe a mensagem
   │     └─ converte para enum Event e enfileira (VecDeque), retorna rápido
   │
   └─ após DispatchMessageW, o loop drena a fila:
         Event → controller → game::step(...) → Vec<GameEvent>
                                      └─ animação → agenda timer se necessário
                                      └─ render → UpdateLayeredWindow
```

### 5.2 Por que enfileirar em vez de processar dentro do WndProc

Funções como `SetWindowPos`, `ShowWindow`, `DestroyWindow` e `TrackPopupMenu` enviam mensagens síncronas,
que chamam o WndProc de novo antes de retornar (reentrância). Se o código estiver com um
`RefCell::borrow_mut()` ativo nesse momento, a segunda entrada causa panic `already borrowed`.
Regra: o WndProc só lê a mensagem, cria o `Event` e enfileira. Nenhuma chamada Win32 que envia mensagens
é feita enquanto houver borrow ativo do estado.

### 5.3 Timers

- Sem animação: nenhum timer ativo.
- Com animação: `SetCoalescableTimer(hwnd, ID, 33, NULL, tolerância)` (~30 quadros/s). A tolerância permite
  que o Windows agrupe o despertar com outros timers, economizando energia.
- Ao terminar a animação: `KillTimer`.
- `timeBeginPeriod` é proibido: altera a resolução de timer do sistema inteiro e aumenta o consumo de energia.
- Tempo medido com `std::time::Instant` (usa `QueryPerformanceCounter`). `dt` limitado a 100 ms por quadro
  para evitar saltos após suspensão ou travamento temporário.

### 5.4 Threads

| Thread | Função |
|---|---|
| UI (principal) | mensagens, lógica, renderização |
| I/O de save | recebe cópia do estado por canal, grava em disco |
| UI Automation (pós-1.0, opcional) | consulta posições de botões da barra com timeout |

A lógica do jogo é barata (microssegundos por turno); não há motivo para paralelizá-la.

## 6. Dependências

| Crate | Uso | Observação |
|---|---|---|
| `windows-sys` | FFI Win32 | só em `platform` |
| `serde`, `serde_json` | save e config | `game` e `app` |
| `png` (ou decodificador QOI) | decodificar assets na inicialização | `assets` |
| `log` | fachada de log | logger próprio grava em arquivo |
| `embed-resource` ou `winres` (build) | manifesto e recurso de versão | só em `build.rs` do `app` |
| `proptest` (dev) | testes de propriedade | só testes |

Proibido (barrado em `deny.toml`): qualquer crate de rede (`reqwest`, `hyper`, `ureq`, `tokio` com rede),
frameworks de GUI e engines de jogo.

## 7. Profiles de compilação

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
overflow-checks = true

[profile.dev]
overflow-checks = true
```

- `panic = "abort"`: binário menor e sem unwinding atravessando funções `extern "system"`.
- `overflow-checks = true`: em release o Rust, por padrão, faz *wrap* silencioso em overflow inteiro
  (ex.: ouro passando do máximo vira número pequeno). Com a opção, vira panic registrado em log. Os tipos
  de domínio (ouro, XP, dano) usam `saturating_*` para que o panic nunca aconteça na prática.
- Não usar UPX nem outro compressor de executável (ver [`07-seguranca.md`](07-seguranca.md)).

## 8. Manifesto do executável

```xml
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security><requestedPrivileges>
      <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
    </requestedPrivileges></security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/> <!-- Windows 10/11 -->
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
      <activeCodePage xmlns="http://schemas.microsoft.com/SMI/2019/WindowsSettings">UTF-8</activeCodePage>
      <heapType xmlns="http://schemas.microsoft.com/SMI/2020/WindowsSettings">SegmentHeap</heapType>
    </windowsSettings>
  </application>
</assembly>
```

| Entrada | Motivo |
|---|---|
| `asInvoker` | nunca solicita elevação (UAC) |
| `PerMonitorV2` | coordenadas em pixels físicos corretos em cada monitor; recebe `WM_DPICHANGED` |
| `UTF-8` | APIs "A" tratam texto como UTF-8 (o código usa APIs "W", mas protege bibliotecas) |
| `SegmentHeap` | heap moderno do Windows 10 2004+, com menor uso de memória para o mesmo padrão de alocação |

Recurso de versão (`VS_VERSIONINFO`): `ProductName`, `FileDescription`, `CompanyName` (autor),
`FileVersion`, `LegalCopyright`. Executáveis sem esse recurso recebem mais desconfiança de antivírus.

## 9. Máquina de estados da aplicação

```
          ┌────────────┐  taskbar ok   ┌──────────────┐
 início → │ Detectando │──────────────▶│ NaBarra      │◀─┐
          └────────────┘               └──────────────┘  │ taskbar volta
                │ falhou/vertical        │  │ tela cheia │
                ▼                        │  ▼            │
          ┌────────────┐                 │ ┌──────────┐  │
          │ Flutuante  │                 │ │ Oculto   │──┘
          └────────────┘                 │ └──────────┘
                                         │ sessão bloqueada
                                         ▼
                                    ┌──────────┐
                                    │ Pausado  │
                                    └──────────┘
```

O estado da aplicação (onde e como a janela aparece) é separado do estado do jogo (`game::GameState`).
