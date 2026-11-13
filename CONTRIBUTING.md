# Como contribuir

Leia antes [`docs/README.md`](docs/README.md), principalmente a arquitetura (`docs/01`) e a verificação
(`docs/09`).

## Ambiente

- Rust: a versão fixada em `rust-toolchain.toml` é instalada automaticamente pelo `rustup`.
- Os crates `game`, `render`, `assets` e `sim` compilam e são testados em qualquer sistema.
- `platform` e o executável final só têm funcionalidade no Windows. No Linux/macOS é possível verificar a
  compilação com:
  ```
  rustup target add x86_64-pc-windows-msvc
  cargo check --workspace --target x86_64-pc-windows-msvc
  ```

## Antes de abrir um PR

```
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Regras de código

1. **`unsafe` só em `crates/platform`.** Todo bloco `unsafe` precisa de um comentário `// SAFETY:` que
   explique por que as pré-condições estão satisfeitas. Os demais crates têm `#![forbid(unsafe_code)]`.
2. **Nada de `unwrap()`/`expect()`** em caminhos que dependem de dados externos (save, config, sistema).
3. **Regras e conteúdo do jogo ficam nos arquivos de dados** (`crates/game/content/`). Nenhum número de
   balanceamento no meio do código. Se o resultado do simulador mudar, siga o processo de
   [`docs/12-balanceamento-de-referencia.md`](docs/12-balanceamento-de-referencia.md).
4. **Dependências novas** precisam de justificativa no PR e passar no `cargo deny check`. Crates de rede
   são proibidos.
5. **Assets** (arte, som, fontes) precisam ser originais ou ter licença CC0, CC-BY ou OFL, registrados em
   `assets-src/CREDITS.md`.
6. Documentação em `docs/` atualizada no mesmo PR quando o comportamento descrito mudar. Casos de borda
   novos entram em `docs/06-casos-de-borda.md`.

## Definição de pronto

Ver [`docs/09-verificacao-e-testes.md`](docs/09-verificacao-e-testes.md), seção 5.

## Licença das contribuições

Ao contribuir, você concorda que o código é licenciado sob `MIT OR Apache-2.0` e a arte sob `CC-BY-4.0`,
como o restante do projeto.
