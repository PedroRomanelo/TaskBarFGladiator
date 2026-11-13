# TaskBarFGladiator

Jogo **original** de gladiador por turnos que roda dentro da área da **barra de tarefas do Windows**,
100 % offline, escrito em **Rust**, com foco em segurança, baixo consumo de memória e CPU, e estabilidade
para ficar aberto o dia inteiro.

> **Status:** em desenvolvimento. Concluídos: Fase 0 (repositório, CI, licenças) e o núcleo do jogo
> (crate `game`: combate, IA, progressão, save) com o simulador de balanceamento e o baseline congelado.
> Renderização e arte prontas (crates `render` e `assets`). Falta a integração com o Windows
> (crate `platform` e executável): o jogo ainda não é jogável.
>
> O nome "TaskBarFGladiator" é provisório. O nome definitivo do jogo ainda não foi escolhido
> (pendência P1 em [`docs/10-cronograma-e-decisoes.md`](docs/10-cronograma-e-decisoes.md)).

## O que é

- Uma faixa de pixel art posicionada sobre uma área livre da barra de tarefas, onde acontecem lutas de
  gladiadores em uma arena de uma dimensão (avançar, recuar, golpes leves/médios/pesados, defesa, fôlego).
- Progressão de personagem (atributos, nível, equipamentos, loja, ligas de torneio).
- Modo **manual** (você escolhe cada ação com o mouse) e modo **automático** (a IA luta por você com a
  estratégia que você escolheu, enquanto você trabalha).
- Um painel expandido (janela acima da barra) para ficha, loja, torneios e configurações.

### Pré-visualização (renderização atual, gerada por `cargo run -p render --example preview`)

Faixa em luta, modo manual (barra de tarefas de 72 px, tema escuro):

![Faixa em luta](docs/img/faixa-luta-manual.png)

Faixa fora de luta (48 px, tema claro):

![Faixa fora de luta](docs/img/faixa-hub-tema-claro.png)

Painel expandido (aba Loja):

![Painel](docs/img/painel-loja.png)

## O que não é

- Não é um port, remake, clone ou "recriação" de nenhum jogo existente. Não contém nome, personagens, arte,
  música, textos nem código de terceiros. As mecânicas são de gênero (RPG de gladiador por turnos) e todo o
  conteúdo é próprio ou de licença livre com crédito registrado.

## Metas técnicas

| Métrica | Meta |
|---|---|
| Memória (Private Bytes) | < 15 MB em uso normal |
| CPU parado | 0,0 % |
| CPU em combate animado | < 1 % de um núcleo |
| Executável | 1 arquivo, < 3 MB, portátil |
| Rede | nenhuma |
| Privilégios | usuário comum (nunca pede administrador) |

Plataformas alvo: Windows 10 22H2 e Windows 11 (23H2, 24H2 e posteriores).

## Documentação

Toda a documentação de planejamento está em [`docs/`](docs/README.md):

| Documento | Conteúdo |
|---|---|
| [00 — Visão geral](docs/00-visao-geral.md) | Objetivo, escopo, metas, aspectos legais |
| [01 — Arquitetura](docs/01-arquitetura.md) | Workspace, crates, renderização, loop, dependências |
| [02 — Fase 0: Preparação](docs/02-fase-0-preparacao.md) | Repositório, CI, licenças, nome |
| [03 — Fase 1: Plataforma](docs/03-fase-1-plataforma.md) | Janela na barra de tarefas e todos os casos de borda |
| [04 — Fase 2: Jogo](docs/04-fase-2-jogo.md) | Lógica, save, renderização, interface, balanceamento |
| [05 — Fase 3: Distribuição](docs/05-fase-3-distribuicao.md) | Polimento, soak test, release assinada |
| [06 — Casos de borda](docs/06-casos-de-borda.md) | Catálogo consolidado |
| [07 — Segurança](docs/07-seguranca.md) | Modelo de ameaças e medidas |
| [08 — Memória e desempenho](docs/08-memoria-e-desempenho.md) | Orçamentos, riscos e medição |
| [09 — Verificação e testes](docs/09-verificacao-e-testes.md) | CI, matriz manual, métricas |
| [10 — Cronograma e decisões](docs/10-cronograma-e-decisoes.md) | Prazos, decisões tomadas e pendentes |
| [11 — Design do jogo](docs/11-design-do-jogo.md) | GDD inicial: atributos, ações, fórmulas, IA |
| [12 — Balanceamento de referência](docs/12-balanceamento-de-referencia.md) | Metas, congelamento das regras, processo de alteração |
| [14 — Formato do conteúdo](docs/14-formato-do-conteudo.md) | Arquivos JSON de regras, itens e ligas |

## Licença

Código sob `MIT OR Apache-2.0` ([`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE)).
Arte própria sob `CC-BY-4.0` ([`assets-src/LICENSE`](assets-src/LICENSE)).
