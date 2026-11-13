# 10 — Cronograma e registro de decisões

## 1. Cronograma

Estimativa para uma pessoa com ~10 horas por semana e experiência prévia em programação. Rust e Win32
novos para a pessoa podem aumentar a Fase 1 em 50 % a 100 %.

| Fase | Duração | Horas | Marco de saída |
|---|---|---|---|
| 0 — Preparação | 1 semana | ~10 h | CI verde com workspace vazio; nome escolhido |
| 1 — Plataforma | 3–4 semanas | ~30–40 h | Faixa estável na barra em toda a matriz; 24 h sem vazamento |
| 2 — Jogo | 6–10 semanas | ~60–100 h | Liga 1 completa, modos manual e automático, metas de balanceamento |
| 3 — Polimento e distribuição | 3–5 semanas | ~30–50 h | Release 1.0 com hash, atestação e (se aprovada) assinatura |
| **Total** | **~13–20 semanas** | **~130–200 h** | **Versão 1.0** |

### 1.1 Marcos intermediários (versões de pré-lançamento)

| Versão | Conteúdo | Fim da |
|---|---|---|
| `0.1.0` | Faixa de teste na barra (sem jogo) | Fase 1 |
| `0.2.0` | Combate jogável com placeholders, sem loja | metade da Fase 2 |
| `0.3.0` | Jogo completo com arte provisória | Fase 2 |
| `0.9.0` | Candidata a lançamento, para testes de terceiros | metade da Fase 3 |
| `1.0.0` | Lançamento | Fase 3 |

### 1.2 Caminho crítico
1. Detecção e posicionamento na barra (Fase 1) — bloqueia tudo que é visual.
2. Fórmulas de combate + `sim` (Fase 2.1/2.5) — bloqueiam balanceamento e conteúdo.
3. Aprovação da assinatura de código — processo externo; solicitar cedo (Fase 0/1).

## 2. Decisões tomadas

Formato: **ID — decisão** · contexto · alternativas consideradas · motivo.

**D1 — Linguagem Rust.**
Escolha do autor. Alternativas: C++ (mais leve, menos seguro em memória), C# com NativeAOT (mais memória).
Motivo: segurança de memória sem coletor de lixo, binário único, desempenho nativo.

**D2 — Jogo original, sem material de terceiros.**
Contexto: inspiração em RPGs de gladiador de navegador. Alternativa (recriação de jogo existente) descartada
por risco jurídico e por impedir distribuição pública segura. Motivo: mecânicas de gênero são livres;
nome, arte, textos e código não.

**D3 — Janela sobreposta à barra (`WS_EX_LAYERED` + `TOPMOST` + `NOACTIVATE`).**
Alternativas: DeskBand (removido no Windows 11), AppBar própria (ocupa espaço da tela), injeção no Explorer
(instável, sinalizada por antivírus, contradiz requisitos de segurança). Motivo: única opção sem injeção que
fica visualmente dentro da barra.

**D4 — Renderização por software + `UpdateLayeredWindow`.**
Alternativas: Direct2D/Direct3D (memória do driver, perda de dispositivo), GDI (sem alpha). Motivo: menor
memória, testável no Linux, simples.

**D5 — `windows-sys` em vez de `windows`.**
Motivo: compilação mais rápida, binário menor; a camada segura é própria e pequena.

**D6 — Lógica em aritmética inteira com RNG próprio.**
Motivo: determinismo entre máquinas, replays, testes reproduzíveis, proteção contra "save scumming".

**D7 — Crate de lógica chamado `game`.**
Motivo: o nome `core` conflita com a biblioteca `core` do Rust.

**D8 — Suporte a Windows 10 22H2 e Windows 11.**
Motivo: base de usuários do Windows 10 ainda grande. Revisar a cada versão maior.

**D9 — Barra vertical/superior usa modo flutuante na 1.0.**
Motivo: caso minoritário; layout vertical dobraria o trabalho de interface.

**D10 — Modos manual e automático na 1.0; progresso offline depois.**
Motivo: o modo automático é o que torna o jogo adequado para a barra; o progresso offline traz casos de
borda de relógio que podem esperar.

**D11 — Sem rede, sem telemetria, sem atualizador automático.**
Motivo: superfície de ataque mínima e privacidade.

**D12 — Sem áudio por padrão.**
Motivo: uso em ambiente de trabalho; economia de memória e energia.

**D13 — Faixa somente com mouse; teclado só no painel.**
Motivo: a faixa não pode roubar o foco do trabalho do usuário.

**D14 — Distribuição em ZIP portátil, sem instalador.**
Motivo: menos superfície, desinstalação simples.

**D15 — Licença `MIT OR Apache-2.0` para código; CC-BY-4.0/CC0 para arte própria.**
Motivo: padrão do ecossistema Rust; permite uso amplo por terceiros.

**D16 — Derrota não é permanente.**
Motivo: uso casual e prolongado; perder progresso por uma luta automática frustraria. Ver GDD.

**D17 — Sem engenharia reversa de outro jogo; balanceamento de referência próprio e congelado.**
Contexto: o autor quer regras estáveis até o fim da 1.0 para só depois alterar, sem risco de desbalancear
antes. Alternativa considerada: descompilar um jogo existente e copiar as regras exatas — descartada por ser
recriação de jogo específico, por risco jurídico e porque as fórmulas extraídas não seriam verificáveis.
Decisão: metas numéricas objetivas + baseline do `sim` comparado por igualdade exata no CI + replays +
tag `balance-v1.0`. Detalhes em [`12-balanceamento-de-referencia.md`](12-balanceamento-de-referencia.md).

**D18 — Metas de ritmo propostas pelo Claude, revisadas pelo autor.**
Contexto: o autor não indicou preferência sobre quem define as metas. As metas iniciais estão no documento 12,
seção 3, e ficam pendentes de revisão (P8) antes do início da Fase 2.

**D19 — Rust 1.97.0, edição 2024; nome provisório isolado.**
Resolve P4. A versão fica fixada em `rust-toolchain.toml`. Enquanto P1 não for decidida, o nome
provisório "TaskBarFGladiator" (binário `taskbar-gladiator`) fica concentrado em `app/src/identity.rs`.

**D20 — Ordem de implementação adaptada ao ambiente de desenvolvimento.**
Contexto: o desenvolvimento assistido acontece em um container Linux, onde o executável Windows não pode
ser executado. Decisão: implementar primeiro o que é totalmente verificável (`game`, `sim`, `render`), e a
plataforma depois, com compilação verificada para `x86_64-pc-windows-msvc`. A execução e a matriz manual da
Fase 1 ficam com o autor, no Windows. O princípio "risco primeiro" continua valendo para a validação:
a plataforma deve ser testada manualmente assim que a primeira versão compilar.

**D21 — Conteúdo e regras orientados a dados.**
Itens, ligas, oponentes e constantes de regras ficam em arquivos JSON em `crates/game/content/`, embutidos
no executável e validados na carga. Permite alterar o conteúdo sem mexer no motor. A versão do balanceamento é
derivada do hash desses arquivos. Formato em [`14-formato-do-conteudo.md`](14-formato-do-conteudo.md).

## 3. Decisões pendentes

| ID | Decisão | Prazo | Responsável |
|---|---|---|---|
| P1 | Nome definitivo do jogo | Fase 0 | Autor |
| P2 | Estilo de arte (paleta, proporção dos sprites) | Início da Fase 2 | Autor |
| P3 | Serviço de assinatura de código (SignPath ou outro) | Fase 0/1 (solicitar) | Autor |
| ~~P4~~ | ~~Edição do Rust e versão fixada~~ — resolvida em D19 | — | — |
| P5 | Detecção automática do espaço livre via UI Automation | Pós-1.0 | — |
| P6 | Suporte nativo a barra vertical | Pós-1.0 | — |
| P7 | Progresso offline e seus limites | Pós-1.0 | — |
| P8 | Revisar e aprovar as metas de balanceamento (doc 12, seção 3), principalmente P6 (duração da liga 1) | Antes da Fase 2 | Autor |

## 4. Como registrar uma nova decisão

Adicionar na seção 2 com o próximo ID (`D22`, `D23`...), contendo: decisão, contexto, alternativas
consideradas e motivo. Se a decisão substituir outra, marcar a antiga como "substituída por Dxx" em vez de
apagá-la.
