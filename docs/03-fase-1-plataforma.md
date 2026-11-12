# 03 — Fase 1 (Início): prova de conceito da plataforma

**Duração estimada:** 3 a 4 semanas (≈ 30–40 h).
**Princípio:** atacar primeiro o maior risco técnico. A lógica de um jogo por turnos é previsível; a
convivência estável com a barra de tarefas do Windows não é, e depende de comportamento não documentado do
Explorer.

**Entregável:** executável que desenha uma cena de teste (retângulo e sprite animado) na área da barra de
tarefas, com overlay de debug (quadros por segundo, Private Bytes, handles GDI/USER), ícone na bandeja com
menu, e que se mantém correto em toda a matriz descrita em [`09-verificacao-e-testes.md`](09-verificacao-e-testes.md).

---

## 1. Tarefas

### 1.1 Inicialização do processo (ordem importa)
1. `SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32)` — antes de qualquer carregamento dinâmico de
   DLL (ver [`07-seguranca.md`](07-seguranca.md), seção DLL hijacking).
2. Instalar o hook de panic (grava log de crash).
3. Inicializar o logger em arquivo.
4. Verificar instância única (mutex).
5. Ler config.
6. Registrar classe de janela, criar janela, ícone da bandeja, hooks.
7. Detectar a barra e posicionar.
8. Entrar no loop de mensagens.

### 1.2 Instância única
- `CreateMutexW(NULL, FALSE, "Local\\<NomeDoJogo>-<GUID>")`.
- Se `GetLastError() == ERROR_ALREADY_EXISTS`: localizar a janela existente com `FindWindowW` pela classe
  própria, enviar uma mensagem registrada (`RegisterWindowMessageW("<NomeDoJogo>.Show")`) para ela se
  mostrar, e sair com código 0.
- Prefixo `Local\` = escopo da sessão do usuário. Dois usuários logados ao mesmo tempo (troca rápida de
  usuário) podem cada um ter sua instância.

### 1.3 Janela principal (faixa)
- `RegisterClassExW` com `hCursor = IDC_ARROW`, sem `hbrBackground`, nome de classe prefixado pelo nome do jogo.
- `CreateWindowExW` com:
  - estilo: `WS_POPUP`
  - estendidos: `WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE`
- Efeito de cada estilo:

| Estilo | Efeito |
|---|---|
| `WS_POPUP` | sem borda nem barra de título |
| `WS_EX_LAYERED` | permite transparência por pixel via `UpdateLayeredWindow` |
| `WS_EX_TOOLWINDOW` | não aparece na barra de tarefas nem no Alt+Tab |
| `WS_EX_TOPMOST` | fica acima de janelas comuns |
| `WS_EX_NOACTIVATE` | clicar não ativa a janela nem rouba o foco |

- `WM_MOUSEACTIVATE` → retornar `MA_NOACTIVATE` (reforça o `WS_EX_NOACTIVATE` em cliques).
- `WM_NCHITTEST`: retornar `HTCLIENT` nas áreas clicáveis; áreas totalmente transparentes já deixam o
  clique passar.

### 1.4 Estado associado à janela
- Na criação: `Box::into_raw(Box::new(WindowState { .. }))` salvo com
  `SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr)` durante `WM_NCCREATE` (via `lpCreateParams`).
- No WndProc: ler o ponteiro; se nulo (mensagens antes de `WM_NCCREATE`), chamar `DefWindowProcW`.
- Em `WM_NCDESTROY`: zerar `GWLP_USERDATA` **antes** de `Box::from_raw` e liberar. Mensagens posteriores
  encontram ponteiro nulo.

### 1.5 Localização da barra de tarefas
1. `FindWindowW("Shell_TrayWnd", NULL)` → barra principal.
2. `EnumWindows` filtrando classe `"Shell_SecondaryTrayWnd"` → barras dos outros monitores (existem apenas
   se "Mostrar a barra de tarefas em todas as telas" estiver ativado).
3. `SHAppBarMessage(ABM_GETTASKBARPOS, &abd)` → `abd.rc` (retângulo em pixels físicos) e `abd.uEdge`
   (`ABE_BOTTOM`, `ABE_TOP`, `ABE_LEFT`, `ABE_RIGHT`).
4. `SHAppBarMessage(ABM_GETSTATE, ..)` → bit `ABS_AUTOHIDE`.
5. `MonitorFromWindow(hwndTaskbar, MONITOR_DEFAULTTONEAREST)` + `GetMonitorInfoW` → retângulo do monitor.
6. `GetDpiForMonitor(hmon, MDT_EFFECTIVE_DPI, ..)` → DPI (96 = 100 %).
7. Decisão:
   - borda inferior → modo `NaBarra`;
   - qualquer outra borda, barra não encontrada, ou retângulo inválido → modo `Flutuante`.

### 1.6 Cálculo da posição
- Altura da faixa = altura da barra (`rc.bottom - rc.top`).
- Largura da faixa = largura lógica configurada (padrão 360 px lógicos) × DPI / 96, limitada a 40 % da
  largura da barra.
- Posição horizontal: offset salvo para este monitor; se não houver, padrão:
  - Windows 11 com ícones centralizados: à esquerda, após uma margem que deixa livre o botão de Widgets
    (padrão ~ 160 px lógicos; ajustável arrastando);
  - Windows 11 com ícones à esquerda / Windows 10: logo à esquerda da área de notificação, calculada com
    `FindWindowExW(hwndTaskbar, NULL, "TrayNotifyWnd", NULL)` + `GetWindowRect` quando a janela existir.
- Arrastar: botão esquerdo pressionado por > 300 ms sobre a faixa inicia arraste horizontal (com
  `SetCapture`); ao soltar, salva o offset relativo à borda esquerda da barra, em px lógicos, com chave
  `"<nome do dispositivo do monitor>@<largura>x<altura>"` (de `MONITORINFOEXW.szDevice`).

### 1.7 Rastreamento de mudanças

| Fonte | O que detecta | Ação |
|---|---|---|
| `SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE, .., pidExplorer, tidTaskbar, WINEVENT_OUTOFCONTEXT)` | barra mudou de tamanho/posição, animação de auto-hide | recalcular posição (com *debounce* de 1 quadro) |
| `SetWinEventHook(EVENT_SYSTEM_FOREGROUND, .., 0, 0, WINEVENT_OUTOFCONTEXT)` | troca de janela em primeiro plano | reavaliar z-order e tela cheia |
| `RegisterWindowMessageW("TaskbarCreated")` | Explorer reiniciou | refazer tudo (ver 2.2) |
| `WM_DISPLAYCHANGE` | resolução/monitores mudaram | redetectar |
| `WM_DPICHANGED` | DPI do monitor da janela mudou | redimensionar buffers, re-escalar |
| `WM_SETTINGCHANGE` com `lParam` = `"TraySettings"` | configurações da barra mudaram | redetectar |
| `WM_SETTINGCHANGE` com `lParam` = `"ImmersiveColorSet"` | tema claro/escuro | trocar paleta |
| `WTSRegisterSessionNotification` → `WM_WTSSESSION_CHANGE` | bloqueio, desbloqueio, RDP | pausar/retomar |
| `WM_POWERBROADCAST` (`PBT_APMSUSPEND`, `PBT_APMRESUMEAUTOMATIC`) | suspensão/retomada | salvar antes; redetectar depois |

`WINEVENT_OUTOFCONTEXT` faz o Windows entregar os eventos como mensagens para a thread do jogo, sem
carregar DLL em outro processo. É a forma segura (sem injeção) e não exige privilégio.

### 1.8 Tela cheia
- `SHQueryUserNotificationState(&state)`: ocultar se `QUNS_BUSY`, `QUNS_RUNNING_D3D_FULL_SCREEN` ou
  `QUNS_PRESENTATION_MODE`.
- Verificação adicional (jogos e players em "tela cheia sem borda" nem sempre acionam o estado acima):
  janela em primeiro plano (`GetForegroundWindow`) cujo `GetWindowRect` cobre todo o retângulo do monitor
  da faixa **e** cuja classe não é `Progman` nem `WorkerW` (desktop) **e** que não é a própria barra.
- Reavaliar em `EVENT_SYSTEM_FOREGROUND` e em `EVENT_OBJECT_LOCATIONCHANGE` da janela em primeiro plano
  (para pegar o momento em que um vídeo entra em tela cheia sem mudar de janela).

### 1.9 Ícone na bandeja
- `Shell_NotifyIconW(NIM_ADD)` com `NOTIFYICONDATAW { uFlags: NIF_ICON | NIF_TIP | NIF_MESSAGE | NIF_GUID }`
  e `uVersion = NOTIFYICON_VERSION_4` via `NIM_SETVERSION`.
- `NIF_GUID` com GUID fixo do jogo: o Windows lembra a preferência do usuário (mostrar/ocultar na bandeja)
  entre execuções. **Atenção:** o GUID fica associado ao caminho do executável; mover o exe para outra
  pasta faz o `NIM_ADD` falhar com GUID já registrado em outro caminho. Tratamento: se `NIM_ADD` falhar,
  tentar novamente sem `NIF_GUID`.
- Menu (botão direito): Mostrar/Ocultar faixa, Pausar, Reposicionar (volta ao padrão), Painel, Sair.
- Sequência obrigatória para o menu fechar corretamente ao clicar fora: `SetForegroundWindow(hwnd)` →
  `TrackPopupMenu(..)` → `PostMessageW(hwnd, WM_NULL, 0, 0)`.

### 1.10 Modo flutuante (fallback)
- Mesma janela, posicionada no canto inferior direito da área de trabalho (`SPI_GETWORKAREA` do monitor
  primário), com margem de 8 px lógicos, arrastável em ambos os eixos.
- Ativado quando: barra não encontrada, barra vertical ou superior, retângulo inválido, ou por escolha do usuário.
- A cada `TaskbarCreated`/`WM_DISPLAYCHANGE`, tentar voltar para o modo `NaBarra`.

### 1.11 Log e crash
- Logger próprio: arquivo `%LOCALAPPDATA%\<Jogo>\logs\<Jogo>.log`, rotação ao atingir 1 MB, mantém 3 arquivos.
- Níveis: `error`, `warn`, `info` em release; `debug`/`trace` com variável de ambiente ou opção.
- Hook de panic: grava mensagem, local (`file:line`), versão do jogo, versão do Windows
  (`RtlGetVersion`) em `crash-<timestamp>.txt`.
- Conteúdo de log sem dados pessoais (não registrar nome de usuário nem caminhos completos com o nome do usuário;
  substituir o prefixo do perfil por `%USERPROFILE%`).

### 1.12 Overlay de debug (feature `debug-overlay`)
- Mostra: quadros por segundo, tempo do último quadro em µs, Private Bytes (`GetProcessMemoryInfo` com
  `PROCESS_MEMORY_COUNTERS_EX.PrivateUsage`), `GetGuiResources(GR_GDIOBJECTS)`,
  `GetGuiResources(GR_USEROBJECTS)`, contagem de handles (`GetProcessHandleCount`).
- Opcional: gravar esses valores a cada minuto em CSV (usado no soak test).

---

## 2. Casos de borda não óbvios desta fase

### 2.1 Z-order
1. **A barra cobre a faixa ao ser clicada.** A barra também é topmost; quando recebe foco, o Explorer a
   coloca no topo do grupo topmost. Ação: em `EVENT_SYSTEM_FOREGROUND` (e quando a barra for clicada),
   chamar `SetWindowPos(hwnd, HWND_TOPMOST, 0,0,0,0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE)`.
2. **Não ficar por cima de flyouts do shell.** Menu Iniciar, Pesquisa, central de notificações, calendário,
   overflow da bandeja, Widgets, Task View e Alt+Tab devem aparecer por cima da faixa. Se o primeiro plano
   for um desses, **não** reafirmar topmost. Identificação por classe da janela em primeiro plano; lista
   inicial (a validar em cada versão do Windows):
   - `Windows.UI.Core.CoreWindow` (Iniciar/Pesquisa em várias versões)
   - `XamlExplorerHostIslandWindow` (vários flyouts do Windows 11)
   - `TopLevelWindowForOverflowXamlIsland` (overflow da bandeja no Windows 11)
   - `NotifyIconOverflowWindow` (overflow da bandeja no Windows 10)
   - `MultitaskingViewFrame`, `ForegroundStaging` (Alt+Tab/Task View)
   A lista fica em `platform/src/shell_classes.rs` (um único lugar para atualizar).
3. **Disputa com outro programa topmost** (outro widget de barra, overlays de jogos, gravadores de tela).
   Ambos reafirmam topmost em resposta um ao outro → laço infinito consumindo CPU. Ação: limitar a
   reafirmação a 4 por segundo; se o limite for atingido por 5 s seguidos, parar por 30 s e registrar aviso no log.
4. **Win+D / "Mostrar área de trabalho".** Janelas topmost não são minimizadas, mas validar em cada versão.
5. **Win+Tab / áreas de trabalho virtuais.** Janelas `WS_EX_TOOLWINDOW` aparecem em todas as áreas de
   trabalho? Comportamento: a janela pertence à área de trabalho em que foi criada, exceto se marcada para
   todas. Validar; se necessário, reagir ao evento de troca de área de trabalho reposicionando a janela
   (a API pública `IVirtualDesktopManager` permite consultar, não fixar em todas; avaliar na implementação).

### 2.2 Explorer travado ou reiniciado
1. `SHAppBarMessage` e `SendMessage` para janelas do Explorer são **síncronos**. Se o Explorer estiver
   travado, a thread do jogo trava junto e a faixa congela. Ações:
   - para mensagens diretas, usar `SendMessageTimeoutW(.., SMTO_ABORTIFHUNG | SMTO_BLOCK, 200, ..)`;
   - antes de chamar `SHAppBarMessage`, testar `IsHungAppWindow(hwndTaskbar)`; se travado, adiar e tentar
     de novo em 1 s.
2. Após `TaskbarCreated`:
   - o `HWND` da barra é outro → buscar de novo;
   - o hook de `EVENT_OBJECT_LOCATIONCHANGE` foi registrado com o PID/TID antigos → `UnhookWinEvent` e
     registrar de novo;
   - o ícone da bandeja sumiu → `NIM_ADD` de novo;
   - recalcular posição.
3. Explorer encerrado e não reiniciado (raro, mas acontece em ambientes corporativos ou com shells
   alternativos): modo `Flutuante`.

### 2.3 Geometria
1. Alturas reais da barra: ~40 px (Windows 10, 100 %), 48 px (Windows 11, 100 %), ~30 px (ícones pequenos
   no Windows 10); proporcionais ao DPI (125 % → 60 px no Windows 11). No modo tablet do Windows 11 em
   dispositivos 2 em 1, a barra alterna dinamicamente entre recolhida (~24 px) e expandida.
   A renderização aceita qualquer altura de 24 a 120 px sem quebrar o layout.
2. Windows 11 com ícones centralizados: o botão de Widgets/clima ocupa o canto esquerdo por padrão.
3. Barra do monitor secundário pode ter DPI diferente da principal.
4. Monitor removido com a faixa nele (notebook desconectado da dock): `WM_DISPLAYCHANGE` → a chave de
   offset do monitor não existe mais → mover para a barra principal com posição padrão.
5. Retângulo da barra com coordenadas negativas (monitor à esquerda ou acima do principal) é válido;
   não assumir coordenadas positivas.
6. Auto-hide: quando recolhida, a barra fica com 2 px visíveis. Se a altura visível for < 24 px, ocultar
   a faixa; mostrar de novo quando a barra expandir (seguindo `EVENT_OBJECT_LOCATIONCHANGE`).
7. Resolução muito pequena (ex.: 1024×768 a 150 %): a largura máxima de 40 % da barra limita a faixa;
   o layout precisa de uma versão compacta (só HP e lutadores).

### 2.4 Sessão, energia e área remota
1. Tela bloqueada, troca rápida de usuário, sessão RDP desconectada: pausar animações e timers.
2. Conexão RDP: resolução e DPI mudam enquanto o jogo roda → tratado por `WM_DISPLAYCHANGE`/`WM_DPICHANGED`.
3. Suspensão/hibernação: em `PBT_APMSUSPEND` salvar o jogo; em `PBT_APMRESUMEAUTOMATIC` redetectar tudo
   (monitores podem ter mudado) e limitar o `dt` da animação.
4. Modo de economia de bateria: nenhuma mudança obrigatória, mas animações podem ser reduzidas pela opção
   "reduzir animações".

### 2.5 Tema e aparência
1. Tema claro/escuro e transparência da barra mudam em tempo de execução (`ImmersiveColorSet`).
   Fundo da faixa transparente; textos, números e barras de HP com contorno escuro de 1 px (×escala) para
   legibilidade em fundo claro e escuro.
2. Ler o tema em `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`, valor
   `SystemUsesLightTheme` (a barra segue o tema do sistema, não o dos aplicativos).
3. Alto contraste: `SystemParametersInfoW(SPI_GETHIGHCONTRAST)` → usar `GetSysColor(COLOR_WINDOWTEXT)`
   e `COLOR_WINDOW` em vez da paleta própria.

### 2.6 Rust e Win32
1. **Reentrância do WndProc** — ver [`01-arquitetura.md`](01-arquitetura.md), seção 5.2.
2. **Panic dentro de `extern "system" fn`**: a partir do Rust 1.81 isso aborta o processo. Com
   `panic = "abort"` o comportamento já é abortar; o hook de panic grava o log antes.
3. **Callbacks de WinEvent** também são `extern "system"`: mesma regra — só enfileirar.
4. **Strings**: todas as APIs "W" recebem UTF-16 terminado em zero. Criar helper `to_wide(&str) -> Vec<u16>`
   e nunca passar ponteiro de buffer temporário que é liberado antes da chamada.
5. **`GetLastError`** deve ser lido imediatamente após a chamada que falhou; qualquer outra chamada no meio
   (inclusive alocação que chame o sistema) pode sobrescrever o valor.
6. **Tipos de 64 bits**: `SetWindowLongPtrW`/`GetWindowLongPtrW` (não as versões sem `Ptr`).

### 2.7 Memória nesta fase
1. Cada objeto GDI selecionado em um DC deve ser desselecionado (restaurando o objeto anterior) antes de
   ser apagado; senão `DeleteObject` falha silenciosamente e o objeto vaza.
2. O limite padrão é 10.000 objetos GDI por processo. Um vazamento de 1 objeto por quadro a 30 quadros/s
   esgota o limite em ~6 minutos de animação; um vazamento por evento raro só aparece após dias.
3. DIB section e buffer de renderização: criados uma vez; recriados apenas quando o tamanho ou DPI mudam.
4. `GetDC(NULL)` precisa de `ReleaseDC` em toda saída da função (RAII resolve, inclusive em `?`).

---

## 3. Preocupações da fase

| Preocupação | Medida |
|---|---|
| Comportamento do Explorer não documentado e diferente por versão | Matriz de testes por versão; lista de classes centralizada; fallback flutuante |
| Testes automáticos limitados (runner de CI sem barra real) | Matriz manual obrigatória; smoke test valida o fallback |
| Gastar tempo demais em perfeição de posicionamento | Arrastar manual resolve o caso geral; detecção automática avançada é pós-1.0 |
| Vazamento de recursos | RAII para todo recurso; overlay de debug; teste de 24 h nesta fase |

## 4. Critérios de aceite

- [ ] Faixa aparece na barra em Windows 10 22H2 e Windows 11 23H2/24H2+.
- [ ] Matriz manual de plataforma 100 % aprovada.
- [ ] Reinício do Explorer (`taskkill /f /im explorer.exe` seguido de `start explorer.exe`): faixa
      reposicionada e ícone da bandeja de volta em < 2 s.
- [ ] Menu Iniciar, Pesquisa e flyouts aparecem por cima da faixa.
- [ ] Tela cheia (vídeo no navegador, jogo) oculta a faixa; sair da tela cheia mostra de novo.
- [ ] 24 h com animação de teste contínua: Private Bytes, handles, GDI e USER sem crescimento (≤ 1 %).
- [ ] CPU 0,0 % com animação desligada.
