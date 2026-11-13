# 04 — Fase 2 (Meio): núcleo do jogo, save, renderização e interface

**Duração estimada:** 6 a 10 semanas (≈ 60–100 h).
**Objetivo:** jogo completo e jogável do primeiro nível até o campeão da liga 1, nos modos manual e
automático, rodando sobre a plataforma da Fase 1, dentro das metas de memória e CPU.

As regras do jogo (atributos, ações, fórmulas) estão em [`11-design-do-jogo.md`](11-design-do-jogo.md).
Este documento trata da implementação.

Ordem sugerida dentro da fase: 2.1 → 2.2 → 2.5 (permite balancear cedo, sem gráficos) → 2.3 → 2.4.

---

## 2.1 Crate `game` — lógica do jogo

> O crate se chama `game` (não `core`): um crate chamado `core` conflita com a biblioteca `core` embutida
> do Rust e causa erros confusos de resolução de caminho.

### Tarefas
1. **Ponto fixo** (`fixed.rs`): tipo `Fx(i32)` com 1/1000 de resolução (ex.: `Fx(1500)` = 1,5). Operações
   `mul`, `div` com arredondamento definido e saturação.
2. **RNG** (`rng.rs`): PCG32 (estado `u64` + incremento `u64`), ~20 linhas, `Serialize`/`Deserialize`.
   Funções: `next_u32`, `range(lo, hi)` sem viés (rejeição), `chance(per_mille)`.
3. **Tipos de domínio** com saturação: `Gold(u32)`, `Xp(u32)`, `Hp(u16)`, `Stamina(u16)`, `Level(u8)`,
   `Attr(u8)`; operações via `saturating_add`/`saturating_sub`.
4. **Estado** (`state.rs`):
   ```rust
   pub struct GameState {
       pub version: u32,
       pub player: Fighter,
       pub inventory: Inventory,
       pub gold: Gold,
       pub league: LeagueProgress,
       pub screen: Screen,          // Hub, PreFight, Fight(FightState), PostFight, Shop, ...
       pub settings_game: GameSettings,
       pub rng: Pcg32,
   }
   ```
5. **Combate** (`combat.rs`): `FightState` com posições, HP, fôlego, efeitos ativos, número do turno,
   semente da luta. Resolução de ação com as fórmulas do GDD.
6. **IA** (`ai.rs`): função pura `choose_action(&FightState, side, &Archetype, &mut Rng) -> Action`.
   Arquétipos: agressivo, defensivo, oportunista (detalhes no GDD). A mesma função controla o jogador no
   modo automático, com a estratégia escolhida.
7. **Itens e loja** (`items.rs`): catálogo estático (`const ITEMS: &[ItemDef]`), identificadores estáveis
   (`ItemId(u16)` que nunca são reutilizados, pois aparecem no save).
8. **Progressão** (`progression.rs`): XP por luta, curva de nível, pontos de atributo, recompensas em ouro,
   tabela de oponentes da liga.
9. **API pública**:
   ```rust
   pub fn step(state: &mut GameState, input: Input) -> StepResult;
   pub struct StepResult { pub events: SmallVec<[GameEvent; 8]>, pub save_hint: SaveHint }
   ```
   - `Input`: ações do jogador (atacar, mover, comprar, distribuir ponto, iniciar luta, alternar modo).
   - `GameEvent`: o que aconteceu (golpe acertou/errou, dano, crítico, nocaute, subiu de nível...),
     consumido pela camada de animação e textos.
   - `SaveHint`: `None`, `Soon` ou `Now` (indica ao `app` quando salvar).
   - O crate não lê relógio, não acessa disco e não conhece animação.

### Casos de borda (lógica)
1. **Luta infinita.** Duas IAs defensivas podem recuar e defender indefinidamente. Regras (detalhadas no
   GDD): limite de 60 turnos; a partir do turno 40, fôlego máximo dos dois cai 5 % por turno ("exaustão");
   no turno 60, vence quem tiver maior percentual de HP; empate exato → decide quem causou mais dano total;
   persistindo, vitória do desafiante (o jogador) para não frustrar. Teste de propriedade: toda luta termina
   em ≤ 60 turnos.
2. **Nocaute simultâneo** (ex.: dano de contra-ataque). Ordem de resolução fixa: ação do atacante resolve
   por completo antes do contra-ataque; se o atacante zera o HP do defensor, não há contra-ataque.
3. **Bordas da arena.** "Avançar" com o oponente na casa adjacente é inválido; "recuar" encostado na parede
   é inválido. A interface só oferece ações válidas; o `game` rejeita inválidas com erro (nunca panic).
4. **Sempre existe ação válida.** "Recuperar fôlego" custa 0 e é sempre válida.
5. **Overflow.** Ouro, XP e dano saturam no máximo do tipo. Dano mínimo de um acerto = 1.
6. **"Save scumming".** A semente da luta é gerada e gravada no **início** da luta, junto com o estado.
   Fechar o jogo no meio e reabrir reproduz exatamente a mesma sequência aleatória.
7. **IDs de item removidos em versões futuras.** O save pode conter `ItemId` que não existe mais: a
   validação converte em ouro equivalente ao preço de venda e registra aviso.
8. **Valores de atributos fora dos limites** após edição manual do save: limitados na validação (2.2).

### Testes
- Unitários por fórmula (valores esperados calculados à mão para casos simples).
- `proptest`:
  - para qualquer par de lutadores válidos e qualquer semente, a luta termina em ≤ 60 turnos;
  - HP nunca negativo, fôlego nunca acima do máximo;
  - mesma semente + mesmas ações = mesmo resultado (determinismo);
  - `serialize → deserialize` do estado é identidade.

---

## 2.2 Save e configurações

### Localização
- Pasta: `SHGetKnownFolderPath(FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, NULL, ..)` + `\<Jogo>\`.
  Não usar a variável de ambiente `%APPDATA%` (pode estar ausente em processos iniciados por agendadores
  ou ter sido alterada).
- Arquivos: `save.json`, `save.bak`, `config.json`.
- Logs e crash dumps ficam em `%LOCALAPPDATA%` (não devem sincronizar em perfis itinerantes).

### Caminhos com caracteres não ASCII
- Usuários com acento no nome (ex.: `C:\Users\João\...`) são comuns no Brasil.
- Manter caminhos como `PathBuf`/`OsString` (UTF-16 internamente no Windows) do início ao fim.
- Nunca usar `to_string_lossy()` para montar caminhos; usar apenas para exibir no log.

### Formato
```json
{
  "format": "<jogo>-save",
  "version": 1,
  "created_utc": "2026-01-01T12:00:00Z",
  "game": { "...": "GameState serializado" }
}
```
- `version` incrementa a cada mudança de esquema. Migração: `fn migrate(value: serde_json::Value, from: u32) -> Result<Value>`
  aplicando `v1→v2→v3...` em sequência. Cada migração é uma função pura com teste próprio usando um save
  real da versão anterior guardado em `crates/game/tests/fixtures/`.
- Campos desconhecidos são ignorados (permite abrir save de versão mais nova com perda mínima, com aviso).
- Campos ausentes usam `#[serde(default)]`.

### Leitura defensiva
1. Obter o tamanho do arquivo; se > 1 MB, rejeitar sem ler (um save legítimo tem poucos KB).
2. Desserializar.
3. Validar e corrigir: atributos dentro de [1, 99], nível dentro de [1, 50], HP/fôlego atuais ≤ máximos,
   índices de oponente dentro da tabela, `ItemId` existentes, posição na arena dentro de [0, N-1].
4. Qualquer correção é registrada no log como aviso.

### Escrita atômica
1. Serializar para `Vec<u8>` na thread de UI (rápido, poucos KB).
2. Enviar para a thread de I/O por canal (`std::sync::mpsc`), que:
   - grava `save.json.tmp`;
   - chama `File::sync_all()` (`FlushFileBuffers`);
   - chama `ReplaceFileW(save.json, save.json.tmp, save.bak, REPLACEFILE_IGNORE_MERGE_ERRORS, ..)`;
     na primeira gravação (sem `save.json`), usa `MoveFileExW(.., MOVEFILE_WRITE_THROUGH)`.
3. Se a fila tiver vários saves pendentes, gravar apenas o mais recente.

Motivo da thread separada: antivírus em tempo real, OneDrive/sincronização e perfis itinerantes
corporativos podem bloquear operações de arquivo por centenas de milissegundos ou segundos; a faixa não
pode congelar.

### Recuperação
- `save.json` inválido → tentar `save.bak` → se também inválido, renomear os dois para
  `save.corrupt-<timestamp>.json` (preserva para análise) e iniciar jogo novo, avisando no painel.

### Momentos de gravação
| Momento | Tipo |
|---|---|
| Início de cada luta (estado + semente) | assíncrono |
| Fim de cada luta | assíncrono |
| Compra/venda, distribuição de pontos | assíncrono, agrupado (até 2 s) |
| Mudança de configuração | assíncrono |
| `PBT_APMSUSPEND` | síncrono |
| `WM_QUERYENDSESSION` / `WM_ENDSESSION` | **síncrono** — o Windows pode encerrar o processo logo após o retorno |
| Sair pelo menu | síncrono, aguardando a fila de I/O esvaziar |

### Fuzzing
- `cargo-fuzz` com alvo `load_save(&[u8])` (no Linux, já que o crate `game` é portável).
- Propriedades: nenhuma entrada causa panic, laço infinito ou alocação acima de 16 MB.

---

## 2.3 Crates `render` e `assets`

### Tarefas
1. `Surface`: largura, altura, `Vec<u32>`; `clear(color)`; acesso por linha (`row_mut(y)`).
2. `blit(sprite, x, y, escala)`: cópia com alpha premultiplicado (`dst = src + dst × (255 − src.a) / 255`),
   com recorte (clipping) nas bordas, escala inteira por vizinho mais próximo.
3. `fill_rect`, `outline_rect`, `bar(valor, máximo)` (HP/fôlego).
4. Fonte bitmap: glifos de 5×7 em uma folha; `draw_text(str, x, y, cor, contorno)`; `measure(str)`.
5. Atlas: PNGs de `assets-src/` empacotados em um atlas (no `build.rs` ou previamente), embutidos com
   `include_bytes!`, decodificados uma única vez na inicialização e convertidos para premultiplicado.
   Após decodificar, o `Vec` do PNG comprimido não é mantido (`include_bytes!` fica na seção somente-leitura
   do executável, mapeada sob demanda e sem custo de Private Bytes).
6. Animação: sequência de quadros com duração em ms; interpolação linear da posição do lutador entre casas.
7. Cena (`scene.rs`): função `draw_fight(&FightView, &mut Surface)` que recebe uma visão somente-leitura
   (estrutura simples, sem depender do crate `game`; o `app` converte).

### Requisitos
- **Zero alocação por quadro.** Verificado por teste: alocador global contador em `cfg(test)`, renderizar
  1.000 quadros, contagem de alocações após o primeiro quadro = 0.
- **Golden images:** cenas fixas renderizadas e comparadas por hash com referências em
  `crates/render/tests/golden/`. Atualização de referência só com flag explícita (`UPDATE_GOLDEN=1`).
- **Acessibilidade visual:**
  - HP e fôlego distinguíveis por posição e forma (HP em cima e cheia; fôlego embaixo e segmentada), não
    só por cor;
  - paleta testada para deuteranopia/protanopia (sem depender de vermelho × verde);
  - opção de escala extra (+1 no fator inteiro quando houver altura).
- **Texto pt-BR é 20–30 % mais longo** que o inglês: layout com abreviações definidas na tabela de strings
  e números compactos (`1,2k`, `3,4M`) respeitando o separador decimal do idioma.

---

## 2.4 Interface e entrada

### Faixa (sempre visível na barra)
- Conteúdo: arena com os dois lutadores, HP e fôlego de cada um, número do turno, indicador de modo.
- Modo manual: botões de ação aparecem na faixa (ícones de 1 caractere de largura × altura da faixa).
  Ações inválidas ficam desabilitadas.
- Modo automático: ícone da estratégia atual; clique alterna pausa.
- Entrada **somente por mouse**. Clique esquerdo = ação; clique direito = abre o painel; arrastar = mover.

### Painel expandido
- Janela separada, comum (sem `WS_EX_NOACTIVATE`), acima da barra, alinhada à faixa; pode receber foco e
  teclado.
- Abas: Ficha (atributos, distribuir pontos), Loja, Liga, Configurações, Sobre.
- Fecha ao perder o foco (`WM_ACTIVATE` com `WA_INACTIVE`) ou com Esc.
- Tamanho em px lógicos (ex.: 360 × 280) multiplicado pela escala do DPI do monitor.

**Por que separar:** a faixa usa `WS_EX_NOACTIVATE` e nunca recebe teclado. Forçar foco nela tiraria o
foco do programa em que o usuário está trabalhando a cada clique.

### Outros
- Tooltips desenhados pelo próprio jogo (o controle padrão `TOOLTIPS_CLASS` tem problemas com janelas
  layered e não ativáveis).
- Atalho global opcional (`RegisterHotKey`), desativado por padrão. Se o registro falhar
  (`ERROR_HOTKEY_ALREADY_REGISTERED`), avisar no painel e manter desativado.
- Internacionalização: tabela de strings por chave (`i18n.rs`), idiomas `pt-BR` e `en`. Idioma inicial por
  `GetUserDefaultUILanguage`; troca manual nas configurações. Nenhum texto visível fora da tabela.

---

## 2.5 Crate `sim` — balanceamento

As metas, a forma de medir, o congelamento do balanceamento (baseline, replays, tag `balance-v1.0`) e o
processo para alterar regras estão em [`12-balanceamento-de-referencia.md`](12-balanceamento-de-referencia.md).
Esta seção trata só da implementação do crate.

### Tarefas
1. Binário headless que recebe uma matriz de cenários: diferença de nível (−5 a +5), build do jogador
   (equilibrada e as seis especializadas), arquétipo do oponente, tier de equipamento.
2. Para cada cenário, 10.000 lutas com sementes `1..=10000`.
3. Modo carreira: 1.000 carreiras completas do nível 1 ao campeão, com a política fixa do jogador simulado
   (documento 12, seção 4.2).
4. Saída CSV: cenário, taxa de vitória, turnos médios, desvio padrão de turnos, dano médio por turno,
   porcentagem de lutas na exaustão, distribuição de ações por arquétipo; para carreiras: nível ao vencer o
   campeão, total de lutas, vitórias por item, maior sequência de derrotas, ouro sem uso.
5. Subcomandos:
   - `check` — confere as metas de `balance/targets.json` (usado no CI);
   - `verify-baseline` — compara byte a byte com o baseline gravado (usado no CI);
   - `compare --baseline balance/` — gera `balance/compare.md` com a tabela antes × depois;
   - `write-baseline` — regenera `balance/baseline.csv` e `balance/baseline-careers.csv`.
6. Execução paralela por cenário com `std::thread::scope` (sem dependências): cada cenário tem suas
   próprias sementes, então o resultado independe da ordem de execução das threads.

---

## Preocupações da fase

| Preocupação | Medida |
|---|---|
| Escopo crescendo | MVP fechado: 1 liga, 3 arquétipos, 12 itens, 8 ações, 2 modos. Ideias novas vão para issues com rótulo `pos-1.0` |
| Acoplamento lógica ↔ Windows | Impossível por estrutura: `game` não depende de `windows-sys` |
| Balanceamento demorado | `sim` existe desde o início da fase; fórmulas com constantes nomeadas em um único módulo |
| Arte consumindo tempo | Placeholders geométricos até o fim da fase; arte final em paralelo |
| Save incompatível entre versões de desenvolvimento | `version` desde o primeiro save; fixtures de cada versão |

## Critérios de aceite

- [ ] Jogo jogável do nível 1 ao campeão da liga 1, em modo manual e automático.
- [ ] Testes unitários, de propriedade, golden images e 1 h de fuzzing sem falhas.
- [ ] `sim` dentro de todas as metas de balanceamento.
- [ ] Save sobrevive a: kill do processo em qualquer momento, arquivo corrompido, arquivo de 50 MB,
      usuário com acento no nome.
- [ ] Metas de memória (< 15 MB) e CPU (0 % parado, < 1 % em combate) atendidas com o jogo real.
