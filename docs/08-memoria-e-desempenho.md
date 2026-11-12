# 08 — Memória e desempenho

## 1. Orçamentos

| Recurso | Orçamento | Observação |
|---|---|---|
| Private Bytes total | < 15 MB | Inclui runtime do Rust, heaps do Windows e DLLs do sistema |
| Buffer de renderização da faixa | ~ 77 KB (400 × 48 × 4) | Cresce com DPI: 720 × 72 × 4 ≈ 207 KB a 150 % |
| DIB section | igual ao buffer | Renderizar direto nela elimina a cópia |
| Painel (quando aberto) | ~ 0,5 MB (540 × 420 × 4 a 150 %) | Liberado ao fechar o painel |
| Atlas de sprites decodificado | < 1 MB | Pixel art de 24 px base é pequena |
| Estado do jogo | < 64 KB | |
| Tempo de `game::step` | < 50 µs | Medido em benchmark |
| Tempo de renderizar um quadro da faixa | < 1 ms | Medido em benchmark |
| CPU parado | 0,0 % | Nenhum timer ativo |
| CPU em combate animado | < 1 % de um núcleo | 30 quadros/s × < 1 ms |
| Despertares por segundo parado | 0 | Verificável com `powercfg /energy` |

## 2. Fontes de consumo e medidas

### 2.1 Renderização
- **Direct3D/Direct2D não são usados.** Criar um dispositivo D3D carrega a DLL do driver de vídeo do
  usuário (específica do fabricante) e aloca heaps do driver; em muitos sistemas isso soma dezenas de MB de
  working set e alguns MB de Private Bytes, além do tratamento de perda de dispositivo.
- Renderização em software sobre buffer pequeno: custo de CPU desprezível e memória previsível.
- Redesenho somente quando algo mudou (evento do jogo, animação ativa, mudança de tema/posição).
- Opcional: retângulo sujo (redesenhar só a região alterada); só implementar se a medição mostrar necessidade.

### 2.2 Alocações
- Buffers (`Surface`, DIB section, filas) alocados uma vez e reutilizados.
- `SmallVec` para listas pequenas e frequentes (eventos de um turno).
- Teste automatizado com alocador contador garante zero alocação por quadro após o primeiro.
- Strings de interface vêm da tabela estática (`&'static str`); formatação de números em buffer
  reutilizável (`String` com `clear()`).

### 2.3 Recursos do sistema (GDI/USER/handles)
- Todo recurso com tipo RAII ([`01-arquitetura.md`](01-arquitetura.md), seção 4).
- Contadores no overlay de debug: `GetGuiResources(GR_GDIOBJECTS)`, `GetGuiResources(GR_USEROBJECTS)`,
  `GetProcessHandleCount`.
- Valores esperados em regime: GDI < 20, USER < 20, handles < 150. Qualquer crescimento contínuo é vazamento.

### 2.4 Heap
- `heapType = SegmentHeap` no manifesto: o heap segmentado do Windows 10 2004+ reduz a memória comprometida
  para o mesmo padrão de alocação em comparação com o heap NT padrão.
- Alocador padrão do sistema (sem `mimalloc`/`jemalloc`): menor binário e menos memória reservada em
  processo pequeno.

### 2.5 Executável e assets
- `include_bytes!` coloca os PNG na seção somente-leitura do executável. Essa memória é mapeada do arquivo
  e não conta em Private Bytes; páginas não usadas podem ser descartadas pelo sistema.
- O atlas decodificado (Private Bytes) é o único custo real; mantido pequeno.
- Release com `lto = "fat"`, `codegen-units = 1`, `strip = true`, `panic = "abort"`.

### 2.6 Threads
- Cada thread reserva pilha (1 MB reservado por padrão, comprometido sob demanda). Manter só as threads
  necessárias: UI + I/O de save. A thread de I/O pode ser criada com pilha menor
  (`std::thread::Builder::stack_size(64 * 1024)`).

### 2.7 Timers e energia
- Nenhum timer quando não há animação.
- `SetCoalescableTimer` com tolerância permite ao Windows agrupar despertares.
- Proibido `timeBeginPeriod`, laços com `sleep` curto e `PeekMessage` em laço ativo.
- Animações pausam quando a faixa está oculta, a sessão bloqueada ou a tela cheia ativa.

### 2.8 Estruturas de dados
- Estado em estruturas planas com índices (`Vec` + `usize`/IDs), sem `Rc<RefCell<..>>` em grafos: evita
  ciclos de referência (vazamento) e borrows dinâmicos que podem causar panic.
- Interface com layout imediato (calcula e desenha a cada quadro a partir do estado), sem árvore de
  widgets persistente.

## 3. Riscos específicos por fase

| Fase | Risco | Medida |
|---|---|---|
| 1 | Vazamento de GDI ao recriar DIB section em mudança de DPI | RAII; teste trocando DPI 100 vezes por script |
| 1 | CPU alta por disputa de z-order | Limite de reafirmações |
| 1 | Hook de `EVENT_OBJECT_LOCATIONCHANGE` sem filtro gera milhares de eventos | Filtrar por PID/TID do Explorer e pelo `HWND` da barra no callback |
| 2 | Alocação por quadro introduzida em código novo | Teste do alocador contador no CI |
| 2 | Painel mantendo buffers grandes após fechar | Liberar buffers do painel em `WM_DESTROY` ou ao ocultar |
| 2 | Save grande sendo serializado com frequência | Agrupar saves; estado pequeno |
| 3 | Regressão acumulada | Benchmarks com limite no CI; soak de 72 h |

## 4. Como medir

| Métrica | Ferramenta | Procedimento |
|---|---|---|
| Private Bytes, handles, GDI, USER | Overlay de debug / Process Explorer (colunas "Private Bytes", "GDI Objects", "USER Objects", "Handles") | Observar por 10 min parado e 10 min em combate |
| CPU | Process Explorer (coluna "CPU" com precisão de 2 casas) | Média de 10 min |
| Despertares | `powercfg /energy` (relatório de 60 s) | Verificar que o processo não aparece entre os que alteram a resolução do timer |
| Alocações por quadro | Teste com `#[global_allocator]` contador | CI |
| Tempo de `step`/render | Benchmarks (`cargo bench` com harness simples ou `criterion` em dev-dependency) | CI com limite |
| Tendência de longo prazo | CSV do overlay + regressão linear | Soak de 72 h |
| Tamanho do exe | Tamanho do arquivo | CI falha se > 3 MB |
