# 16 — Catálogo de itens e pedido de arte

**Status:** proposta. Os níveis de desbloqueio seguem o pedido do autor; os bônus, a armadura, o dano e
os preços são **preliminares**: saem de uma regra simples (seção 3) e serão ajustados pelo simulador
antes de entrar no jogo. Nada deste documento está implementado.

Para que serve: saber **quais itens existem, em que nível aparecem, o que cada um faz e como pedir a
arte de cada um** (seções 5 a 7).

## 1. Resumo

| Grupo | Tipos | Qualidades | Itens |
|---|---|---|---|
| Armadura | capacete, ombreiras, peitoral, manoplas, escudo, coxotes, caneleiras, botas | 8 por tipo | 64 |
| Armas | espada, machado, clava | 8 por tipo | 24 |
| **Total** | | | **88** |

O equipamento é fixo: o mesmo item tem o mesmo desenho e os mesmos números para qualquer jogador. Cada
qualidade é melhor que a anterior em tudo o que importa para o tipo; as mais pesadas cobram um pouco
de Agilidade ou Fôlego.

## 2. Nível de desbloqueio

| Qualidade | Q1 | Q2 | Q3 | Q4 | Q5 | Q6 | Q7 | Q8 |
|---|---|---|---|---|---|---|---|---|
| Todos os tipos, menos manoplas | 1 | 4 | 6 | 8 | 10 | 12 | 14 | 16 |
| Manoplas (pula o nível 10) | 1 | 4 | 6 | 8 | 12 | 14 | 16 | 18 |

- "Desbloqueia" = aparece na loja e pode ser comprado a partir desse nível.
- Q1 no nível 1 é uma suposição (o pedido define a partir do segundo). As armas seguem a mesma
  tabela dos tipos de armadura, também por suposição. Ver seção 9.

## 3. Atributos e regra dos números

| Sigla | Atributo | O que faz (doc 15, seção 3) |
|---|---|---|
| FOR | Força (Strength) | aumenta o dano dos golpes |
| AGI | Agilidade (Agility) | dificulta ser acertado; age primeiro |
| ATQ | Ataque (Attack) | aumenta a chance de acertar e de crítico |
| DEF | Defesa (Defence) | reduz o dano recebido |
| VIT | Vitalidade (Vitality) | aumenta a vida |
| CAR | Carisma (Charisma) | mais ouro, loja mais barata, provocação mais forte |
| FÔL | Fôlego (Stamina) | mais fôlego para agir e recupera mais rápido |

Além dos atributos, cada peça de armadura tem **ARM** (armadura: soma na absorção de dano, como hoje)
e cada arma tem **DANO** (soma no dano base) e **ACERTO** (ajuste na chance de acertar).

Regra usada para gerar as tabelas:

- **Pontos de bônus por qualidade:** Q1 0, Q2 1, Q3 1, Q4 2, Q5 2, Q6 3, Q7 3, Q8 4. Cerca de 60 % vão
  para o atributo principal do tipo e o resto para o secundário.
- **Penalidades:** peitoral cobra Agilidade (até −2), escudo e clava cobram Fôlego (até −2), machado
  cobra Agilidade (até −2).
- **ARM no Q8:** peitoral 12, escudo 10, capacete 6, ombreiras 5, coxotes 5, manoplas 4, caneleiras 4,
  botas 4. Total 50, o mesmo teto de hoje (armadura 28 + escudo 22); as qualidades menores são
  proporcionais.
- **Preço:** base por qualidade (15, 40, 75, 120, 180, 260, 360, 500) vezes o peso do tipo
  (peitoral 1,6; armas 1,3 a 1,4; escudo 1,3; capacete 1,0; os demais 0,7 a 0,8).

Referência: o conjunto completo de armadura Q8 dá +4 FOR, +2 AGI, +3 ATQ, +14 DEF, +5 VIT, +1 CAR e
−1 FÔL (28 pontos). Um gladiador de nível 16 tem cerca de 90 pontos próprios, então o equipamento
completo vale perto de um terço disso. Comprar todas as 64 peças de armadura custa cerca de 11.900 de
ouro, contra 2.340 de todos os itens de hoje: a economia do doc 15 (seção 5) precisa ser refeita junto.

## 4. Tabelas

Cada linha é um item a desenhar. `·` = sem efeito.

### 4.1 Escada de materiais

Igual para todos os tipos. Cada qualidade tem uma cor dominante própria, para ser reconhecida no
lutador pequeno (o corpo tem ~36 px de altura): a sequência alterna quente e frio, claro e escuro, e
nenhuma qualidade se confunde com a vizinha. A silhueta cresce junto: o Q1 é justo e simples; o Q8 tem
volume e enfeites.

| Q | Material | Cor dominante | Silhueta | Frase para a descrição |
|---|---|---|---|---|
| Q1 | Couro cru | marrom claro, opaco | justo, costuras à vista | couro cru gasto, costuras grossas, remendos |
| Q2 | Couro fervido com rebites | marrom escuro com pontos de ferro | bordas reforçadas, tiras e fivelas | couro endurecido escuro com rebites de ferro e tiras |
| Q3 | Bronze | laranja-dourado quente, pátina verde | primeiras placas, bordas arredondadas | placas de bronze martelado sobre couro, pátina esverdeada nas bordas |
| Q4 | Ferro | cinza escuro, fosco | placas maiores, rebites grandes | ferro forjado escuro, marcas de martelo, aspecto rústico |
| Q5 | Aço | cinza azulado, brilho forte | bordas dobradas; entra o tecido na cor do time | aço polido com reflexos, faixa de tecido na cor do time |
| Q6 | Aço negro | quase preto com filetes vermelhos | pontas e arestas agressivas | aço enegrecido com filetes vermelhos e pontas afiadas |
| Q7 | Osso de fera | marfim amarelado com couro escuro | presas, chifres, placas de osso amarradas | ossos e presas de uma fera gigante, amarrados com couro escuro, marcas tribais |
| Q8 | Ouro branco celeste | branco perolado com azul-céu | filigrana, pequenas asas, gema que brilha | ouro branco perolado com filigrana azul-céu, pequenas asas e uma gema luminosa |

- O aço negro (Q6) escurece de propósito antes do osso: o contraste faz o Q7 e o Q8 se destacarem.
- O osso do Q7 é de uma fera lendária, um troféu, e não um material barato.
- Escudos podem usar madeira no lugar de couro no Q1 e no Q2.
- As cores de cada material estão em `docs/arte/paleta.png` (seção 6).

### 4.2 Armadura

#### Capacetes (HELMET)

Bônus principal: DEF; secundário: CAR.

| Q | Nível | Nome | FOR | AGI | ATQ | DEF | VIT | CAR | FÔL | ARM | Preço |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Q1 | 1 | Capacete de couro cru | · | · | · | · | · | · | · | 1 | 15 |
| Q2 | 4 | Capacete de couro fervido | · | · | · | +1 | · | · | · | 2 | 40 |
| Q3 | 6 | Capacete de bronze | · | · | · | +1 | · | · | · | 2 | 75 |
| Q4 | 8 | Capacete de ferro | · | · | · | +2 | · | · | · | 3 | 120 |
| Q5 | 10 | Capacete de aço | · | · | · | +2 | · | · | · | 4 | 180 |
| Q6 | 12 | Capacete de aço negro | · | · | · | +2 | · | +1 | · | 5 | 260 |
| Q7 | 14 | Capacete de osso de fera | · | · | · | +2 | · | +1 | · | 5 | 360 |
| Q8 | 16 | Capacete de ouro branco celeste | · | · | · | +3 | · | +1 | · | 6 | 500 |

#### Ombreiras (SHOULDERGUARDS)

Bônus principal: FOR; secundário: DEF.

| Q | Nível | Nome | FOR | AGI | ATQ | DEF | VIT | CAR | FÔL | ARM | Preço |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Q1 | 1 | Ombreiras de couro cru | · | · | · | · | · | · | · | 1 | 10 |
| Q2 | 4 | Ombreiras de couro fervido | +1 | · | · | · | · | · | · | 1 | 30 |
| Q3 | 6 | Ombreiras de bronze | +1 | · | · | · | · | · | · | 2 | 60 |
| Q4 | 8 | Ombreiras de ferro | +2 | · | · | · | · | · | · | 3 | 95 |
| Q5 | 10 | Ombreiras de aço | +2 | · | · | · | · | · | · | 3 | 145 |
| Q6 | 12 | Ombreiras de aço negro | +2 | · | · | +1 | · | · | · | 4 | 210 |
| Q7 | 14 | Ombreiras de osso de fera | +2 | · | · | +1 | · | · | · | 4 | 290 |
| Q8 | 16 | Ombreiras de ouro branco celeste | +3 | · | · | +1 | · | · | · | 5 | 400 |

#### Peitorais (BREASTPLATE)

Bônus principal: DEF; secundário: VIT; penalidade: AGI.

| Q | Nível | Nome | FOR | AGI | ATQ | DEF | VIT | CAR | FÔL | ARM | Preço |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Q1 | 1 | Peitoral de couro cru | · | · | · | · | · | · | · | 2 | 25 |
| Q2 | 4 | Peitoral de couro fervido | · | · | · | +1 | · | · | · | 3 | 65 |
| Q3 | 6 | Peitoral de bronze | · | · | · | +1 | · | · | · | 5 | 120 |
| Q4 | 8 | Peitoral de ferro | · | −1 | · | +2 | · | · | · | 6 | 190 |
| Q5 | 10 | Peitoral de aço | · | −1 | · | +2 | · | · | · | 8 | 290 |
| Q6 | 12 | Peitoral de aço negro | · | −1 | · | +2 | +1 | · | · | 9 | 415 |
| Q7 | 14 | Peitoral de osso de fera | · | −2 | · | +2 | +1 | · | · | 11 | 575 |
| Q8 | 16 | Peitoral de ouro branco celeste | · | −2 | · | +3 | +1 | · | · | 12 | 800 |

#### Manoplas (GAUNTLETS)

Bônus principal: ATQ; secundário: FOR.

| Q | Nível | Nome | FOR | AGI | ATQ | DEF | VIT | CAR | FÔL | ARM | Preço |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Q1 | 1 | Manoplas de couro cru | · | · | · | · | · | · | · | 1 | 10 |
| Q2 | 4 | Manoplas de couro fervido | · | · | +1 | · | · | · | · | 1 | 30 |
| Q3 | 6 | Manoplas de bronze | · | · | +1 | · | · | · | · | 2 | 60 |
| Q4 | 8 | Manoplas de ferro | · | · | +2 | · | · | · | · | 2 | 95 |
| Q5 | 12 | Manoplas de aço | · | · | +2 | · | · | · | · | 3 | 145 |
| Q6 | 14 | Manoplas de aço negro | +1 | · | +2 | · | · | · | · | 3 | 210 |
| Q7 | 16 | Manoplas de osso de fera | +1 | · | +2 | · | · | · | · | 4 | 290 |
| Q8 | 18 | Manoplas de ouro branco celeste | +1 | · | +3 | · | · | · | · | 4 | 400 |

#### Escudos (SHIELD)

Bônus principal: DEF; secundário: VIT; penalidade: FOL.

| Q | Nível | Nome | FOR | AGI | ATQ | DEF | VIT | CAR | FÔL | ARM | Preço |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Q1 | 1 | Escudo de couro cru | · | · | · | · | · | · | · | 1 | 20 |
| Q2 | 4 | Escudo de couro fervido | · | · | · | +1 | · | · | · | 3 | 50 |
| Q3 | 6 | Escudo de bronze | · | · | · | +1 | · | · | −1 | 4 | 100 |
| Q4 | 8 | Escudo de ferro | · | · | · | +2 | · | · | −1 | 5 | 155 |
| Q5 | 10 | Escudo de aço | · | · | · | +2 | · | · | −1 | 6 | 235 |
| Q6 | 12 | Escudo de aço negro | · | · | · | +2 | +1 | · | −2 | 8 | 340 |
| Q7 | 14 | Escudo de osso de fera | · | · | · | +2 | +1 | · | −2 | 9 | 470 |
| Q8 | 16 | Escudo de ouro branco celeste | · | · | · | +3 | +1 | · | −2 | 10 | 650 |

#### Coxotes (GREAVES)

Bônus principal: VIT; secundário: DEF.

| Q | Nível | Nome | FOR | AGI | ATQ | DEF | VIT | CAR | FÔL | ARM | Preço |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Q1 | 1 | Coxotes de couro cru | · | · | · | · | · | · | · | 1 | 10 |
| Q2 | 4 | Coxotes de couro fervido | · | · | · | · | +1 | · | · | 1 | 30 |
| Q3 | 6 | Coxotes de bronze | · | · | · | · | +1 | · | · | 2 | 60 |
| Q4 | 8 | Coxotes de ferro | · | · | · | · | +2 | · | · | 3 | 95 |
| Q5 | 10 | Coxotes de aço | · | · | · | · | +2 | · | · | 3 | 145 |
| Q6 | 12 | Coxotes de aço negro | · | · | · | +1 | +2 | · | · | 4 | 210 |
| Q7 | 14 | Coxotes de osso de fera | · | · | · | +1 | +2 | · | · | 4 | 290 |
| Q8 | 16 | Coxotes de ouro branco celeste | · | · | · | +1 | +3 | · | · | 5 | 400 |

#### Caneleiras (SHINGUARDS)

Bônus principal: DEF; secundário: AGI.

| Q | Nível | Nome | FOR | AGI | ATQ | DEF | VIT | CAR | FÔL | ARM | Preço |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Q1 | 1 | Caneleiras de couro cru | · | · | · | · | · | · | · | 1 | 10 |
| Q2 | 4 | Caneleiras de couro fervido | · | · | · | +1 | · | · | · | 1 | 30 |
| Q3 | 6 | Caneleiras de bronze | · | · | · | +1 | · | · | · | 2 | 55 |
| Q4 | 8 | Caneleiras de ferro | · | · | · | +2 | · | · | · | 2 | 85 |
| Q5 | 10 | Caneleiras de aço | · | · | · | +2 | · | · | · | 3 | 125 |
| Q6 | 12 | Caneleiras de aço negro | · | +1 | · | +2 | · | · | · | 3 | 180 |
| Q7 | 14 | Caneleiras de osso de fera | · | +1 | · | +2 | · | · | · | 4 | 250 |
| Q8 | 16 | Caneleiras de ouro branco celeste | · | +1 | · | +3 | · | · | · | 4 | 350 |

#### Botas (BOOTS)

Bônus principal: AGI; secundário: FOL.

| Q | Nível | Nome | FOR | AGI | ATQ | DEF | VIT | CAR | FÔL | ARM | Preço |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Q1 | 1 | Botas de couro cru | · | · | · | · | · | · | · | 1 | 10 |
| Q2 | 4 | Botas de couro fervido | · | +1 | · | · | · | · | · | 1 | 30 |
| Q3 | 6 | Botas de bronze | · | +1 | · | · | · | · | · | 2 | 55 |
| Q4 | 8 | Botas de ferro | · | +2 | · | · | · | · | · | 2 | 85 |
| Q5 | 10 | Botas de aço | · | +2 | · | · | · | · | · | 3 | 125 |
| Q6 | 12 | Botas de aço negro | · | +2 | · | · | · | · | +1 | 3 | 180 |
| Q7 | 14 | Botas de osso de fera | · | +2 | · | · | · | · | +1 | 4 | 250 |
| Q8 | 16 | Botas de ouro branco celeste | · | +3 | · | · | · | · | +1 | 4 | 350 |

### 4.3 Armas

Nas armas, o Q1 é de madeira (ou pedra) e o couro aparece só no cabo; do Q3 em diante seguem a mesma escada da armadura. A coluna Aparência é o ponto de partida da descrição.

#### Espadas (SWORD)

Bônus principal: ATQ; secundário: AGI.

| Q | Nível | Nome | FOR | AGI | ATQ | DEF | VIT | CAR | FÔL | DANO | ACERTO | Preço | Aparência |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Q1 | 1 | Espada de madeira | · | · | · | · | · | · | · | 3 | +5% | 20 | madeira de treino com cabo enrolado em couro cru |
| Q2 | 4 | Gládio gasto | · | · | +1 | · | · | · | · | 5 | +5% | 55 | lâmina curta de ferro gasto e lascado, cabo de couro com rebites |
| Q3 | 6 | Gládio de bronze | · | · | +1 | · | · | · | · | 7 | +4% | 105 | lâmina curta e larga de bronze com pátina verde, cabo de madeira |
| Q4 | 8 | Espada longa de ferro | · | · | +2 | · | · | · | · | 9 | +4% | 170 | lâmina longa de ferro escuro forjado, guarda em cruz |
| Q5 | 10 | Espada de aço | · | · | +2 | · | · | · | · | 11 | +3% | 250 | lâmina de aço polido com reflexos, faixa de tecido na cor do time no cabo |
| Q6 | 12 | Espada de aço negro | · | +1 | +2 | · | · | · | · | 13 | +3% | 365 | lâmina de aço enegrecido com filete vermelho, guarda pontiaguda |
| Q7 | 14 | Espada de osso de fera | · | +1 | +2 | · | · | · | · | 15 | +3% | 505 | lâmina serrilhada feita de uma presa de fera gigante, cabo de osso amarrado com couro escuro |
| Q8 | 16 | Espada de ouro branco celeste | · | +1 | +3 | · | · | · | · | 18 | +3% | 700 | lâmina de ouro branco perolado com filigrana azul-céu, guarda em forma de asas e gema luminosa no pomo |

#### Machados (AXE)

Bônus principal: FOR; secundário: ATQ; penalidade: AGI.

| Q | Nível | Nome | FOR | AGI | ATQ | DEF | VIT | CAR | FÔL | DANO | ACERTO | Preço | Aparência |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Q1 | 1 | Machado de pedra | · | · | · | · | · | · | · | 4 | −2% | 20 | cabeça de pedra lascada amarrada com couro cru num cabo de madeira |
| Q2 | 4 | Machadinha gasta | +1 | · | · | · | · | · | · | 6 | −3% | 55 | cabeça pequena de ferro gasto, cabo curto com tiras de couro |
| Q3 | 6 | Machado de bronze | +1 | −1 | · | · | · | · | · | 9 | −4% | 105 | cabeça de bronze em meia-lua com pátina verde, cabo de freixo |
| Q4 | 8 | Machado barbado de ferro | +2 | −1 | · | · | · | · | · | 11 | −4% | 170 | lâmina longa de ferro escuro em forma de barba, cabo reforçado com anéis |
| Q5 | 10 | Machado de aço | +2 | −1 | · | · | · | · | · | 14 | −5% | 250 | lâmina de aço polido com reflexos, cabo escuro com tecido na cor do time |
| Q6 | 12 | Machado de aço negro | +2 | −1 | +1 | · | · | · | · | 16 | −5% | 365 | duas lâminas de aço enegrecido com filetes vermelhos e pontas |
| Q7 | 14 | Machado de osso de fera | +2 | −2 | +1 | · | · | · | · | 19 | −6% | 505 | lâmina feita da omoplata de uma fera gigante, presas na ponta do cabo, amarrações de couro escuro |
| Q8 | 16 | Machado de ouro branco celeste | +3 | −2 | +1 | · | · | · | · | 22 | −6% | 700 | lâmina de ouro branco perolado com filigrana azul-céu em forma de asa, gema luminosa no centro |

#### Clavas (CLUB)

Bônus principal: FOR; secundário: VIT; penalidade: FOL.

| Q | Nível | Nome | FOR | AGI | ATQ | DEF | VIT | CAR | FÔL | DANO | ACERTO | Preço | Aparência |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Q1 | 1 | Porrete de carvalho | · | · | · | · | · | · | · | 3 | 0% | 20 | tronco de carvalho grosso e simples, cabo enrolado em couro cru |
| Q2 | 4 | Clava com pregos | +1 | · | · | · | · | · | · | 6 | −1% | 50 | madeira escura com pregos de ferro gastos cravados |
| Q3 | 6 | Clava com anéis de bronze | +1 | · | · | · | · | · | · | 8 | −1% | 100 | madeira com anéis de bronze com pátina verde na ponta |
| Q4 | 8 | Maça de ferro | +2 | · | · | · | · | · | −1 | 10 | −2% | 155 | cabeça de ferro escuro maciça, marcas de martelo, cabo de madeira |
| Q5 | 10 | Maça flangeada de aço | +2 | · | · | · | · | · | −1 | 13 | −2% | 235 | cabeça de aço polido com abas (flanges), tecido na cor do time no cabo |
| Q6 | 12 | Maça estrelada de aço negro | +2 | · | · | · | +1 | · | −1 | 15 | −3% | 340 | cabeça de aço enegrecido com pontas em estrela e filetes vermelhos |
| Q7 | 14 | Clava de osso de fera | +2 | · | · | · | +1 | · | −2 | 17 | −3% | 470 | fêmur de fera gigante com presas cravadas na ponta, amarrações de couro escuro |
| Q8 | 16 | Maça de ouro branco celeste | +3 | · | · | · | +1 | · | −2 | 20 | −3% | 650 | cabeça de ouro branco perolado com filigrana azul-céu e pequenas asas, gema luminosa |

## 5. Que imagens pedir para cada item

### 5.1 Ícone da loja (os 88 itens)

Uma imagem por item: o item sozinho, de lado, virado para a direita. É o que aparece na loja e no
inventário. É o tipo de imagem que um gerador como o Gemini faz razoavelmente bem, com a limpeza da
seção 7. Use o prompt da seção 6.

Nome do arquivo: `assets-src/itens/<tipo>/q<n>.png`, com o tipo em minúsculas, como na tabela
(`helmet`, `shoulderguards`, `breastplate`, `gauntlets`, `shield`, `greaves`, `shinguards`, `boots`,
`sword`, `axe`, `club`). Exemplo: `assets-src/itens/helmet/q3.png` para o Capacete de bronze.

### 5.2 Peça vestida no lutador

Para a peça aparecer no corpo do lutador, ela precisa ser desenhada **em cima de cada pose da
animação**, no tamanho do corpo (o cavaleiro tem ~36 px de altura; um capacete vestido tem uns
12 × 12 px). Isso exige, antes de tudo, um **corpo base sem armadura** no mesmo estilo, com todas as
animações.

O volume é grande: mesmo reduzindo as animações a 8 poses-chave (parado, passo A, passo B, preparo,
golpe, guarda, dano, caído), são 88 itens × 8 poses = **704 desenhos encaixados**. Geradores de imagem
não mantêm proporção, ângulo e posição entre uma imagem e outra, então essa parte precisa de um artista
de pixel art (ou de muito retoque à mão no Aseprite).

Enquanto isso não existe, o jogo pode mostrar o equipamento no corpo só pela troca de cores (material
da armadura, já funciona), com os ícones na loja. Ver seção 9.

## 6. Prompt padrão para o Gemini

Escreva primeiro o que você quer, em uma ou duas frases, e cole o prompt logo depois. Exemplo de
abertura:

> Capacete de couro surrado, costuras grossas e uma faixa de couro vermelho na testa. Qualidade Q1, o
> mais simples do jogo.

Anexe junto `docs/arte/paleta.png` (as cores do jogo; cada linha é uma rampa, do escuro ao claro: contorno e aço, vermelho do time e ouro, azul, pele, couro, bronze e pátina, aço negro, osso de fera, ouro branco celeste, tecido e roxo do oponente) e, quando houver, um item já aprovado do mesmo
tipo, para manter o estilo.

```text
Create a single pixel art game item sprite of the item described above, for a 2D side-view
gladiator game.

SUBJECT
- Show ONLY this one item, alone. Not worn, no character, no hands, no mannequin, no stand.
- No text, no letters, no UI, no frame, no border, no logo, no watermark.
- Side view, facing right, with a slight 3/4 angle, like equipment in a 16-bit side-scrolling RPG.
- Centered, filling about 80% of the canvas, with empty margin on every side.

PIXEL GRID (most important)
- True pixel art drawn on a 32 x 32 pixel grid, then scaled up 16x with nearest neighbor to a
  512 x 512 image. Every art pixel must be a perfect 16 x 16 square aligned to the grid.
- No anti-aliasing, no blur, no gradients, no soft shading, no noise, no glow, no texture filters.

COLOR AND LIGHT
- Use only colors from the attached palette image. At most 12 colors in the whole item.
- Flat color areas with 3 to 4 tone ramps per material; shadows shift toward purple/blue,
  highlights toward warm yellow.
- One light source from the top-left. Small bright highlights on metal edges.
- 1-pixel dark outline around the silhouette using dark navy #1a1932 (not pure black); lighter
  outline segments where light hits the edge.

BACKGROUND
- Solid flat pure magenta #FF00FF filling the entire background. No checkerboard, no gradient, no
  floor, no cast shadow.
- Do not use magenta or pink anywhere on the item.

STYLE
- High-quality modern indie pixel art, crisp and readable at small size, clear silhouette,
  consistent with the attached reference item if one is attached.
```

Para **armas**, acrescente ao fim: `Weapon shown diagonally, handle at bottom-left, tip or head at
top-right.`

Para manter a **série** (Q1 a Q8 do mesmo tipo parecendo da mesma família), gere o Q1, aprove, e nas
seguintes anexe o anterior com: `Same item family as the attached reference: keep the same size,
angle, outline and palette, but upgrade the material and detail as described above.`

## 7. Depois de gerar: conferência

1. A imagem tem 512 × 512 e os "pixels" são quadrados iguais? Se o gerador entregou outro tamanho ou
   pixels tortos, gere de novo ou corrija no Aseprite.
2. Reduza para 32 × 32 com vizinho mais próximo (nearest neighbor). Nada de "bilinear" ou "bicúbico".
3. Apague o fundo magenta (varinha mágica sem suavização). Confira que não sobrou contorno rosado.
4. Conte as cores (no Aseprite: Sprite > Color Mode > Indexed). Mais de 12, ou cores fora da paleta:
   ajuste.
5. Salve como PNG com transparência em `assets-src/itens/<tipo>/q<n>.png` e anote a origem em
   `assets-src/CREDITS.md`.

Os passos 2 a 4 podem virar um script (reduzir para a grade, trocar o magenta por transparente e
aproximar as cores da paleta), ainda não feito.

## 8. O que muda no jogo para isso existir

Nada disto está no código. Para entrar, além da arte:

- **Conteúdo:** `items.json` passa de 3 tipos (arma, armadura, escudo) para 11 tipos, com nível
  mínimo, bônus de atributo, ARM, DANO e ACERTO. As armas de hoje (faca, lança, arco, bastão) saem ou
  viram tipos extras.
- **Regras:** a absorção soma o ARM de todas as peças; os bônus somam nos atributos; as penalidades
  de peso de hoje viram bônus negativos de atributo. Depende dos sete atributos da fase A do doc 15.
- **Loja:** 11 grupos e 88 itens pedem rolagem ou filtro por tipo, e mostrar só o que já está
  desbloqueado mais o próximo.
- **Balanceamento:** muda o baseline congelado (doc 12): novas metas no simulador, preços e ouro por
  vitória ajustados juntos, e o processo de troca de baseline.

## 9. Decisões em aberto

| ID | Pergunta | Suposição usada aqui |
|---|---|---|
| I1 | O Q1 de cada tipo fica disponível desde o nível 1 (ou é o equipamento inicial)? | nível 1, comprável |
| I2 | As armas também são 8 por tipo, com os mesmos níveis da armadura? | sim |
| I3 | GREAVES e SHINGUARDS: as duas protegem a perna. Aqui GREAVES = coxotes (coxa) e SHINGUARDS = caneleiras (canela). Está certo? | sim |
| I4 | Peças vestidas: contratar artista para o corpo base e as 704 poses, ou começar só com ícones e troca de cor? | começar com ícones e troca de cor |
| I5 | Faca, lança, arco e bastão saem do jogo? | saem |
