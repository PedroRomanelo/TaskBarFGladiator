# 00 — Visão geral

## 1. Objetivo

Construir um jogo de gladiador por turnos, original, que:

- roda **dentro da área visual da barra de tarefas do Windows**, como uma faixa de pixel art posicionada
  sobre um espaço livre da barra;
- funciona **totalmente offline**, sem conta, sem servidor e sem telemetria;
- consome o mínimo possível de memória e CPU, porque foi feito para ficar aberto durante o dia de trabalho;
- é distribuído como **software livre no GitHub**, com builds verificáveis e assinadas.

## 2. Público e uso esperado

- Pessoas que passam o dia no computador e querem um jogo de baixa intensidade, visível de relance, que
  não atrapalha o trabalho.
- Uso típico: o jogo fica em modo automático lutando sozinho; o usuário abre o painel de vez em quando para
  distribuir pontos, comprar equipamento e escolher o próximo torneio. Quem quiser pode jogar no modo manual.

## 3. Escopo da versão 1.0

### Incluído

| Área | Conteúdo |
|---|---|
| Plataforma | Faixa na barra de tarefas horizontal (inferior); modo flutuante como fallback; ícone na bandeja |
| Combate | Arena 1D, 8 ações, fôlego, críticos, esquiva, limite de turnos |
| Progressão | 6 atributos, níveis, ouro, loja com 12 itens, 1 liga completa com campeão |
| IA | 3 arquétipos de oponente; IA do jogador para o modo automático |
| Modos | Manual e automático |
| Persistência | Save local com backup, migração de versão e recuperação de corrupção |
| Idiomas | Português do Brasil e inglês |
| Acessibilidade | Escala extra, reduzir animações, alto contraste, HP distinguível sem depender de cor |
| Distribuição | ZIP portátil com exe assinado, SHA-256 e atestação de procedência |

### Fora da versão 1.0 (pós-1.0)

- Progresso offline (ganhos enquanto o jogo está fechado).
- Ligas adicionais, mais arquétipos, mais itens.
- Detecção automática do espaço livre da barra via UI Automation.
- Suporte nativo a barra vertical ou superior (na 1.0 elas usam o modo flutuante).
- Áudio além de efeitos simples.
- Linux e macOS (a arquitetura permite, mas não é objetivo).

## 4. Metas mensuráveis

Estas metas são critérios de aceite em todas as fases. Uma fase não termina se alguma delas regredir.

| Métrica | Meta | Como medir |
|---|---|---|
| Memória (Private Bytes) | < 15 MB em uso normal | Overlay de debug (`GetProcessMemoryInfo`), Process Explorer |
| CPU parado (sem animação) | 0,0 % | Gerenciador de Tarefas / Process Explorer por 10 min |
| CPU em combate animado | < 1 % de um núcleo | Process Explorer durante 10 min de modo automático |
| Tamanho do executável | < 3 MB | Tamanho do `.exe` da release |
| Handles GDI/USER | sem crescimento em 72 h | `GetGuiResources` registrado a cada minuto |
| Tempo de inicialização | < 300 ms até a faixa aparecer | Log com timestamps |
| Rede | nenhuma conexão | `cargo-deny` (bans) + Resource Monitor |
| Privilégios | `asInvoker` | Manifesto do executável |

## 5. Plataformas

- **Windows 11** 23H2, 24H2 e posteriores — alvo principal.
- **Windows 10** 22H2 — suportado. O Windows 10 saiu do suporte padrão da Microsoft em outubro de 2025,
  mas ainda tem base grande de usuários. O suporte é revisado a cada versão maior do jogo.
- Barra horizontal inferior: modo principal.
- Barra superior, esquerda ou direita (possível no Windows 10 nativamente e no Windows 11 com ferramentas
  como ExplorerPatcher): o jogo usa o modo janela flutuante.

## 6. Aspectos legais

- O projeto é inspirado no **gênero** de RPG de gladiador por turnos em navegador dos anos 2000.
  Mecânicas de jogo em abstrato (atributos, combate por distância, loja, torneios) não são protegidas por
  direito autoral.
- É **proibido** incluir no repositório, inclusive como placeholder temporário:
  - nomes, marcas, personagens, textos, falas ou nomes de itens de jogos existentes;
  - arte, sprites, sons, músicas ou fontes extraídos de jogos existentes;
  - código ou fórmulas obtidos por descompilação de jogos existentes.
- Todo asset de terceiros precisa de licença compatível (CC0, CC-BY, OFL) registrada em
  `assets-src/CREDITS.md` com autor, URL de origem e licença.
- Pull requests com material de origem duvidosa são recusados.

## 7. Princípios de engenharia

1. **Risco primeiro:** a integração com a barra de tarefas é o maior risco técnico; por isso é a Fase 1,
   antes da lógica do jogo.
2. **Separação estrita:** a lógica do jogo e a renderização não conhecem o Windows. Todo código `unsafe`
   fica em um único crate.
3. **Determinismo:** a lógica usa aritmética inteira e gerador aleatório próprio com estado salvo. A mesma
   semente gera sempre a mesma luta.
4. **Nada roda sem necessidade:** sem loop ativo; o processo dorme até chegar um evento.
5. **Falhar de forma segura:** se algo do shell não for encontrado ou mudar, o jogo cai para o modo
   flutuante em vez de travar ou sumir.
6. **Medir antes de afirmar:** toda meta de desempenho tem um método de medição documentado em
   [`09-verificacao-e-testes.md`](09-verificacao-e-testes.md).
