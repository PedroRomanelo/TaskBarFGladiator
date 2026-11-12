# 06 — Catálogo de casos de borda

Lista consolidada de situações incomuns que exigem tratamento explícito. Cada item tem: o que acontece,
por que é um problema, e o tratamento planejado. A coluna **Fase** indica quando deve ser implementado.

Ao corrigir um bug causado por um caso não listado, adicioná-lo aqui no mesmo PR.

---

## 1. Barra de tarefas e shell

| # | Situação | Problema | Tratamento | Fase |
|---|---|---|---|---|
| B1 | Usuário clica na barra | Barra sobe no z-order e cobre a faixa | Reafirmar `HWND_TOPMOST` com `SWP_NOACTIVATE` em `EVENT_SYSTEM_FOREGROUND` | 1 |
| B2 | Menu Iniciar/Pesquisa/flyout aberto | Reafirmar topmost colocaria a faixa por cima do menu | Não reafirmar quando o primeiro plano for classe de flyout do shell | 1 |
| B3 | Outro programa topmost reafirma em resposta | Laço infinito de `SetWindowPos`, CPU alta | Limite de 4 reafirmações/s; pausa de 30 s se persistir | 1 |
| B4 | Explorer reinicia | `HWND` da barra muda, hook aponta para PID antigo, ícone da bandeja some | Tratar `TaskbarCreated`: rebuscar barra, refazer hook, `NIM_ADD`, reposicionar | 1 |
| B5 | Explorer travado | Chamadas síncronas travam a UI do jogo | `IsHungAppWindow` antes; `SendMessageTimeoutW` com `SMTO_ABORTIFHUNG` | 1 |
| B6 | Explorer não está rodando / shell alternativo | Barra inexistente | Modo flutuante | 1 |
| B7 | Barra vertical ou superior | Layout horizontal não cabe | Modo flutuante na 1.0 | 1 |
| B8 | Auto-hide ligado | Barra recolhe para 2 px; faixa ficaria flutuando sozinha | Ocultar faixa quando altura visível < 24 px; seguir animação | 1 |
| B9 | Modo tablet do Windows 11 (2 em 1) | Altura da barra alterna entre ~24 e 48 px | Layout aceita qualquer altura de 24 a 120 px | 1 |
| B10 | Ícones centralizados no Windows 11 | Canto esquerdo ocupado pelo botão de Widgets | Margem padrão à esquerda + arrastar manual | 1 |
| B11 | "Barra em todas as telas" desligado | Não existe `Shell_SecondaryTrayWnd` | Usar só a barra principal | 1 |
| B12 | Barra de monitor secundário com DPI diferente | Escala errada | DPI por monitor (`GetDpiForMonitor`) | 1 |
| B13 | Muitos ícones na barra | Espaço livre insuficiente para a largura configurada | Versão compacta do layout; largura máxima de 40 % da barra | 1 |
| B14 | Atualização do Windows muda classes de janela | Detecção de flyouts/barra falha | Lista centralizada; fallback flutuante; template de issue | 1/3 |
| B15 | ExplorerPatcher, StartAllBack, Windhawk | Estrutura da barra diferente | Tratar como "barra desconhecida" se o retângulo for incoerente → flutuante | 1 |
| B16 | Áreas de trabalho virtuais | Faixa pode ficar só na área em que foi criada | Validar; reposicionar ao trocar de área de trabalho se necessário | 1 |
| B17 | Ícone da bandeja com `NIF_GUID` e exe movido de pasta | `NIM_ADD` falha (GUID associado ao caminho antigo) | Repetir `NIM_ADD` sem `NIF_GUID` | 1 |
| B18 | Menu de contexto da bandeja não fecha ao clicar fora | Comportamento documentado do Windows | `SetForegroundWindow` antes e `WM_NULL` depois de `TrackPopupMenu` | 1 |

## 2. Monitores, DPI e geometria

| # | Situação | Problema | Tratamento | Fase |
|---|---|---|---|---|
| M1 | DPI 125 % / 175 % | Escala não inteira deforma a pixel art | Fator inteiro por arredondamento para baixo + margem | 1 |
| M2 | Monitor com coordenadas negativas | Cálculos que assumem coordenadas positivas erram | Usar `i32` em todo cálculo de posição | 1 |
| M3 | Monitor desconectado com a faixa nele | Faixa fora da tela | `WM_DISPLAYCHANGE` → mover para a barra principal | 1 |
| M4 | Mudança de DPI em tempo de execução | Buffers e escala desatualizados | `WM_DPICHANGED` → recriar buffers | 1 |
| M5 | Conexão RDP | Resolução e DPI mudam | Mesmo tratamento de M3/M4 | 1 |
| M6 | Resolução muito pequena | Faixa não cabe | Layout compacto | 2 |
| M7 | Offset salvo para monitor que mudou de resolução | Posição inválida | Chave inclui a resolução; offset limitado à largura atual | 1 |

## 3. Sessão, energia e tempo

| # | Situação | Problema | Tratamento | Fase |
|---|---|---|---|---|
| S1 | Tela bloqueada / troca de usuário | Animação rodando sem ninguém ver | Pausar em `WTS_SESSION_LOCK`, retomar em `WTS_SESSION_UNLOCK` | 1 |
| S2 | Suspensão | Perda de progresso se a bateria acabar | Salvar em `PBT_APMSUSPEND` | 1/2 |
| S3 | Retomada após suspensão | `dt` gigante; monitores mudaram | Limitar `dt` a 100 ms; redetectar tudo | 1 |
| S4 | Desligamento do Windows | Processo encerrado logo após `WM_ENDSESSION` | Save síncrono em `WM_QUERYENDSESSION`/`WM_ENDSESSION` | 2 |
| S5 | Relógio do sistema alterado | Progresso offline (pós-1.0) exploraria o relógio | Usar UTC, ignorar delta negativo, limitar ganho | pós-1.0 |
| S6 | Processo morto pelo Gerenciador de Tarefas | Sem chance de salvar | Saves frequentes (início e fim de luta) + escrita atômica | 2 |
| S7 | Duas instâncias | Saves concorrentes corrompem o arquivo | Mutex de instância única | 1 |
| S8 | Troca rápida de usuário com o jogo aberto nos dois | Mutex global impediria o segundo usuário | Mutex com prefixo `Local\` (por sessão) | 1 |

## 4. Tela cheia e foco

| # | Situação | Problema | Tratamento | Fase |
|---|---|---|---|---|
| F1 | Jogo em tela cheia exclusiva | Faixa poderia aparecer por cima ou atrapalhar | `QUNS_RUNNING_D3D_FULL_SCREEN` → ocultar | 1 |
| F2 | Tela cheia sem borda (borderless) | Não aciona `QUNS_*` | Comparar retângulo do primeiro plano com o monitor | 1 |
| F3 | Vídeo entra em tela cheia sem trocar de janela | `EVENT_SYSTEM_FOREGROUND` não dispara | Observar `EVENT_OBJECT_LOCATIONCHANGE` da janela em primeiro plano | 1 |
| F4 | Apresentação (PowerPoint) | Faixa aparecendo durante apresentação | `QUNS_PRESENTATION_MODE` → ocultar | 1 |
| F5 | Desktop (`Progman`/`WorkerW`) cobre o monitor | Seria confundido com tela cheia | Excluir essas classes da verificação | 1 |
| F6 | Clique na faixa | Roubaria o foco do programa em uso | `WS_EX_NOACTIVATE` + `MA_NOACTIVATE` | 1 |
| F7 | Necessidade de teclado | Faixa nunca recebe teclado | Painel separado com foco normal | 2 |
| F8 | Atalho global já usado por outro programa | `RegisterHotKey` falha | Avisar e manter desativado | 2 |

## 5. Tema e aparência

| # | Situação | Problema | Tratamento | Fase |
|---|---|---|---|---|
| T1 | Tema claro × escuro | Texto ilegível em um dos dois | Contorno escuro de 1 px × escala em textos e barras | 1 |
| T2 | Troca de tema com o jogo aberto | Paleta antiga | `WM_SETTINGCHANGE` "ImmersiveColorSet" | 1 |
| T3 | Alto contraste | Paleta própria inadequada | Usar cores do sistema | 1 |
| T4 | Texto via GDI na janela layered | GDI escreve alpha 0 → texto invisível | Fonte bitmap própria | 2 |
| T5 | Pixels não premultiplicados | Halos claros, cores erradas | Converter atlas para premultiplicado ao carregar | 2 |
| T6 | Daltonismo | HP e fôlego confundidos | Diferenciar por posição e forma | 2 |

## 6. Arquivos, caminhos e save

| # | Situação | Problema | Tratamento | Fase |
|---|---|---|---|---|
| A1 | Nome de usuário com acento | Conversão para UTF-8 com perda gera caminho inválido | `PathBuf`/`OsString` do início ao fim | 2 |
| A2 | `%APPDATA%` ausente ou alterado | Save em lugar errado | `SHGetKnownFolderPath` | 2 |
| A3 | Save corrompido | Jogo não abre | `save.bak` → senão jogo novo preservando o arquivo ruim | 2 |
| A4 | Save gigante (malicioso ou erro) | Consumo de memória | Rejeitar > 1 MB antes de ler | 2 |
| A5 | Save editado manualmente | Valores absurdos | Validação com limites e log de correções | 2 |
| A6 | Save de versão mais nova | Campos desconhecidos | Ignorar com aviso | 2 |
| A7 | Save com item removido do catálogo | `ItemId` inexistente | Converter em ouro | 2 |
| A8 | Disco lento (antivírus, OneDrive, perfil em rede) | UI congela durante gravação | Gravação em thread separada | 2 |
| A9 | Disco cheio | Gravação falha | Manter save anterior intacto (escrita atômica); avisar no painel | 2 |
| A10 | Pasta somente leitura / sem permissão | Gravação falha | Avisar; continuar jogando sem salvar | 2 |
| A11 | Exe executado de pasta com DLLs maliciosas | Sequestro de DLL | `SetDefaultDllDirectories` | 1 |

## 7. Lógica do jogo

| # | Situação | Problema | Tratamento | Fase |
|---|---|---|---|---|
| G1 | Duas IAs defensivas | Luta infinita | Exaustão a partir do turno 40; fim no turno 60 | 2 |
| G2 | Nocaute simultâneo | Resultado ambíguo | Ordem de resolução fixa; sem contra-ataque se o defensor cair | 2 |
| G3 | Ação inválida (avançar encostado, recuar na parede) | Estado incoerente | Interface só oferece válidas; `game` rejeita com erro | 2 |
| G4 | Sem fôlego | Jogador sem ação | "Recuperar" custa 0 e é sempre válida | 2 |
| G5 | Overflow de ouro/XP/dano | Wrap silencioso em release | Tipos saturantes + `overflow-checks` | 2 |
| G6 | Fechar o jogo para refazer luta perdida | Exploração ("save scumming") | Semente gravada no início da luta | 2 |
| G7 | Build dominante | Jogo sem decisões | Metas do `sim` no CI | 2 |
| G8 | Nível máximo atingido | XP continua acumulando sem efeito | XP satura; interface mostra "nível máximo" | 2 |

## 8. Rust e Win32

| # | Situação | Problema | Tratamento | Fase |
|---|---|---|---|---|
| R1 | Reentrância do WndProc | Panic `already borrowed` | WndProc só enfileira eventos | 1 |
| R2 | Panic em `extern "system"` | Aborta o processo | Hook de panic grava log; `panic = "abort"` | 1 |
| R3 | Mensagens após `WM_NCDESTROY` | Uso de memória liberada | Zerar `GWLP_USERDATA` antes de liberar | 1 |
| R4 | `GetLastError` lido tarde | Código de erro errado | Ler imediatamente após a chamada | 1 |
| R5 | Buffer UTF-16 temporário liberado antes da chamada | Ponteiro inválido | Manter o `Vec<u16>` vivo até o fim da chamada | 1 |
| R6 | Objeto GDI apagado ainda selecionado | `DeleteObject` falha, vazamento | RAII restaura o objeto anterior antes de apagar | 1 |
| R7 | Crate chamado `core` | Conflito com a biblioteca `core` do Rust | Crate de lógica chama-se `game` | 0 |
