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

### Balanceamento
- Baseline inicial criado (`balance/`). Todas as 22 metas de `docs/12` atendidas. Metas P3, P4 e P6
  revisadas por serem incompatíveis com as demais (justificativa em `docs/12`, seção 3.3).
