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
| P1 | Nível do jogador ao vencer o campeão da liga 1 | 15 – 18 | O campeão é nível 16; vencer bem abaixo indica progressão fácil demais, bem acima indica grind |
| P2 | Total de lutas até vencer o campeão (vitórias + derrotas + treinos) | 60 – 120 | Duração de campanha adequada para a versão 1.0 |
| P3 | Vitórias por item comprado (média da carreira) | 2 – 4 | Recompensa material frequente |
| P4 | Maior sequência de derrotas seguidas na carreira (mediana entre carreiras) | ≤ 4 | Evita frustração em uso passivo |
| P5 | Ouro sem uso ao vencer o campeão | ≤ 2 × preço do item mais caro | A loja precisa continuar relevante até o fim da liga |
| P6 | Tempo de jogo até o campeão no modo automático contínuo | 1 h – 3 h | **Revisar:** depende de quanto tempo o autor quer que a liga 1 dure |

## 4. Como as metas são medidas

### 4.1 Lutas isoladas
- Para cada combinação (diferença de nível −5…+5, build, arquétipo do oponente, tier de equipamento):
  10.000 lutas com sementes `1..=10000`.
- Os dois lados são controlados pela IA (o jogador com a estratégia "equilibrado").

### 4.2 Carreiras
- Simulação de 1.000 carreiras completas, do nível 1 ao campeão, com sementes `1..=1000`.
- Política fixa e documentada do jogador simulado:
  - distribuição de pontos: build equilibrada (variante: cada build especializada);
  - compra: o item mais caro que puder pagar, na ordem arma → armadura → escudo;
  - escolha de luta: desafia o próximo oponente da liga; após 2 derrotas seguidas contra ele, faz 1 luta
    de treino contra o oponente anterior;
  - modo automático com a estratégia "equilibrado".
- Mudar essa política também muda os resultados; ela faz parte do baseline.

### 4.3 Determinismo
Como a lógica usa somente aritmética inteira e o gerador aleatório próprio
([`04-fase-2-jogo.md`](04-fase-2-jogo.md), seção 2.1), a mesma versão de regras com as mesmas sementes
produz **exatamente** os mesmos resultados em qualquer máquina e sistema operacional. Isso permite comparar
o baseline por igualdade exata, sem margem de tolerância.

## 5. Mecanismo de congelamento

### 5.1 Arquivos

| Arquivo | Conteúdo |
|---|---|
| `crates/game/src/balance.rs` | **Todas** as constantes de regras (custos, precisões, multiplicadores, curvas de XP, preços, atributos dos oponentes) e `pub const BALANCE_VERSION: u32` |
| `balance/targets.toml` | As metas da seção 3 em formato de máquina (`id`, `min`, `max`) |
| `balance/baseline.csv` | Resultado completo do `sim` para a versão de regras atual; cabeçalho com `BALANCE_VERSION` |
| `balance/baseline-careers.csv` | Resultado das 1.000 carreiras |
| `crates/game/tests/replays/*.json` | ~20 lutas gravadas: estado inicial, semente, sequência de entradas e estado final esperado |

Regra: nenhum número que afete o balanceamento pode existir fora de `balance.rs`. Revisão de código
rejeita números de regra espalhados.

### 5.2 Verificações automáticas (CI, job `balance-check`)

1. **Metas:** roda o `sim` em modo reduzido (1.000 lutas por cenário, 100 carreiras) e confere cada métrica
   contra `balance/targets.toml`. Qualquer meta fora da faixa → CI falha.
2. **Baseline:** roda o `sim` completo com as sementes fixas e compara byte a byte com
   `balance/baseline.csv` e `balance/baseline-careers.csv`.
   - Se `BALANCE_VERSION` não mudou e o resultado for diferente → CI falha com a mensagem
     "mudança de balanceamento não declarada" e mostra as linhas alteradas.
   - Isso pega mudanças indiretas: alterar a IA, a ordem de resolução de ações ou a ordem das chamadas ao
     gerador aleatório também altera o resultado, e essas mudanças afetam o balanceamento.
3. **Replays:** cada replay é executado e o estado final comparado com o esperado. Diferença → falha.
4. **Consistência:** se `BALANCE_VERSION` mudou, o PR precisa conter o baseline e os replays regenerados e
   uma entrada no `CHANGELOG.md` na seção "Balanceamento".

Refatorações que não mudam regras (renomear funções, reorganizar módulos, otimizar desempenho) mantêm o
resultado idêntico e passam sem tocar no baseline. Se uma refatoração mudar o resultado, ela mudou as regras.

### 5.3 Marco de congelamento
- No fim da Fase 2, quando todas as metas estiverem atendidas e o autor tiver aprovado pelo checklist da
  seção 7, criar a tag **`balance-v1.0`** no commit correspondente.
- A Fase 3 não altera regras. Correção de bug que altere o resultado do `sim` segue o processo da seção 6
  e é registrada como exceção.

## 6. Processo para alterar regras (depois da `balance-v1.0`)

1. Criar branch e alterar somente `balance.rs` (ou a lógica, se a mudança for estrutural).
2. Rodar `cargo run -p sim --release -- compare --baseline balance/` → gera `balance/compare.md` com uma
   tabela por métrica: valor no baseline, valor novo, diferença absoluta, se continua dentro da meta.
3. Avaliar a tabela. Se a mudança for aceita:
   - incrementar `BALANCE_VERSION`;
   - `cargo run -p sim --release -- write-baseline` (regenera os CSVs);
   - `UPDATE_REPLAYS=1 cargo test -p game replays` (regenera os replays);
   - se as metas mudaram de propósito, atualizar `balance/targets.toml` e este documento;
   - registrar no `CHANGELOG.md`.
4. Abrir o PR com `compare.md` colado na descrição.
5. Para voltar ao ponto de referência a qualquer momento: `git checkout balance-v1.0 -- crates/game/src/balance.rs balance/`.

### 6.1 Efeitos da mudança de regras em saves existentes
- O save grava `BALANCE_VERSION`.
- Luta em andamento salva com versão de regras diferente da atual: a luta é reiniciada do turno 0 com a
  mesma semente (continuar no meio com constantes novas produziria um estado que nenhuma das duas versões
  geraria).
- Itens com preço alterado: o inventário não muda; só compras e vendas novas usam o preço novo.
- Oponentes já vencidos continuam vencidos.

## 7. Revisão das metas pelo autor (antes da Fase 2)

- [ ] Ler a seção 3 e ajustar qualquer meta com a qual não concorde (principalmente C1, P1, P2 e P6).
- [ ] Confirmar a política do jogador simulado (seção 4.2).
- [ ] Registrar a aprovação em [`10-cronograma-e-decisoes.md`](10-cronograma-e-decisoes.md) (pendência P8).

Checklist de aprovação no fim da Fase 2 (antes da tag `balance-v1.0`):
- [ ] CI `balance-check` verde.
- [ ] O autor jogou ao menos 20 lutas no modo manual e 1 h no modo automático.
- [ ] Nenhuma luta pareceu impossível ou trivial demais para o nível; anotar exceções e, se houver,
      ajustar e repetir.
- [ ] O autor venceu o campeão da liga 1 em uma carreira real.
