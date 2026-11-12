# 02 — Fase 0: Preparação

**Duração estimada:** 1 semana (≈ 10 h).
**Objetivo:** repositório pronto para receber código, com CI, licenças, regras de qualidade e decisões
básicas tomadas, de forma que nenhuma delas precise ser refeita depois.

## 1. Tarefas

### 1.1 Nome do jogo
1. Listar 5 a 10 candidatos de nome próprio.
2. Para cada um, pesquisar conflitos em: GitHub, Steam, itch.io, lojas de aplicativos, INPI (Brasil) e
   USPTO (EUA), além de buscas na web.
3. Escolher um nome sem conflito e registrar em [`10-cronograma-e-decisoes.md`](10-cronograma-e-decisoes.md).
4. Derivar do nome: nome do executável, nome da pasta em `%APPDATA%`, prefixo das classes de janela, nome do
   mutex de instância única, identificador nas strings de log.

**Por que agora:** o nome aparece no caminho do save, no mutex, no exe, nos crates e na documentação.
Mudar depois exige migrar o save dos usuários de uma pasta para outra.

### 1.2 Licenças
- Código: `MIT OR Apache-2.0` (padrão do ecossistema Rust; arquivos `LICENSE-MIT` e `LICENSE-APACHE`).
- Arte própria: `CC-BY-4.0` ou `CC0-1.0` (arquivo `assets-src/LICENSE`).
- Assets de terceiros: só CC0, CC-BY ou OFL (fontes), registrados em `assets-src/CREDITS.md`
  com: arquivo, autor, URL de origem, licença, modificações feitas.

### 1.3 Workspace
1. `Cargo.toml` do workspace com membros `crates/*`, `app`.
2. `[workspace.package]`: `edition = "2021"` (ou `2024`, conforme a versão de Rust fixada), `license`,
   `repository`, `rust-version`.
3. `[workspace.lints]`:
   ```toml
   [workspace.lints.rust]
   unsafe_op_in_unsafe_fn = "deny"
   missing_debug_implementations = "warn"

   [workspace.lints.clippy]
   undocumented_unsafe_blocks = "deny"
   unwrap_used = "warn"
   expect_used = "warn"
   indexing_slicing = "warn"       # em game: preferir get() para não haver panic por índice
   arithmetic_side_effects = "warn" # em game: força saturating/checked em tipos de domínio
   ```
4. `#![forbid(unsafe_code)]` em `game`, `render`, `assets`, `sim`.
5. `rust-toolchain.toml` com versão exata (ex.: `channel = "1.xx.0"`, `components = ["rustfmt", "clippy"]`,
   `targets = ["x86_64-pc-windows-msvc"]`).
6. Crates vazios compilando (`lib.rs` com um teste trivial) para validar o CI.

### 1.4 Integração contínua (GitHub Actions)

`.github/workflows/ci.yml`:

| Job | Runner | Passos |
|---|---|---|
| `lint` | `ubuntu-latest` | `cargo fmt --check`; `cargo clippy -p game -p render -p assets -p sim -- -D warnings` |
| `test-portable` | `ubuntu-latest` | `cargo test -p game -p render -p assets -p sim` |
| `windows` | `windows-latest` | `cargo clippy --workspace -- -D warnings`; `cargo test --workspace`; `cargo build --release --locked` |
| `deny` | `ubuntu-latest` | `cargo deny check` |

Regras de segurança do CI:
- Toda action referenciada por **SHA de commit completo**, não por tag (tags podem ser movidas).
- `permissions: contents: read` no topo do workflow; permissões maiores só no job de release.
- Sem `pull_request_target` (executa código de PR com permissões do repositório).
- Cache do Cargo com chave baseada no hash do `Cargo.lock`.

`.github/dependabot.yml`: atualizações semanais para `cargo` e `github-actions`.

### 1.5 `deny.toml`
- `[licenses]`: permitir MIT, Apache-2.0, BSD-2/3-Clause, ISC, Zlib, Unicode-3.0; negar o resto.
- `[advisories]`: falhar com vulnerabilidades do banco RustSec; avisar sobre crates sem manutenção.
- `[bans]`: negar crates de rede (`reqwest`, `hyper`, `ureq`, `curl`, `native-tls`, `openssl`);
  avisar sobre versões duplicadas.
- `[sources]`: permitir apenas `crates.io`; negar dependências por git.

### 1.6 Arquivos do repositório
- `README.md` (existe), `SECURITY.md` (como reportar vulnerabilidade de forma privada — GitHub Security
  Advisories), `CONTRIBUTING.md` (padrões de código, regra de assets, DCO opcional), `CODE_OF_CONDUCT.md`.
- `.gitignore`: `target/`, `*.pdb` fora de release, arquivos de editor.
- `.gitattributes`: `* text=auto eol=lf`; `*.png binary`; `*.ase binary`.
- Templates de issue: bug (pede versão do Windows via `winver`, DPI, número de monitores, posição da barra,
  log anexado), sugestão.

### 1.7 Documento de design
- Revisar e aprovar [`11-design-do-jogo.md`](11-design-do-jogo.md) (atributos, ações, fórmulas iniciais).
  Ele é a especificação do crate `game` na Fase 2.

## 2. Preocupações

| Preocupação | Medida |
|---|---|
| Nome escolhido tarde | Decidir nesta fase; tudo que deriva do nome fica em um único módulo de constantes |
| Lints rígidos demais travando o desenvolvimento | `warn` em vez de `deny` nos lints de estilo; `deny` só nos de segurança |
| Versão do Rust mudando entre máquinas | `rust-toolchain.toml` fixa a versão |
| Material de terceiros entrando como placeholder | Regra explícita em `CONTRIBUTING.md`; usar retângulos coloridos como placeholder |
| CI lento no Windows | Jobs portáveis no Linux; Windows só para `platform`/`app` |

## 3. Critérios de aceite

- [ ] Nome definitivo escolhido e registrado.
- [ ] Workspace com todos os crates vazios compilando.
- [ ] CI verde nos quatro jobs.
- [ ] `cargo deny check` passando.
- [ ] Licenças, `SECURITY.md`, `CONTRIBUTING.md` e templates de issue publicados.
- [ ] GDD revisado.
