# 14 — Formato dos arquivos de conteúdo

Todo o conteúdo e todas as regras numéricas do jogo ficam em três arquivos JSON em
`crates/game/content/`. Eles são embutidos no executável na compilação (`include_str!`) e validados na
carga (`Content::from_json`). O motor (combate, IA, progressão) não contém números de balanceamento.

| Arquivo | Conteúdo |
|---|---|
| `rules.json` | Fórmulas, custos, precisões, multiplicadores, término da luta, progressão, economia, pesos da IA |
| `items.json` | Catálogo de itens |
| `leagues.json` | Ligas e oponentes |

Convenções numéricas:
- Sufixo `_pm`: **por mil** (‰). `850` = 85 %; como multiplicador, `1500` = ×1,5.
- Sufixo `_tenth`: décimos. `damage_per_strength_tenth: 7` = +0,7 de dano por ponto de Força.
- Todos os valores são inteiros.

## 1. Como alterar o conteúdo com segurança

1. Edite os arquivos JSON.
2. Rode os testes de carga: `cargo test -p game content` (o conteúdo inválido é recusado com uma mensagem
   que indica o campo).
3. Meça o impacto: `cargo run --release -p sim -- compare` (tabela antes × depois) e
   `cargo run --release -p sim -- check` (metas).
4. Se aceitar o resultado, regenere o baseline e os replays (processo completo em
   [`12-balanceamento-de-referencia.md`](12-balanceamento-de-referencia.md), seção 6).

A versão do balanceamento é o hash dos três arquivos, então qualquer mudança é detectada pelo CI até que
o baseline seja regenerado.

## 2. `rules.json`

Campos desconhecidos são **recusados** (protege contra erro de digitação).

### 2.1 `arena`
| Campo | Tipo | Significado | Validação |
|---|---|---|---|
| `cells` | inteiro | número de casas | 4 a 32 |
| `player_start` | inteiro | casa inicial do jogador (lado esquerdo) | menor que `opponent_start` |
| `opponent_start` | inteiro | casa inicial do oponente (lado direito) | menor que `cells` |

### 2.2 `attributes`
| Campo | Significado |
|---|---|
| `min`, `max` | limites de cada atributo (`min` ≥ 1) |
| `start_value` | valor inicial de cada um dos 6 atributos |
| `start_free_points` | pontos livres na criação do personagem |
| `points_per_level` | pontos ganhos a cada nível |

Ordem dos atributos em todos os vetores: Força, Técnica, Agilidade, Defesa, Vitalidade, Fôlego.

### 2.3 `derived`
| Campo | Fórmula em que aparece |
|---|---|
| `hp_base`, `hp_per_vitality`, `hp_per_level` | HP = base + por_vitalidade × Vitalidade + por_nível × Nível |
| `stamina_base`, `stamina_per_point`, `stamina_min` | Fôlego máx. = base + por_ponto × Fôlego − penalidade do escudo (mínimo `stamina_min`) |
| `regen_base`, `regen_divisor` | Recuperação por turno = base + Fôlego / divisor |
| `damage_base`, `damage_per_strength_tenth` | Dano = base + Força × décimos / 10 + dano da arma |
| `crit_base_pm`, `crit_per_technique_tenth_pm`, `crit_max_pm` | Crítico = base + Técnica × décimos / 10, limitado ao máximo |
| `crit_multiplier_pm` | multiplicador do dano crítico |
| `absorb_per_defense` | Absorção = por_defesa × Defesa + armadura + escudo |
| `initiative_die` | lados do dado de iniciativa |

### 2.4 `hit`
`min_pm`, `max_pm` (limites da chance de acerto) e `per_point_pm` (‰ por ponto de diferença entre a
Técnica do atacante e a Agilidade efetiva do defensor).

### 2.5 `damage`
`variance_min_pm`/`variance_max_pm` (variação do dano), `min_damage` (dano mínimo de um acerto, ≥ 1) e
`absorb_scale` (dano final = dano × escala / (escala + absorção)).

### 2.6 `actions`
| Ação | Campos |
|---|---|
| `advance`, `retreat` | `stamina` |
| `light`, `medium`, `heavy` | `stamina`, `range_min`, `range_max`, `accuracy_pm`, `far_penalty_pm` (por casa além da distância 1), `damage_pm`, `crit_bonus_pm` |
| `defend` | `damage_taken_pm` |
| `recover` | `regen_multiplier`, `damage_taken_pm` |
| `taunt` | `stamina`, `next_crit_bonus_pm`, `next_damage_pm`, `self_absorb_pm` |

As 8 ações existem no código; os arquivos definem só os números. Adicionar uma ação nova exige código.

### 2.7 `fight_end`
`exhaustion_turn` (turno em que a exaustão começa), `exhaustion_pm_per_turn` (perda de fôlego máximo por
turno, em ‰ do original) e `max_turns` (último turno; obrigatório, garante que toda luta termina).

### 2.8 `progression`
| Campo | Significado |
|---|---|
| `max_level` | nível máximo |
| `xp_curve.a`, `.b`, `.c` | XP do nível n para n+1 = a + b·n + c·n² |
| `xp_win_base`, `xp_win_per_level` | XP por vitória = base + por_nível × nível do oponente |
| `xp_higher_level_pm` | multiplicador quando o oponente tem nível maior |
| `xp_loss_pm`, `gold_loss_pm` | fração da recompensa em caso de derrota |
| `gold_win_base`, `gold_win_per_level` | ouro por vitória |
| `rematch_reward_pm` | fração da recompensa contra oponente já vencido |
| `sell_pm` | preço de venda em relação ao de compra |
| `start_gold` | ouro inicial |
| `inventory_max` | itens máximos no inventário |

### 2.9 `archetypes` e `ai`
Cada arquétipo (`aggressive`, `defensive`, `opportunist`, `balanced`) tem: `preferred_distance`,
`w_damage`, `w_risk`, `w_stamina`, `w_distance`, `w_opportunity`, `heavy_min_stamina_pm`. O bloco `ai`
tem `noise_max`, `taunt_value`, `recover_value_per_missing_pm` e `potential_pm`. O modelo de pontuação
está em [`11-design-do-jogo.md`](11-design-do-jogo.md), seção 8.

## 3. `items.json`

```json
{ "items": [
  { "id": 2, "kind": "weapon", "name": { "pt": "Espada curta de bronze", "en": "Bronze shortsword" },
    "price": 60, "damage": 4, "accuracy_pm": 20, "weight": 1 }
] }
```

| Campo | Tipos de item | Significado |
|---|---|---|
| `id` | todos | identificador único e **estável** (aparece no save; nunca reutilize) |
| `kind` | todos | `weapon`, `armor` ou `shield` |
| `name.pt`, `name.en` | todos | nome exibido |
| `price` | todos | preço de compra |
| `damage`, `accuracy_pm`, `weight` | armas | dano adicional, modificador de precisão, custo extra de fôlego por ataque |
| `armor`, `agility_penalty` | armaduras | absorção, redução de Agilidade |
| `shield`, `stamina_penalty` | escudos | absorção, redução do fôlego máximo |

Campos de outro tipo de item são ignorados (valor 0). Validação: IDs únicos, valores de peso, armadura e
escudo não negativos, e ao menos uma arma com preço 0 (a arma inicial; se houver várias, vale a de menor
ID).

Itens removidos do arquivo são retirados do inventário dos saves existentes na carga (com registro no
log).

## 4. `leagues.json`

```json
{ "leagues": [
  { "id": 1, "name": { "pt": "Liga do Porto", "en": "Harbor League" },
    "opponents": [
      { "id": 1, "name": "Taro, o Pescador", "level": 1, "archetype": "balanced",
        "build": [1, 1, 1, 1, 1, 1], "weapon": 1, "armor": null, "shield": null }
    ] }
] }
```

| Campo | Significado |
|---|---|
| `opponents` | em ordem; o último é o campeão da liga |
| `level` | nível do oponente (1 a `max_level`) |
| `archetype` | `aggressive`, `defensive`, `opportunist` ou `balanced` |
| `build` | pesos de distribuição dos pontos, na ordem dos atributos; os pontos do nível são distribuídos proporcionalmente (função `Fighter::distribute`) |
| `weapon`, `armor`, `shield` | ID de item do tipo correspondente, ou `null` |

Regras: a primeira liga está sempre liberada; cada liga seguinte é liberada ao vencer o campeão da
anterior. Para desafiar o oponente k é preciso ter vencido o k−1. Validação: ao menos uma liga, nenhuma
liga vazia, níveis válidos, build com algum peso positivo, itens existentes e do tipo certo.

## 5. O que exige código (não só dados)

- Novos tipos de ação, novos tipos de item ou novos slots de equipamento.
- Novos atributos primários.
- Mudanças na forma das fórmulas (por exemplo, trocar a redução proporcional por subtração fixa).
- Mudanças na política do jogador simulado (`crates/sim/src/career.rs`).
