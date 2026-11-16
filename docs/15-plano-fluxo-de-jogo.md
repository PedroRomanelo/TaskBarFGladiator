# 15 — Plano: fluxo de jogo (criação, atributos, intervalo entre lutas e economia)

**Status:** proposto. A execução começa quando o autor responder às decisões da seção 8.
**Depende de:** palco acima da barra e lutador em camadas (D26–D28, já implementados).

## 1. Objetivo

Hoje o jogo abre direto num gladiador pronto chamado "Gladiador", com seis atributos sem explicação,
e depois de cada luta o jogador precisa descobrir sozinho que existe uma loja e que ganhou pontos. Este
plano define o caminho completo de uma partida, sem tela morta:

```
            sem save                                    com save
               │                                           │
        ┌──────▼──────┐                             ┌──────▼──────┐
        │   TÍTULO    │── NOVO JOGO ──┐             │   TÍTULO    │── CONTINUAR ──┐
        └─────────────┘               │             └─────────────┘               │
                          ┌───────────▼────────────┐                              │
                          │ CRIAÇÃO (painel)       │                              │
                          │ 1/3 nome               │                              │
                          │ 2/3 aparência          │                              │
                          │ 3/3 atributos          │                              │
                          └───────────┬────────────┘                              │
                                      ▼                                           ▼
   ┌────────────────────────────► PREPARAR ◄──────────────────────────────────────┘
   │                                  │ LUTAR
   │                                  ▼
   │                                LUTA  (sua vez → rodada narrada → …)
   │                                  │ fim
   │                                  ▼
   │                              RESULTADO  (XP, ouro, "NÍVEL 5!")
   │                                  │ CONTINUAR
   │                                  ▼
   │            ┌──────────────── INTERVALO ────────────────┐
   │            │ subiu de nível?  → FICHA: distribuir pontos │   o painel só abre se houver
   │            │ item ao alcance? → LOJA: comprar            │   algo a fazer
   └────────────┴─────────────────────────────────────────────┘
```

Princípios:

1. **Cada etapa tem um único botão principal**, sempre no mesmo lugar (D27), e a legenda diz o que fazer.
2. **O jogo só interrompe quando há algo a decidir.** Sem pontos e sem item ao alcance, CONTINUAR volta
   direto para PREPARAR.
3. **Todo atributo tem efeito escrito e visível**: a descrição e os números derivados aparecem na hora
   de distribuir, e mudam ao vivo.
4. **O dinheiro sempre tem destino próximo**: a loja mostra o que cada item muda e quantas vitórias
   faltam para o próximo.
5. **Regras e conteúdo próprios** (D2, D17). Os papéis dos atributos seguem a convenção do gênero
   (força → dano, ataque → acerto, carisma → economia e provocação), mas fórmulas, números, nomes, itens,
   oponentes e arte são deste projeto. As notas de referência do autor sobre jogos do gênero servem de
   inspiração de estrutura, não de tabela a copiar.

## 2. Tela de título e criação do personagem

### 2.1 Título

- **Quando aparece:** ao abrir o jogo sem `save.json` válido, e ao escolher NOVO JOGO em CONFIG.
- **No palco:** um gladiador de aparência padrão parado no centro; legenda com o nome do jogo; botão
  principal **NOVO JOGO** (sem save) ou **CONTINUAR** (com save). Clique direito abre o painel, como hoje.
- **Novo jogo com save existente:** pede confirmação no painel ("APAGAR O GLADIADOR ATUAL?"). O save
  antigo é copiado para `save.json.bak` antes (mecanismo que já existe em `app/src/storage.rs`).
- **Um gladiador por vez** na 1.0 (ver decisão PD5).

### 2.2 Assistente de criação (no painel, 3 passos)

A criação acontece no painel porque ele é a única janela que recebe teclado (D13). Enquanto o painel
está aberto, o palco mostra o gladiador sendo montado, ao vivo.

```
┌ CRIAR GLADIADOR ─────────────── 1/3 ┐   ┌ CRIAR GLADIADOR ─────────────── 3/3 ┐
│ NOME                                │   │ PONTOS LIVRES: 4                    │
│  ┌──────────────┐                   │   │ FORÇA       7  [-][+]   VIDA    60  │
│  │ ARIA_        │  [SORTEAR]        │   │ ATAQUE      6  [-][+]   FÔLEGO  55  │
│  └──────────────┘                   │   │ AGILIDADE   5  [-][+]   DANO    14  │
│ 2 A 12 LETRAS                       │   │ DEFESA      5  [-][+]   ACERTO  71% │
│                                     │   │ VITALIDADE  7  [-][+]   CRÍTICO  7% │
│                                     │   │ FÔLEGO      5  [-][+]   DESCONTO 0% │
│                                     │   │ CARISMA     6  [-][+]               │
│                                     │   │ FORÇA: AUMENTA O DANO DOS GOLPES    │
│                    [AVANÇAR]        │   │ [VOLTAR]  [EQUILIBRAR]   [COMEÇAR]  │
└─────────────────────────────────────┘   └─────────────────────────────────────┘
```

| Passo | Conteúdo | Regras |
|---|---|---|
| 1/3 Nome | campo de texto com cursor, botão SORTEAR | 2 a 12 caracteres; letras (com os acentos da fonte), números, espaço e hífen; espaços nas pontas são removidos; SORTEAR escolhe um nome de uma lista própria, então dá para criar só com o mouse |
| 2/3 Aparência | as cinco linhas da aba VISUAL (pele, cabelo, cor, olhos, barba) + retrato | mesmo código da aba VISUAL; pode ser mudado depois, de graça |
| 3/3 Atributos | sete linhas com `[-]` `[+]`, pontos livres, coluna de números derivados, linha de descrição do atributo sob o mouse | parte de 5 em cada atributo, com 10 pontos livres; `[-]` devolve o ponto; COMEÇAR só habilita com 0 pontos livres; EQUILIBRAR distribui o que falta no menor atributo |

- VOLTAR preserva o que foi preenchido. Esc ou clicar fora fecha o painel e **mantém o rascunho**; o
  palco continua no título com o botão **CRIAR** para reabrir.
- COMEÇAR cria o `GameState`, grava o save e vai para PREPARAR com a legenda da primeira luta.

## 3. Atributos

### 3.1 Lista e efeitos

Sete atributos, de 1 a 99. Seis já existem; **Técnica passa a se chamar Ataque** (mesmo efeito, nome
mais claro) e entra **Carisma**. Os números das seis primeiras linhas são os do baseline atual
(`rules.json`); os de Carisma são valores iniciais a ajustar com o `sim` (seção 6).

Convenção: `c = Carisma − 5` (o quanto está acima do valor inicial). Com Carisma 5, tudo abaixo vale
exatamente o que vale hoje.

| Atributo | O que faz (texto mostrado ao jogador) | Fórmula |
|---|---|---|
| **Força** | AUMENTA O DANO DOS GOLPES | dano base = `9 + 0,7 × Força + dano da arma` |
| **Ataque** | AUMENTA A CHANCE DE ACERTAR E DE CRÍTICO | acerto `+15‰` por ponto acima da Agilidade do oponente; crítico = `50‰ + 4‰ × Ataque` (máx. 300‰) |
| **Agilidade** | DIFICULTA SER ACERTADO; AGE PRIMEIRO | acerto do oponente `−15‰` por ponto; iniciativa = `Agilidade + d10` |
| **Defesa** | REDUZ O DANO RECEBIDO | absorção = `3 × Defesa + armadura + escudo`; dano × `100 / (100 + absorção)` |
| **Vitalidade** | AUMENTA A VIDA | HP = `30 + 4 × Vitalidade + 2 × nível` |
| **Fôlego** | MAIS FÔLEGO PARA AGIR E RECUPERA MAIS RÁPIDO | fôlego = `30 + 5 × Fôlego − penalidade do escudo`; recuperação = `3 + Fôlego / 3` por rodada |
| **Carisma** | MAIS OURO, LOJA MAIS BARATA, PROVOCAÇÃO MAIS FORTE | ver 3.2 |

### 3.2 Carisma (novo)

| Efeito | Fórmula inicial | Teto |
|---|---|---|
| Desconto na loja | `6‰ × c` | 300‰ (30 %) |
| Ouro por vitória | `× (1000 + 10 × c)‰` | +500‰ |
| Valor de venda | `500‰ + 5‰ × c` | 750‰ |
| Provocar: crítico do próximo golpe | `150‰ + 4‰ × c` | 350‰ |
| Provocar: dano do próximo golpe | `× (1100 + 4 × c)‰` | ×1,3 |
| Provocar: abala o oponente | ele perde `2 + c / 4` de fôlego | — |

Preço pago = `preço × (1000 − desconto) / 1000`, arredondado para cima, mínimo 1. O preço de etiqueta
do conteúdo não muda; quem muda é o preço calculado para aquele jogador.

Por que Carisma precisa de efeito em combate: sem isso, uma build de Carisma perderia toda luta
isolada e violaria a meta B2 (nenhuma build abaixo de 25 % contra todas). A provocação mais forte é
esse efeito; a economia é o motivo principal para investir nele.

### 3.3 Onde o jogador vê isso

- **Criação e FICHA:** a linha de descrição mostra o texto do atributo sob o mouse; a coluna da
  direita mostra os derivados (vida, fôlego, dano, acerto contra o próximo oponente, crítico, desconto)
  e pisca em verde o número que mudaria com o próximo `[+]`.
- **Distribuir com confirmação:** os `[+]` e `[-]` mexem num rascunho; **CONFIRMAR** aplica tudo de uma
  vez. Hoje cada clique é definitivo e não há como corrigir um erro.
- **Sem redistribuição** depois de confirmar (ver PD4).

## 4. Intervalo entre lutas: nível, pontos e loja

### 4.1 Resultado

A legenda e a faixa de status mostram, nesta ordem:

1. `VITÓRIA!` ou `DERROTA` (e `LIGA CONCLUÍDA!` no campeão);
2. recompensa: `+92 XP  +57 OURO`;
3. se subiu de nível: segunda linha `NÍVEL 5! +3 PONTOS`.

Botão principal: **CONTINUAR**.

### 4.2 O que acontece ao continuar

| Situação depois da luta | O painel abre? | Em qual aba | Dica na tela PREPARAR |
|---|---|---|---|
| há pontos para distribuir | sim | FICHA | `+3 PONTOS NO MENU` |
| sem pontos, mas há item melhor que o equipado e o ouro alcança | sim | LOJA | `LOJA: ITEM AO SEU ALCANCE` |
| nenhuma das duas | não | — | `CLIQUE EM LUTAR` |

- Ao confirmar os pontos, se também houver item ao alcance, o painel passa sozinho para a LOJA.
- Fechar o painel (Esc, clicar fora ou o botão **PRONTO**) leva a PREPARAR. Nada é obrigatório: dá para
  fechar e lutar com pontos guardados.
- Opção em CONFIG: `ABRIR PAINEL APÓS A LUTA` — QUANDO HOUVER NOVIDADE (padrão) / NUNCA.

### 4.3 Loja

```
┌ FICHA  LOJA  LIGA  VISUAL  CONFIG ──────────────── X ┐
│ OURO 184        PRÓXIMO: MACHADO EM 3 VITÓRIAS       │
│ ARMA                                                 │
│  ESPADA CURTA DE BRONZE   +4 DANO          EQUIPADO  │
│  LANÇA DE FREIXO          +6 DANO (+2)  134$ COMPRAR │
│  MACHADO DE FERRO         +10 DANO (+6) 286$ FALTA 102│
│ ARMADURA                                             │
│  TÚNICA ACOLCHOADA        +4 ARMADURA    39$ COMPRAR │
│  …                                                   │
│ LANÇA: +6 DANO, +2 FÔLEGO POR GOLPE          [PRONTO] │
└──────────────────────────────────────────────────────┘
```

(Exemplo com Carisma 13: desconto de 4,8 %.)

- Itens agrupados por slot, do mais barato ao mais caro.
- Cada linha mostra o efeito e, entre parênteses, **a diferença para o item equipado**.
- Preço já com o desconto do Carisma. Sem ouro suficiente, o botão vira `FALTA n`.
- **COMPRAR** compra e equipa. O item antigo fica no inventário, com botão **VENDER** (valor de venda
  mostrado) e **EQUIPAR**.
- Cabeçalho: ouro e a meta seguinte — o item mais barato que melhora algum slot e quantas vitórias
  contra o próximo oponente faltam para pagá-lo.
- A linha de descrição mostra tudo o que o item sob o mouse muda, inclusive as penalidades (peso,
  agilidade, fôlego), que hoje não aparecem em lugar nenhum.

### 4.4 Nível

- Curva atual mantida: XP para o nível seguinte = `30 + 20 × n + 3 × n²`; 3 pontos por nível; máximo 30.
- A faixa de status em PREPARAR ganha uma barra fina de XP; a FICHA mostra `XP 120/210` (já mostra) e
  `FALTAM 2 VITÓRIAS` contra o próximo oponente.
- Subir de nível nunca acontece "em silêncio": aparece no resultado e leva à FICHA.

### 4.5 Modo automático

Hoje o modo automático distribui pontos sozinho, mas **nunca compra nada**: o gladiador acumula ouro e
para de evoluir. Passa a existir `COMPRAR SOZINHO` (CONFIG, ligado por padrão no modo automático), com
a mesma política do simulador: para cada slot, o item mais caro que o ouro alcança e que seja melhor
que o equipado. No modo automático o painel nunca abre sozinho.

## 5. Economia

### 5.1 De onde vem e para onde vai o ouro

| Entrada | Valor (baseline atual) | Muda neste plano |
|---|---|---|
| Vitória | `15 + 6 × nível do oponente` | × bônus de Carisma |
| Derrota | 25 % da vitória | — |
| Treino (oponente já vencido) | 50 % | — |
| Venda de item | 50 % do preço | 50 % a 75 % com Carisma |
| Prêmio de campeão | não existe | novo: prêmio único ao vencer o campeão da liga (PD3) |
| Ouro inicial | 0 | — |

| Saída | Valor |
|---|---|
| Itens (12 na 1.0) | 30 a 520; total de 2.340 para comprar tudo, sem desconto |

Referência com os números atuais (sem Carisma): ouro por vitória contra cada oponente da liga 1.

| Oponente (nível) | 1 | 2 | 4 | 6 | 8 | 10 | 12 | 14 | 16 |
|---|---|---|---|---|---|---|---|---|---|
| Ouro por vitória | 21 | 27 | 39 | 51 | 63 | 75 | 87 | 99 | 111 |

A primeira compra possível (broquel, 30) vem depois de 2 vitórias; a última (placas, 520) custa cerca
de 5 vitórias no fim da liga. O baseline mede 6,2 vitórias por item comprado (meta P3: 4 a 8) e 1,7 ×
o item mais caro em ouro parado no fim (meta P5: ≤ 2).

### 5.2 Metas novas para o `sim`

As 22 metas de `docs/12` continuam valendo. Entram:

| ID | Métrica | Meta proposta | Por quê |
|---|---|---|---|
| E1 | Lutas até o campeão, build Carisma ÷ build equilibrada | 0,90 – 1,10 | Carisma não é obrigatório nem inútil |
| E2 | Ouro gasto ÷ ouro ganho ao vencer o campeão | ≥ 80 % | o dinheiro tem uso até o fim |
| E3 | Vitórias até a primeira compra | 2 – 4 | primeira recompensa cedo |
| E4 | Desconto médio efetivo da build equilibrada no fim da liga | 5 % – 15 % | o teto de 30 % é para quem investe |

B1 e B2 passam a incluir a build Carisma (50 % dos pontos livres em Carisma).

### 5.3 Itens com requisito de atributo (opcional, PD2)

Dar identidade às builds: armas pesadas exigem Força, armas leves exigem Agilidade. A loja mostraria
`EXIGE FORÇA 12`. Fica fora do escopo até o autor decidir, porque mexe no catálogo e no simulador.

## 6. Impacto no balanceamento congelado

Este plano **muda regras** (novo atributo, economia), então segue o processo de `docs/12`, seção 6:

1. Carisma no valor inicial reproduz as regras de hoje, de propósito: o que muda no baseline é só a
   diluição dos pontos (sete atributos em vez de seis) e a política de compra com desconto.
2. Ajustar com `sim compare` e `sim check` até as 22 metas atuais e as 4 novas passarem. Alavancas, em
   ordem de preferência: constantes de Carisma (3.2) → pontos livres iniciais (10) → pontos por nível (3).
3. `sim write-baseline`, `UPDATE_REPLAYS=1 cargo test -p game --test replays`, registrar em
   "Balanceamento" no `CHANGELOG.md` e colar `balance/compare.md` no PR.
4. **Save:** `SCHEMA_VERSION` 1 → 2. Migração: vetor de atributos de 6 para 7 posições com Carisma = 5;
   `technique` renomeado para `attack`; luta em andamento reiniciada (regra que já existe para mudança
   de versão de balanceamento). Teste com um save v1 real gravado em `crates/game/tests/`.

## 7. Fases de execução

Cada fase termina com `cargo fmt --check`, `cargo clippy -- -D warnings` e `cargo test` verdes, e pode
ser entregue sozinha.

### Fase A — Regras: sete atributos e Carisma (`game`, `sim`)

| # | Tarefa | Onde |
|---|---|---|
| A1 | `Attr::Charisma`, `ATTR_COUNT = 7`; renomear `Technique` → `Attack` (código, JSON, textos) | `fighter.rs`, `content.rs`, `rules.json`, `leagues.json` (7 pesos por oponente), `i18n.rs` |
| A2 | Bloco `charisma` em `rules.json` com as constantes de 3.2; validação na carga | `content.rs`, `docs/14` |
| A3 | Preço e venda calculados: `shop::price(content, player, item)` e `sell_value`; `buy`/`sell` usam | novo `crates/game/src/shop.rs`, `state.rs` |
| A4 | Ouro de vitória com bônus; provocação escalada por Carisma | `progression.rs`, `combat.rs`, `ai.rs` (a IA precisa enxergar a provocação mais forte) |
| A5 | Esquema v2 e migração v1 → v2 | `save.rs`, teste com save v1 |
| A6 | `sim`: build Carisma, métricas E1–E4, política de compra com preço calculado | `crates/sim`, `balance/targets.json` |
| A7 | Ajuste até passar nas metas; novo baseline e replays | `balance/`, `replays.json`, `docs/11`, `docs/12` |

**Aceite:** `sim check` verde com 26 metas; teste de propriedade "Carisma 5 ⇒ preço = etiqueta e ouro
= fórmula base"; save v1 carrega e joga.

### Fase B — Criação do personagem (`game`, `platform`, `render`, `app`)

| # | Tarefa | Onde |
|---|---|---|
| B1 | `GameState::create(content, NewCharacter { name, appearance, attrs }, seed)` com validação (soma dos pontos, limites, nome) | `state.rs` |
| B2 | Texto digitado no painel: `WM_CHAR` → `Event::Char(char)`; Backspace e Enter por `Event::Key` | `platform/src/win/window.rs`, `event.rs` |
| B3 | Widgets do painel: campo de texto com cursor, linha com `[-]` `[+]`, coluna de derivados, linha de descrição, modo assistente (sem abas, com passo `n/3`) | `render/src/scene.rs` |
| B4 | Região sob o mouse no painel (descrições) | `win_main.rs`, `controller.rs` |
| B5 | Estado do aplicativo sem save: título, rascunho de criação, NOVO JOGO com confirmação | `controller.rs`, `win_main.rs`, novo `app/src/creation.rs` |
| B6 | Palco: etapa `Phase::Title`; retrato ao vivo durante a criação | `scene.rs` |
| B7 | Lista própria de nomes para SORTEAR; textos pt-BR/en | `i18n.rs` |

**Aceite:** do executável sem save até a primeira luta só com o mouse (SORTEAR) e também digitando o
nome; fechar o painel no meio não perde o rascunho; nomes inválidos não passam; golden images das
novas telas; teste de zero alocação continua passando no palco.

### Fase C — Intervalo entre lutas (`game`, `render`, `app`)

| # | Tarefa | Onde |
|---|---|---|
| C1 | `FightSummary` guarda os níveis ganhos; resultado mostra `NÍVEL n! +p PONTOS` | `state.rs`, `controller.rs` |
| C2 | Distribuição com rascunho e CONFIRMAR: `Input::AllocatePoints([u8; 7])` (valida a soma e os limites de uma vez) | `state.rs`, `controller.rs` |
| C3 | Derivados ao vivo na FICHA, com destaque do que muda | `controller.rs`, `scene.rs` |
| C4 | Regra de abertura do painel ao continuar (tabela 4.2) e opção em CONFIG | `controller.rs`, `config.rs` |
| C5 | Loja nova: grupos por slot, diferença para o equipado, preço calculado, `FALTA n`, VENDER, meta seguinte, descrição com penalidades, PRONTO | `controller.rs`, `scene.rs` |
| C6 | Rolagem ou paginação do painel (a loja passa de 12 linhas com os cabeçalhos de grupo) | `scene.rs` |
| C7 | Barra de XP na faixa de status de PREPARAR; dicas da tabela 4.2 | `scene.rs`, `controller.rs` |
| C8 | `COMPRAR SOZINHO` no modo automático | `controller.rs`, `config.rs` |

**Aceite:** teste de controlador para cada linha da tabela 4.2; comprar e vender pelo painel bate com
`shop::price`; modo automático conclui a liga 1 comprando sozinho; nenhuma legenda ou linha cortada em
pt-BR e en.

### Fase D — Fechamento

| # | Tarefa |
|---|---|
| D1 | Prêmio de campeão, se PD3 for aprovada, e ajuste final das metas |
| D2 | Dicas da primeira luta (três legendas únicas: andar até o alcance, atacar, recuperar fôlego) |
| D3 | Atualizar `README.md`, `docs/LEIA-ME-USUARIO.txt`, `docs/04`, `docs/09` (matriz manual) e capturas |
| D4 | Rodar o CI manual (MSVC) e validar em Windows a 100 % e 150 % de escala |

### Ordem e dependências

`A → B → C → D`. B e C dependem de A por causa dos sete atributos (telas de atributos e preços).
B3 (widgets) é reaproveitado por C2, C3 e C5. A fase A não toca em interface e pode ser revisada só
pelos números do `sim`.

## 8. Decisões que dependem do autor

Recomendação em negrito. As fases indicadas só começam depois da resposta.

| ID | Decisão | Opções | Bloqueia |
|---|---|---|---|
| PD1 | Lista de atributos | **7: Força, Ataque, Agilidade, Defesa, Vitalidade, Fôlego, Carisma** · manter os 6 atuais e só renomear Técnica · incluir também um atributo de magia (exige sistema de magias, fora da 1.0) | A |
| PD2 | Itens com requisito de atributo (5.3) | **deixar para depois da 1.0** · incluir na fase A | A |
| PD3 | Prêmio de campeão | **sim, valor definido pelo `sim`** · não | D |
| PD4 | Redistribuir pontos depois de confirmados | **não** · sim, pagando ouro (vira mais uma saída de dinheiro) | C |
| PD5 | Mais de um gladiador salvo | **um por vez na 1.0** · vários slots | B |
| PD6 | Abrir o painel sozinho depois da luta | **quando houver novidade** · sempre · nunca | C |

## 9. Riscos

| Risco | Medida |
|---|---|
| O sétimo atributo dilui os pontos e desloca várias metas de uma vez | Carisma neutro no valor inicial; ajustar uma alavanca por vez e comparar com `sim compare` |
| Carisma vira obrigatório (economia) ou inútil (combate) | metas E1 e B2 com a build Carisma; tetos em todos os efeitos |
| Painel de 200 × 150 px lógicos não comporta a loja nova | C6 (rolagem) antes de C5; conferir em golden image com os nomes mais longos em pt-BR |
| Texto digitado: teclado morto e acentos | usar `WM_CHAR` (o Windows já entrega o caractere composto); aceitar só o que a fonte desenha |
| Saves v1 dos testadores | migração testada com arquivo real; `save.json.bak` preservado na primeira gravação em v2 |
