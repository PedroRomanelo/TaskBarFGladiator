# 11 — Design do jogo (GDD inicial)

Este documento é a especificação do crate `game`. Todas as regras são originais. Todos os números são
**valores iniciais** para serem ajustados com o crate `sim` até atingir as metas de
[`12-balanceamento-de-referencia.md`](12-balanceamento-de-referencia.md); depois disso ficam congelados na
tag `balance-v1.0`. Toda constante deste
documento vira uma constante nomeada em um único módulo (`balance.rs`), para que o balanceamento não exija
procurar números espalhados pelo código.

Convenções:
- Percentuais são implementados em **por mil** (‰) com inteiros: 85 % = 850‰.
- Multiplicadores são implementados em ponto fixo com 1/1000: ×1,6 = 1600.
- Divisões inteiras arredondam para baixo, salvo indicação.

---

## 1. Lutador

### 1.1 Atributos primários (1 a 99)

| Atributo | Efeito principal |
|---|---|
| **Força** | dano base |
| **Técnica** | precisão e chance de crítico |
| **Agilidade** | esquiva e iniciativa |
| **Defesa** | redução de dano |
| **Vitalidade** | HP máximo |
| **Fôlego** | fôlego máximo e recuperação por turno |

Personagem novo: todos os atributos em 5 e **10 pontos** livres para distribuir.

### 1.2 Atributos derivados

| Derivado | Fórmula inicial |
|---|---|
| HP máximo | `50 + 8 × Vitalidade + 2 × Nível` |
| Fôlego máximo | `30 + 4 × Fôlego` |
| Recuperação de fôlego por turno | `2 + Fôlego / 5` |
| Dano base | `4 + Força + dano_da_arma` |
| Chance de crítico | `50‰ + Técnica × 2,5‰`, máximo 300‰ |
| Multiplicador de crítico | ×1,5 |
| Iniciativa | `Agilidade + d10` (sorteado a cada rodada) |
| Absorção | `4 × Defesa + armadura + escudo` |

---

## 2. Arena e turnos

- Arena de **10 casas** (0 a 9). O jogador começa na casa 2 e o oponente na casa 7 (distância 5).
- Dois lutadores nunca ocupam a mesma casa.
- **Rodada:** os dois lutadores escolhem uma ação; age primeiro quem tiver maior iniciativa na rodada
  (empate: maior Agilidade; persistindo, sorteio). Cada ação é resolvida por completo antes da próxima.
- No fim de cada rodada, cada lutador recupera fôlego (`recuperação por turno`), limitado ao máximo.
- Se o primeiro a agir derrubar o oponente (HP 0), a ação do oponente naquela rodada não acontece.

---

## 3. Ações

| Ação | Custo de fôlego | Alcance (distância) | Precisão base | Multiplicador de dano | Efeito adicional |
|---|---|---|---|---|---|
| Avançar | 3 | — | — | — | move 1 casa em direção ao oponente |
| Recuar | 3 | — | — | — | move 1 casa para longe do oponente |
| Ataque leve | 6 | 1–2 | 850‰ (−150‰ a distância 2) | ×0,7 | — |
| Ataque médio | 12 | 1 | 700‰ | ×1,0 | — |
| Ataque pesado | 22 | 1 | 500‰ | ×1,6 | +50‰ de chance de crítico |
| Defender | 0 | — | — | — | dano recebido ×0,5 até o fim da próxima ação do oponente |
| Recuperar | 0 | — | — | — | recupera 3 × recuperação por turno; dano recebido ×1,25 até a próxima ação própria |
| Provocar | 4 | — | — | — | próximo ataque: +150‰ crítico e dano ×1,1; absorção própria −20 % até a próxima ação própria |

Regras de validade:
- Avançar é inválido com o oponente na casa adjacente.
- Recuar é inválido encostado na parede (casa 0 ou 9).
- Ataques são inválidos fora do alcance ou sem fôlego suficiente.
- Defender e Recuperar são sempre válidas (garantem que sempre existe uma ação possível).
- O custo de fôlego das ações de ataque aumenta com o peso da arma (seção 5).

---

## 4. Resolução de um ataque

1. **Chance de acerto:**
   `acerto = precisão_base + mod_precisão_arma + (Técnica_atacante − Agilidade_defensor) × 10‰`,
   limitada a [100‰, 950‰]. Sorteio `rng.chance(acerto)`.
2. **Se errou:** fim da ação (evento "errou").
3. **Dano bruto:** `dano_base × multiplicador_da_ação`, com variação aleatória de ±10 %
   (`rng.range(900, 1100)` em ‰).
4. **Crítico:** sorteio com a chance de crítico (+ bônus da ação e da provocação). Se crítico, ×1,5.
5. **Redução:** `dano = dano_bruto × 100 / (100 + absorção_defensor)`.
6. **Modificadores de postura do defensor:** Defender ×0,5; Recuperar ×1,25.
7. **Mínimo:** um acerto causa pelo menos 1 de dano.
8. HP do defensor `saturating_sub(dano)`.

Exemplo numérico (nível 1, atacante Força 10 / Técnica 8, arma +3 de dano; defensor Agilidade 6 /
Defesa 5, sem armadura), ataque médio:
- acerto = 700 + (8 − 6) × 10 = 720‰
- dano base = 4 + 10 + 3 = 17; bruto ×1,0 = 17 (sem variação para o exemplo)
- absorção = 4 × 5 = 20 → dano = 17 × 100 / 120 = 14

---

## 5. Equipamento (12 itens na 1.0)

Nomes genéricos e descritivos; todos originais.

### 5.1 Armas
| ID | Nome | Dano | Mod. precisão | Peso (custo extra de fôlego por ataque) | Preço |
|---|---|---|---|---|---|
| 1 | Faca de treino | +1 | +50‰ | 0 | 0 (inicial) |
| 2 | Espada curta de bronze | +4 | +20‰ | 1 | 60 |
| 3 | Lança de freixo | +6 | 0 | 2 | 140 |
| 4 | Machado de ferro | +10 | −50‰ | 4 | 300 |

### 5.2 Armaduras
| ID | Nome | Armadura | Penalidade de Agilidade | Preço |
|---|---|---|---|---|
| 101 | Túnica acolchoada | +4 | 0 | 40 |
| 102 | Couro fervido | +10 | −1 | 110 |
| 103 | Malha de anéis | +18 | −3 | 260 |
| 104 | Placas de bronze | +28 | −6 | 520 |

### 5.3 Escudos
| ID | Nome | Escudo | Penalidade de fôlego máximo | Preço |
|---|---|---|---|---|
| 201 | Broquel de madeira | +3 | 0 | 30 |
| 202 | Escudo redondo | +8 | −4 | 120 |
| 203 | Escudo oval reforçado | +14 | −8 | 280 |
| 204 | Escudo de torre | +22 | −15 | 480 |

- Venda: 50 % do preço.
- IDs nunca são reutilizados (aparecem no save). Faixas: armas 1–99, armaduras 101–199, escudos 201–299.

---

## 6. Progressão

| Item | Fórmula inicial |
|---|---|
| XP para passar do nível n para n+1 | `40 + 30 × n + 5 × n²` |
| Pontos de atributo por nível | 3 |
| Nível máximo na 1.0 | 30 |
| XP por vitória | `20 + 8 × nível_oponente` (+50 % se o oponente tiver nível maior) |
| XP por derrota | 50 % da vitória |
| Ouro por vitória | `20 + 10 × nível_oponente` |
| Ouro por derrota | 25 % da vitória |

- **Derrota não é permanente:** o lutador perde apenas a recompensa integral. HP e fôlego são restaurados
  entre lutas.
- XP satura no nível máximo; a interface mostra "nível máximo".

---

## 7. Liga 1

- 8 oponentes em ordem crescente de nível (1, 2, 4, 6, 8, 10, 12, 14) + campeão (nível 16).
- Para desafiar o oponente k, é preciso ter vencido o k−1.
- Oponentes já vencidos podem ser enfrentados de novo para treino (recompensa de 50 %).
- Atributos dos oponentes gerados a partir do nível e do arquétipo (distribuição de pontos definida por
  arquétipo), com equipamento fixo por posição na liga. Nomes de oponentes originais, definidos na Fase 2.
- Vencer o campeão conclui a liga 1 (marco da versão 1.0); ligas seguintes são pós-1.0.

---

## 8. Inteligência artificial

### 8.1 Modelo
A IA avalia todas as ações válidas e escolhe pela pontuação de utilidade:
`pontuação(ação) = Σ (peso_do_arquétipo[fator] × valor_do_fator(ação, estado))` + ruído pequeno
(`rng.range(0, 50)`) para evitar previsibilidade total. Escolhe a maior pontuação. Função pura e determinística
dado o estado do RNG.

### 8.2 Fatores
| Fator | Valor |
|---|---|
| Dano esperado | `chance_de_acerto × dano_médio` da ação |
| Risco recebido | dano esperado do melhor ataque do oponente na próxima rodada, considerando a postura resultante |
| Fôlego após a ação | quanto fôlego resta em relação ao máximo |
| Distância ideal | proximidade da distância preferida do arquétipo |
| Oportunidade | oponente com fôlego baixo, em postura Recuperar, ou com HP baixo |

### 8.3 Arquétipos
| Arquétipo | Distância preferida | Comportamento resultante |
|---|---|---|
| **Agressivo** | 1 | prioriza dano esperado; usa ataque pesado com fôlego > 50 %; raramente defende |
| **Defensivo** | 2 | prioriza reduzir risco; usa ataque leve a distância 2; defende quando o oponente tem fôlego para ataque pesado |
| **Oportunista** | 1–2 | provoca quando o oponente está longe ou sem fôlego; ataque pesado quando o oponente usou Recuperar ou está com fôlego baixo |
| **Equilibrado** (só jogador, modo automático) | 1 | pesos médios |

No modo automático, o jogador escolhe a estratégia (um dos quatro) no painel.

---

## 9. Regras de término da luta

| Condição | Resultado |
|---|---|
| HP de um lutador chega a 0 | o outro vence |
| Turno 40 em diante | **exaustão**: fôlego máximo dos dois cai 5 % do valor original por turno |
| Turno 60 | vence o maior percentual de HP restante |
| Empate de percentual no turno 60 | vence quem causou mais dano total |
| Empate de dano total | vence o jogador (desafiante) |

Garantia (testada): toda luta termina em no máximo 60 turnos.

---

## 10. Pontos a validar com o `sim`

As metas numéricas oficiais, a forma de medir e o congelamento do balanceamento estão em
[`12-balanceamento-de-referencia.md`](12-balanceamento-de-referencia.md). Os pontos abaixo são as
preocupações de design que motivaram aquelas metas.

1. Ataque pesado não pode ser a melhor escolha em todas as situações (verificar a taxa de uso por arquétipo).
2. Build só de Defesa + Vitalidade não pode levar a maioria das lutas à exaustão.
3. Agilidade alta não pode tornar o lutador praticamente impossível de acertar (limite inferior de 100‰
   existe por isso; verificar se é suficiente).
4. Progressão: jogador com build média deve conseguir vencer o campeão da liga 1 por volta do nível 15–18.
5. Preços: o jogador deve conseguir comprar um item novo a cada 2–4 vitórias.
