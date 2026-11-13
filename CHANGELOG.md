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

### Balanceamento
- Baseline inicial criado (`balance/`). Todas as 22 metas de `docs/12` atendidas. Metas P3, P4 e P6
  revisadas por serem incompatíveis com as demais (justificativa em `docs/12`, seção 3.3).
