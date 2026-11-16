# Corpo do lutador em folhas de sprites

Cada pasta aqui dentro é um conjunto de animações do corpo do lutador. O `crates/assets/build.rs` lê
as folhas PNG no build e as embute no executável num formato próprio (o executável não tem
decodificador de PNG). O jogo usa o primeiro conjunto em ordem alfabética; sem nenhum, usa o boneco em
camadas desenhado em texto (`crates/assets/src/parts.rs`).

A pasta `knight/` (pacote "FREE Knight 2D Pixel Art") **não vai para o repositório**: a licença dele
permite usar e modificar em um jogo, mas não redistribuir os arquivos como asset, e o repositório é
público. Ela está no `.gitignore`; quem quiser ver o jogo com ela precisa baixar o pacote e copiar os
PNG e o `set.json` para cá.

## Como montar um conjunto

1. Uma pasta por conjunto, com um PNG por animação e um `set.json` (use `knight/set.json` de modelo).
2. Cada PNG é uma fila de quadros do mesmo tamanho, lado a lado, da esquerda para a direita.
3. Os pés ficam na mesma linha em todos os quadros (`floor_y` = primeira linha abaixo dos pés) e o
   eixo do corpo na mesma coluna (`anchor_x`). O lutador olha para a **direita**; o oponente é
   espelhado pelo jogo.
4. 1 pixel da arte = 1 pixel da imagem. Na barra de 48 px o jogo amplia 3×, então um corpo de
   ~36 px de altura fica com ~108 px na tela. Não exporte ampliado.
5. Fundo transparente de verdade (canal alfa). Pixels semitransparentes viram transparentes abaixo
   de 50 % e opacos acima: sem antisserrilhado nas bordas.
6. No máximo 63 cores no conjunto inteiro.
7. Animações reconhecidas: `idle` (obrigatória), `walk`, `attack_light`, `attack_medium`,
   `attack_heavy`, `guard` (postura de defesa, repete), `block` (golpe aparado), `hurt`, `death` (para
   no último quadro). As que faltarem usam a mais parecida.
8. `roles` lista as cores que o jogo troca: `team` (3 tons: capa, penacho), `skin` (3 tons), `eyes`
   (1) e `armor` (3 tons do material da armadura). Cada tom precisa ser uma cor que só aparece naquele
   papel.

Para conferir: `cargo run -p render --example preview -- <pasta>` gera `cavaleiro_animacoes.png`
(todas as animações quadro a quadro), `cavaleiro_variacoes.png` (trocas de paleta) e o palco em cada
etapa.

## Peças geradas por IA (Gemini e parecidos)

Funciona como ponto de partida, mas a imagem gerada não é pixel art de verdade e precisa de limpeza
antes de entrar aqui:

- **Grade:** o gerador desenha "pixels" de tamanhos diferentes e fora de alinhamento. É preciso
  reduzir para a resolução real (nearest neighbor) e corrigir à mão no Aseprite ou similar.
- **Cores:** sai com centenas de tons. Reduza para a paleta do conjunto (as rampas de cor de
  `knight/set.json` são a referência de estilo).
- **Fundo:** o "transparente" que o gerador mostra costuma ser um xadrez cinza pintado. Remova o fundo
  e confira que o PNG tem canal alfa, sem halo claro em volta do contorno.
- **Consistência:** cada geração muda proporção, ângulo e detalhes. Para animação e para peças que se
  encaixam (elmo, arma, escudo), os quadros precisam ter o mesmo tamanho, o mesmo eixo e os pés na
  mesma linha; isso quase sempre exige redesenho manual.
- **Licença:** confira os termos do gerador para uso comercial e registre a origem em
  `assets-src/CREDITS.md`. Não peça "no estilo de" um jogo específico.
