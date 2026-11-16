# Changelog

Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/). Versionamento semântico.
A seção **Balanceamento** registra toda mudança de regras ou conteúdo (ver
`docs/12-balanceamento-de-referencia.md`).

## [Não lançado]

### Adicionado
- Documentação de planejamento em `docs/`.
- Workspace Rust (`game`, `render`, `assets`, `platform`, `sim`, executável), CI, `cargo-deny`, licenças e
  arquivos de governança (Fase 0).
- Crate `game`: conteúdo em JSON validado na carga, combate por turnos determinístico (PCG32, aritmética
  inteira), IA por utilidade com 4 arquétipos, progressão, loja, ligas e save com leitura defensiva,
  migração de esquema e reinício de luta quando as regras mudam.
- Crate `sim`: matriz de lutas, carreiras completas, conferência de metas, baseline e comparação.
- Testes: unitários, de propriedade (`proptest`), replays gravados; job de CI `balance`.
- `docs/14-formato-do-conteudo.md`.
- Crate `assets`: arte original (lutador em 4 quadros, 11 ícones, fonte 3×6 com acentos) definida como
  mapas de caracteres, sem decodificador de imagem.
- Crate `render`: superfície BGRA premultiplicada, texto com contorno, faixa (luta manual/automática,
  fora de luta, resultado) e painel com regiões clicáveis; temas claro, escuro e alto contraste;
  codificador PNG mínimo; golden images e teste de zero alocação por quadro.
- Crate `platform`: faixa em janela em camadas sobre a barra de tarefas, modo flutuante, hooks de
  WinEvent fora de contexto, tela cheia, sessão, energia, bandeja, instância única, pastas conhecidas,
  tema, alto contraste, chave Run, estatísticas de memória; cálculo de posição portável e testado.
- Executável: controlador (luta manual e automática, painel com ficha/loja/liga/configurações),
  animação por passos, textos pt-BR/en, save atômico com backup em thread separada, configuração,
  log com rotação, relatório de crash, manifesto (asInvoker, PerMonitorV2, UTF-8, SegmentHeap).
- CI: teste de fumaça do executável no Windows.
- Workflow de release (tag `v*.*.*`): testes, metas, baseline, build, ZIP portátil, `SHA256SUMS.txt`,
  atestação de procedência e rascunho de release. `docs/LEIA-ME-USUARIO.txt` incluído no ZIP.

### Alterado
- O jogo saiu de dentro da barra de tarefas: agora é um palco sem fundo apoiado no topo da barra, com
  lutadores de cerca de duas vezes a altura dela. A escala acompanha a altura da barra e pode ser
  ajustada em CONFIG (pequeno, normal, grande). Com ocultação automática, o palco desce junto com a
  barra em vez de sumir. Posição padrão: canto direito.
- Arte refeita: lutador em camadas de 32 × 32 (corpo por pose + aparência + equipamento) no lugar dos
  4 quadros fixos de 16 × 20. Novos ícones dos golpes.
- Fluxo mais claro: legenda da etapa no topo (preparar, sua vez, rodada em andamento, resultado),
  rodada narrada passo a passo com dano e fôlego flutuando sobre os lutadores, botões de ação em
  volta do jogador com explicação ao passar o mouse, e o botão principal (LUTAR / CONTINUAR) sempre
  no mesmo canto da faixa de status.

### Adicionado
- Aparência do gladiador (pele, cabelo, cor do cabelo, olhos, barba) na nova aba VISUAL do painel,
  gravada no save. Oponentes têm aparência própria, derivada do nome.
- Equipamento visível e fixo: 6 armas (faca, espada curta, lança, machado, arco, bastão; as duas
  últimas ainda sem item no jogo), 4 escudos e 4 armaduras com elmo, peitoral e grevas.
- Folhas de sprites no `cargo run -p render --example preview`.
- `docs/15-plano-fluxo-de-jogo.md`: plano do fluxo de jogo (título, criação do personagem, sete
  atributos com Carisma, intervalo entre lutas e economia), ainda não implementado.

### Balanceamento
- Sem mudança de regras. A aparência só é gravada no save quando difere da padrão, então o estado
  serializado dos replays continua idêntico.
- Baseline inicial criado (`balance/`). Todas as 22 metas de `docs/12` atendidas. Metas P3, P4 e P6
  revisadas por serem incompatíveis com as demais (justificativa em `docs/12`, seção 3.3).
