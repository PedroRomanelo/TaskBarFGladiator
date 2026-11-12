# 07 — Segurança

## 1. Modelo de ameaças

### 1.1 O que o jogo é, do ponto de vista de segurança
- Processo de usuário comum (`asInvoker`), sem rede, sem serviço, sem driver, sem componentes injetados em
  outros processos.
- Única entrada de dados não confiável: arquivos em `%APPDATA%\<Jogo>\` (save e config), que o usuário ou
  outro programa pode editar.
- Distribuído como executável baixado do GitHub.

### 1.2 Ameaças consideradas

| # | Ameaça | Impacto | Medida |
|---|---|---|---|
| T1 | Save/config malicioso causando falha de memória | Execução de código | `game` sem `unsafe`; `serde_json` (Rust seguro); limite de tamanho; fuzzing |
| T2 | Save malicioso causando consumo excessivo | Travamento | Limite de 1 MB; validação de tamanhos de coleções |
| T3 | Falha de memória no código `unsafe` de `platform` | Crash ou execução de código | `unsafe` restrito, documentado e revisado; RAII; testes |
| T4 | Sequestro de DLL (DLL maliciosa ao lado do exe) | Execução de código ao abrir o jogo | `SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32)`; nenhuma DLL própria |
| T5 | Dependência comprometida (supply chain) | Código malicioso no binário | Poucas dependências; `Cargo.lock`; `cargo-deny`; RustSec; revisão de atualizações |
| T6 | Action do GitHub comprometida | Binário de release adulterado | Actions fixadas por SHA; permissões mínimas |
| T7 | Binário adulterado em espelho/terceiro | Usuário executa malware com o nome do jogo | SHA-256 publicado; atestação de procedência; assinatura de código |
| T8 | Elevação de privilégio | — | Não aplicável: o jogo nunca roda elevado e não tem componente privilegiado |
| T9 | Vazamento de dados pessoais | Privacidade | Sem rede; logs sem nome de usuário ou caminhos com o nome do perfil |
| T10 | Overflow aritmético levando a estado inválido | Bugs explorados no jogo | `overflow-checks`; tipos saturantes; validação |

### 1.3 Fora do escopo
- Trapaça no jogo pelo próprio usuário (editar o save para ter mais ouro). É um jogo offline e local;
  a validação existe para estabilidade, não para impedir o usuário de alterar o próprio jogo.
- Malware já em execução com os privilégios do usuário (pode fazer qualquer coisa que o jogo faz).

## 2. Regras de código

### 2.1 `unsafe`
- Permitido **apenas** no crate `platform`. Os demais têm `#![forbid(unsafe_code)]`.
- Todo bloco `unsafe` tem comentário `// SAFETY:` explicando as pré-condições (lint
  `clippy::undocumented_unsafe_blocks = "deny"`).
- Funções `unsafe fn` internas têm seção `# Safety` na documentação.
- Ponteiros brutos não saem do módulo que os cria; a API pública de `platform` é segura.
- PRs que adicionam ou alteram `unsafe` exigem revisão do mantenedor (CODEOWNERS).

### 2.2 Tratamento de erros
- Nenhum `unwrap()`/`expect()` em caminho de produção que dependa de dados externos (lint `unwrap_used`
  como aviso; exceções justificadas com comentário).
- Falhas de API Win32 viram `Result` com o código de `GetLastError`, registradas em log.
- Falha não crítica (ex.: ícone da bandeja não criado) degrada a funcionalidade, não encerra o jogo.

### 2.3 Entrada não confiável (save e config)
1. Tamanho máximo: 1 MB (save), 64 KB (config).
2. Desserialização com `serde_json` para tipos fortemente tipados.
3. Validação de domínio com limites explícitos.
4. Coleções com tamanho máximo (ex.: inventário ≤ 64 itens) verificadas após desserializar.
5. Fuzzing contínuo (`cargo-fuzz`), com corpus inicial de saves válidos de cada versão.

### 2.4 Inicialização endurecida
1. `SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32)` como primeira instrução de `main`.
   Observação: as DLLs importadas estaticamente são carregadas pelo loader antes do `main`; as do sistema
   protegidas pela lista *KnownDLLs* não são afetadas por DLLs na pasta do exe. Por isso o projeto
   minimiza imports de DLLs fora dessa lista e carrega as demais sob demanda após a chamada.
2. `SetProcessMitigationPolicy` (avaliar na implementação, com teste de compatibilidade):
   - `ProcessExtensionPointDisablePolicy` (bloqueia DLLs legadas de extensão como AppInit);
   - `ProcessImageLoadPolicy` com `PreferSystem32Images` e `NoRemoteImages`.
3. Flags de link padrão do MSVC já ativas no Rust: ASLR (`/DYNAMICBASE`), DEP (`/NXCOMPAT`),
   `/HIGHENTROPYVA`. Avaliar `/CETCOMPAT` (proteção de pilha por hardware) via `-C link-arg=/CETCOMPAT`.
   Control Flow Guard (`-C control-flow-guard`) também pode ser avaliado (custo de desempenho baixo).

### 2.5 O que o jogo nunca faz
- Abrir conexões de rede.
- Executar outros programas ou comandos.
- Injetar DLL ou usar `SetWindowsHookEx` com hook global.
- Ler arquivos fora das próprias pastas em `%APPDATA%` e `%LOCALAPPDATA%`.
- Gravar no registro fora de `HKCU\...\Run` (somente se o usuário ativar "iniciar com o Windows").
- Coletar telemetria.

## 3. Cadeia de suprimentos

| Medida | Detalhe |
|---|---|
| Dependências mínimas | Lista fechada em [`01-arquitetura.md`](01-arquitetura.md); nova dependência exige justificativa no PR |
| `Cargo.lock` commitado | Builds usam `--locked` |
| `cargo-deny` | Licenças permitidas, RustSec, bans de crates de rede, apenas crates.io |
| Dependabot | Semanal; PRs revisados manualmente (ler changelog e diff das dependências diretas) |
| `cargo-vet` (opcional) | Registro de auditorias de dependências |
| GitHub Actions | Fixadas por SHA; `permissions` mínimas; sem `pull_request_target` |
| Proteção de branch | `main` exige CI verde e revisão; sem push forçado |

## 4. Integridade da distribuição

1. Build somente no GitHub Actions a partir de uma tag.
2. `SHA256SUMS.txt` publicado junto da release.
3. Atestação de procedência (`actions/attest-build-provenance`); verificação pelo usuário:
   ```
   gh attestation verify <arquivo>.zip --repo <dono>/<repositório>
   ```
4. Assinatura Authenticode do executável (ver [`05-fase-3-distribuicao.md`](05-fase-3-distribuicao.md)).
5. Sem UPX ou empacotadores: além de aumentarem falsos positivos, impedem a verificação do binário.

## 5. Divulgação de vulnerabilidades

- `SECURITY.md` aponta para o recurso de *Private vulnerability reporting* do GitHub.
- Prazo de resposta declarado (ex.: 7 dias para confirmar recebimento).
- Correção publicada com nota na release e, quando aplicável, GitHub Security Advisory.
