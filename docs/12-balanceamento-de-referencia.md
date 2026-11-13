# 12 — Balanceamento de referência (baseline congelado)

## 1. Objetivo

Ter um conjunto de regras **estável, medido e congelado** do início ao fim do desenvolvimento da versão 1.0,
para que:

1. nenhuma mudança de balanceamento aconteça por acidente durante o desenvolvimento;
2. ao terminar o jogo exista um **ponto de referência fixo** (tag `balance-v1.0`) a partir do qual o autor
   começa a alterar regras;
3. toda alteração posterior tenha o impacto numérico visível antes de ser aceita.

## 2. Por que não há engenharia reversa de outro jogo

- As regras deste projeto são **originais** e estão em [`11-design-do-jogo.md`](11-design-do-jogo.md).
- Não se descompila nem se extrai código, fórmulas, tabelas de itens, oponentes ou progressão de nenhum
  jogo existente (ver [`00-visao-geral.md`](00-visao-geral.md), seção 6). Copiar as regras exatas de um jogo
  específico produz uma recriação dele, e a descompilação de software de terceiros para reimplementação e
  distribuição pública tem risco jurídico.
- O objetivo real ("não desbalancear antes de começar a alterar") não depende de copiar regras. Depende de
  ter metas objetivas e um mecanismo que detecte qualquer desvio, descritos abaixo.

## 3. Metas de ritmo (propostas, sujeitas à revisão do autor)

Todas as metas são medidas pelo crate `sim`, com a política de jogo descrita na seção 4.2.
"Build equilibrada" = pontos distribuídos igualmente entre os seis atributos.

### 3.1 Combate (lutas isoladas)

| ID | Métrica | Meta | Justificativa |
|---|---|---|---|
| C1 | Turnos médios por luta, mesmo nível | 10 – 18 | No modo automático, a ~1,5 s por rodada, uma luta dura 15–27 s: acompanhável de relance sem exigir atenção contínua |
| C2 | Vitória contra oponente de mesmo nível, build equilibrada | 45 % – 60 % | Leve vantagem ao jogador; derrotas frequentes o suficiente para importar |
| C3 | Vitória contra oponente 1 nível abaixo | 60 % – 72 % | Treinar contra oponentes fracos compensa, mas não garante vitória |
| C4 | Vitória contra oponente 3 níveis abaixo | 80 % – 92 % | Nunca 100 %: o acaso continua relevante |
| C5 | Vitória contra oponente 1 nível acima | 35 % – 48 % | Desafio superável com bom equipamento |
| C6 | Vitória contra oponente 3 níveis acima | 15 % – 35 % | Possível, mas improvável |
| C7 | Vitória contra oponente 5 níveis acima | ≤ 10 % | Impede pular a progressão |
| C8 | Lutas que chegam à exaustão (turno ≥ 40) | < 5 % | A exaustão é uma garantia de término, não uma mecânica comum |
| C9 | Lutas decididas no turno 60 | < 0,5 % | Idem |
| C10 | Proporção de ataques pesados entre os ataques, arquétipo agressivo | 15 % – 40 % | O ataque pesado é uma escolha situacional, não a padrão |
| C11 | Ação mais usada por qualquer arquétipo | ≤ 60 % do total de ações | Nenhum arquétipo reduzido a uma única ação repetida |
| C12 | Desvio padrão de turnos por luta | ≥ 3 | Lutas variadas, não todas com a mesma duração |

### 3.2 Builds

| ID | Métrica | Meta | Justificativa |
|---|---|---|---|
| B1 | Taxa de vitória de cada build especializada contra todas as outras, mesmo nível | nenhuma > 80 % contra todas | Sem build dominante |
| B2 | Pior build especializada contra todas as outras | nenhuma < 25 % contra todas | Sem build inútil |
| B3 | Build só Defesa + Vitalidade | lutas na exaustão < 10 % | Impede estratégia de apenas sobreviver |
| B4 | Build Agilidade máxima: chance de ser atingido por ataque médio | ≥ 25 % | O limite inferior de acerto (100‰) e a fórmula precisam manter o jogador atingível |

Builds especializadas medidas: Força, Técnica, Agilidade, Defesa, Vitalidade, Fôlego (cada uma com 50 %
dos pontos no atributo principal e o restante distribuído igualmente), além da equilibrada.

### 3.3 Progressão (carreira completa)

| ID | Métrica | Meta | Justificativa |
|---|---|---|---|
| P0 | Carreiras simuladas que concluem a liga 1 | ≥ 99 % | Nenhuma carreira fica presa |
| P1 | Nível do jogador ao vencer o campeão da liga 1 | 15 – 18 | O campeão é nível 16; vencer bem abaixo indica progressão fácil demais, bem acima indica grind |
| P2 | Total de lutas até vencer o campeão (vitórias + derrotas + treinos) | 60 – 120 | Duração de campanha adequada para a versão 1.0 |
| P3 | Vitórias por item comprado (mediana da carreira) | 4 – 8 | Revisada: a 1.0 tem 12 itens e a campanha ~110 lutas, então 2–4 era inalcançável sem mais itens |
| P4 | Maior sequência de derrotas seguidas na carreira (mediana entre carreiras) | ≤ 6 | Revisada: com C2 em 45–60 % e ~110 lutas, sequências de 5 derrotas são estatisticamente esperadas; ≤ 4 contradizia C2 |
| P5 | Ouro sem uso ao vencer o campeão | ≤ 2 × preço do item mais caro | A loja precisa continuar relevante até o fim da liga |

**Informativo, sem meta:** `P6_horas_ate_campeao` (rodadas × 1,5 s). A primeira proposta (1–3 h)
contradizia C1 + P2: 10–18 turnos × 1,5 s × 60–120 lutas dá entre 15 e 54 minutos de luta. O tempo real
de uma campanha no modo automático depende do ritmo definido no aplicativo (pausa entre lutas), não das
regras; por isso é configuração do aplicativo, não meta de balanceamento.

### 3.4 Resultado do baseline atual

Medido com `cargo run --release -p sim -- check` (10.000 lutas por cenário, 1.000 carreiras). Valores
completos em `balance/baseline-metrics.json`.

| Meta | Valor | Meta | Meta | Valor | Meta |
|---|---|---|---|---|---|
| C1 | 14,7 turnos | 10–18 | B1 | 51,0 % | ≤ 80 |
| C2 | 54,7 % | 45–60 | B2 | 46,6 % | ≥ 25 |
| C3 | 71,1 % | 60–72 | B3 | 0,6 % | ≤ 10 |
| C4 | 87,0 % | 80–92 | B4 | 42,5 % | ≥ 25 |
| C5 | 45,0 % | 35–48 | P0 | 100 % | ≥ 99 |
| C6 | 24,9 % | 15–35 | P1 | nível 15 | 15–18 |
| C7 | 9,5 % | ≤ 10 | P2 | 114 lutas | 60–120 |
| C8 | 0,1 % | < 5 | P3 | 6,2 | 4–8 |
| C9 | 0,0 % | ≤ 0,5 | P4 | 5 | ≤ 6 |
| C10 | 34,3 % | 15–40 | P5 | 1,7 × | ≤ 2 |
| C11 | 38,0 % | ≤ 60 | P6 (info) | 0,75 h | — |
| C12 | 6,7 | ≥ 3 | | | |

Metas perto do limite (C3, C7, P1) devem ser observadas com atenção em qualquer ajuste.

## 4. Como as metas são medidas

### 4.1 Lutas isoladas
- Cenários (`crates/sim/src/fights.rs`):
  - **diferença de nível:** jogador com build equilibrada nos níveis 4, 9 e 15 contra oponentes de cada
    arquétipo (agressivo, defensivo, oportunista), com diferença de −5 a +5 níveis;
  - **builds:** cada par entre a build equilibrada e as seis especializadas (50 % dos pontos livres no
    atributo principal), nível 9, IA equilibrada nos dois lados;
  - **tank:** build Defesa + Vitalidade com IA defensiva contra equilibrada (meta B3).
- Equipamento por nível: o tier sobe a cada 4 níveis; a armadura acompanha com um tier de atraso e o
  escudo com dois.
- 10.000 lutas por cenário, com sementes `1..=10000`. Os dois lados são controlados pela IA.

### 4.2 Carreiras
- 1.000 carreiras completas, do nível 1 ao campeão, com sementes `1..=1000` (`crates/sim/src/career.rs`).
- Política fixa do jogador simulado:
  - distribuição de pontos: equilibrada (sempre no atributo de menor valor);
  - compra: para cada slot, na ordem arma → armadura → escudo, o item mais caro que puder pagar e que
    seja mais caro que o equipado;
  - escolha de luta: desafia o próximo oponente da liga; faz uma luta de treino contra o oponente
    anterior quando estiver mais de um nível abaixo do próximo ou depois de duas derrotas seguidas
    contra ele;
  - modo automático com a estratégia "equilibrado".
- Mudar essa política também muda os resultados; ela faz parte do baseline.

### 4.3 Determinismo
Como a lógica usa somente aritmética inteira e o gerador PCG32 próprio (verificado contra o vetor de
teste da implementação de referência), a mesma versão de regras com as mesmas sementes produz
**exatamente** os mesmos resultados em qualquer máquina, sistema operacional e número de threads. Isso
permite comparar o baseline por igualdade exata, sem margem de tolerância.

## 5. Mecanismo de congelamento

### 5.1 Arquivos

| Arquivo | Conteúdo |
|---|---|
| `crates/game/content/rules.json` | Todas as constantes de regras (custos, precisões, multiplicadores, curvas, economia, pesos da IA) |
| `crates/game/content/items.json` | Catálogo de itens |
| `crates/game/content/leagues.json` | Ligas e oponentes |
| `balance/targets.json` | As metas da seção 3 em formato de máquina (`id` → `min`/`max`) |
| `balance/baseline.csv` | Resultado completo da matriz de lutas; cabeçalho com a versão do balanceamento |
| `balance/baseline-careers.csv` | Resultado das 1.000 carreiras |
| `balance/baseline-metrics.json` | Métricas do baseline (usadas pelo `compare`) |
| `crates/game/tests/replays/replays.json` | 20 lutas gravadas: semente, estratégia, build, oponente, vencedor, turnos e hash do estado final |

**Versão do balanceamento:** é o hash FNV-1a de 64 bits dos três arquivos de conteúdo
(`Content::balance_version`). Ela muda sozinha quando qualquer valor muda; não existe número de versão
para esquecer de incrementar. O save grava essa versão.

Regra: nenhum número que afete o balanceamento pode existir fora dos arquivos de conteúdo. Revisão de
código rejeita números de regra espalhados no código.

### 5.2 Verificações automáticas (CI, job `balance`)

1. **Metas:** `sim check` roda a simulação completa e confere cada métrica contra `balance/targets.json`.
   Qualquer meta fora da faixa → CI falha.
2. **Baseline:** `sim verify-baseline` roda a simulação completa e compara byte a byte com
   `balance/baseline.csv` e `balance/baseline-careers.csv`. Diferença → CI falha com a mensagem
   "mudança de balanceamento não declarada" e mostra as linhas alteradas. Isso pega mudanças indiretas:
   alterar a IA, a ordem de resolução das ações ou a ordem das chamadas ao gerador aleatório também
   altera o resultado.
3. **Replays:** o teste `crates/game/tests/replays.rs` (job `test-portable`) executa os 20 replays e
   compara com o arquivo gravado.

Refatorações que não mudam regras (renomear funções, reorganizar módulos, otimizar desempenho) mantêm o
resultado idêntico e passam sem tocar no baseline. Se uma refatoração mudar o resultado, ela mudou as regras.

### 5.3 Marco de congelamento
- No fim da Fase 2, quando todas as metas estiverem atendidas e o autor tiver aprovado pelo checklist da
  seção 7, criar a tag **`balance-v1.0`** no commit correspondente.
- A Fase 3 não altera regras. Correção de bug que altere o resultado do `sim` segue o processo da seção 6
  e é registrada como exceção.

## 6. Processo para alterar regras (depois da `balance-v1.0`)

1. Criar branch e alterar os arquivos em `crates/game/content/` (ou a lógica, se a mudança for estrutural).
2. Medir o impacto:
   ```
   cargo run --release -p sim -- compare     # gera balance/compare.md (baseline × atual)
   cargo run --release -p sim -- check       # confere as metas
   ```
3. Se a mudança for aceita:
   ```
   cargo run --release -p sim -- write-baseline
   UPDATE_REPLAYS=1 cargo test -p game --test replays
   ```
   - se as metas mudaram de propósito, atualizar `balance/targets.json` e este documento;
   - registrar no `CHANGELOG.md`, seção "Balanceamento".
4. Abrir o PR com `balance/compare.md` colado na descrição.
5. Para voltar ao ponto de referência a qualquer momento:
   `git checkout balance-v1.0 -- crates/game/content/ balance/ crates/game/tests/replays/`.

Ferramentas de diagnóstico do simulador:
- `sim trace --seed N --level L` — imprime uma luta turno a turno;
- `sim career-trace --seed N` — imprime uma carreira luta a luta;
- `sim metrics` — imprime todas as métricas.

### 6.1 Efeitos da mudança de regras em saves existentes
- Luta em andamento salva com versão de balanceamento diferente da atual: a luta é reiniciada do turno 0
  com a mesma semente (continuar no meio com constantes novas produziria um estado que nenhuma das duas
  versões geraria).
- Itens removidos do catálogo são retirados do inventário e do equipamento na carga.
- Itens com preço alterado: o inventário não muda; só compras e vendas novas usam o preço novo.
- Oponentes já vencidos continuam vencidos (o progresso é limitado ao tamanho atual da liga).

## 7. Revisão das metas pelo autor (antes da Fase 2)

- [ ] Ler a seção 3 e ajustar qualquer meta com a qual não concorde (principalmente C1, P1 e P2).
- [ ] Confirmar a política do jogador simulado (seção 4.2).
- [ ] Registrar a aprovação em [`10-cronograma-e-decisoes.md`](10-cronograma-e-decisoes.md) (pendência P8).

Checklist de aprovação no fim da Fase 2 (antes da tag `balance-v1.0`):
- [ ] CI `balance-check` verde.
- [ ] O autor jogou ao menos 20 lutas no modo manual e 1 h no modo automático.
- [ ] Nenhuma luta pareceu impossível ou trivial demais para o nível; anotar exceções e, se houver,
      ajustar e repetir.
- [ ] O autor venceu o campeão da liga 1 em uma carreira real.
