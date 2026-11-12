# 05 — Fase 3 (Fim): polimento, robustez e distribuição

**Duração estimada:** 3 a 5 semanas (≈ 30–50 h).
**Objetivo:** versão 1.0 estável para uso diário, distribuída de forma verificável e com o mínimo de atrito
com antivírus e SmartScreen.

---

## 1. Tarefas

### 1.1 Teste de longa duração (soak test)
- Duração: 72 horas contínuas.
- Configuração: modo automático lutando sem parar, overlay de debug gravando CSV a cada minuto
  (Private Bytes, Working Set, handles, GDI, USER, tempo médio de quadro).
- Script PowerShell de perturbação executado em paralelo, a intervalos aleatórios de 10–60 min:
  - reiniciar o Explorer;
  - alternar tema claro/escuro (alterando o registro e enviando `WM_SETTINGCHANGE`);
  - alternar auto-hide da barra;
  - bloquear a sessão (`rundll32 user32.dll,LockWorkStation`) — exige desbloqueio manual ou máquina de teste dedicada;
  - abrir e fechar um vídeo em tela cheia.
- Critério: nenhuma métrica com tendência de crescimento (regressão linear com inclinação ≈ 0) e nenhum
  crash nos logs.

### 1.2 Áudio (opcional, desligado por padrão)
- `PlaySoundW(ptr, NULL, SND_MEMORY | SND_ASYNC | SND_NODEFAULT)` com WAV curtos embutidos.
- Motivo de não usar uma biblioteca de áudio completa: elas criam threads de mixagem e buffers permanentes
  (vários MB) e mantêm o dispositivo de áudio aberto, o que pode impedir o sistema de economizar energia.
- Desligado por padrão porque o uso principal é em ambiente de trabalho.

### 1.3 Iniciar com o Windows (opcional, desligado por padrão)
- Ativar: criar valor em `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` com o caminho do exe entre aspas.
- Respeitar `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run`: se o usuário
  desativou pelo Gerenciador de Tarefas, a opção aparece como desativada no jogo, e o jogo não sobrescreve
  essa escolha.
- Se o exe for movido, o valor fica apontando para o caminho antigo: ao iniciar, se a opção estiver ativa
  e o caminho for diferente do atual, atualizar.

### 1.4 Acessibilidade
- Escala extra (+1 no fator inteiro).
- Reduzir animações (troca deslocamentos interpolados por saltos; respeita `SPI_GETCLIENTAREAANIMATION`
  como padrão inicial).
- Alto contraste automático (Fase 1) e manual.
- Painel navegável por teclado (Tab, setas, Enter, Esc).

### 1.5 Pipeline de release (`.github/workflows/release.yml`)
Disparado por tag `v*.*.*`:
1. Checkout com SHA fixo; toolchain da `rust-toolchain.toml`.
2. `cargo build --release --locked --target x86_64-pc-windows-msvc`.
3. Testes completos e `sim` completo.
4. Assinatura de código do `.exe` (ver 1.6).
5. Empacotar `<Jogo>-vX.Y.Z-windows-x64.zip` com: exe, `LICENSE-*`, `README.txt`, `CREDITS.md`.
6. Gerar `SHA256SUMS.txt`.
7. `actions/attest-build-provenance` sobre o zip e o exe (atestação verificável com
   `gh attestation verify`).
8. Criar a release no GitHub como rascunho; publicação manual após teste em máquina limpa.

Permissões: `contents: write`, `id-token: write`, `attestations: write` **somente** neste job.

### 1.6 Assinatura de código
| Opção | Custo | Observação |
|---|---|---|
| SignPath Foundation | gratuito para projetos open source aprovados | exige processo de solicitação e build no CI |
| Azure Trusted Signing (Artifact Signing) | assinatura mensal paga | elegibilidade para pessoa física depende do país; verificar antes |
| Certificado OV/EV comercial | anual, mais caro | exige token de hardware ou HSM em nuvem |
| Sem assinatura | — | SmartScreen exibe aviso "Windows protegeu o computador"; maior chance de falso positivo |

Recomendação: solicitar SignPath Foundation na Fase 0/1 (o processo leva tempo); publicar sem assinatura
até a aprovação, documentando no README como verificar o SHA-256 e a atestação.

### 1.7 Distribuição
- ZIP portátil (principal): extrair e executar.
- `winget` (opcional): manifesto enviado ao repositório `microsoft/winget-pkgs` apontando para o zip da release.
- **Sem atualizador automático:** o jogo não usa rede. O painel "Sobre" mostra a versão e o link do
  repositório como texto.
- Sem instalador na 1.0 (um instalador adiciona superfície de ataque, desinstalador e entradas de registro).

### 1.8 Documentação de usuário
- README com GIF da faixa na barra, requisitos, instalação, onde fica o save, como fazer backup.
- Desinstalação: fechar o jogo, apagar o exe, apagar `%APPDATA%\<Jogo>` e `%LOCALAPPDATA%\<Jogo>`, remover
  a opção "iniciar com o Windows" antes (ou apagar o valor em `HKCU\...\Run`).
- FAQ: aviso do SmartScreen, antivírus, a faixa sumiu após atualização do Windows, a faixa ficou por cima do
  Menu Iniciar, como mudar de posição, como usar em barra vertical.

---

## 2. Preocupações da fase

| Preocupação | Detalhe | Medida |
|---|---|---|
| Falsos positivos de antivírus | Executáveis novos, sem assinatura, que usam `SetWinEventHook` e janelas topmost podem ser sinalizados por heurística | Não usar UPX nem empacotadores; não usar hooks globais com DLL (`SetWindowsHookEx`); recurso de versão completo; assinatura; enviar amostra ao portal de análise da Microsoft quando houver detecção |
| SmartScreen | Mesmo assinado, um executável sem reputação pode gerar aviso até acumular downloads | Documentar no README; reputação cresce com o tempo e com o mesmo certificado |
| Atualizações do Windows | Mudanças em classes de janela do shell podem quebrar a detecção de flyouts ou da barra | Template de issue específico; lista de classes em um único arquivo; fallback flutuante |
| Contribuições externas | Código `unsafe` novo pode introduzir falhas de memória | Revisão obrigatória em PRs que tocam `platform`; CI obrigatório; CODEOWNERS para `crates/platform/` |
| Regressão de desempenho | Novas features aumentam memória/CPU aos poucos | Benchmarks no CI (tempo de `step` e de renderização) com limite; soak antes de cada versão maior |

## 3. Critérios de aceite (versão 1.0)

- [ ] Soak test de 72 h sem crescimento de métricas e sem crash.
- [ ] Matriz manual completa aprovada em Windows 10 22H2, Windows 11 23H2 e Windows 11 24H2+.
- [ ] Release gerada pelo CI, com `SHA256SUMS.txt` e atestação verificável.
- [ ] Executável assinado (ou, enquanto não houver aprovação, README explicando a verificação por hash).
- [ ] Teste em máquina virtual limpa: baixar, verificar hash, extrair, executar, jogar 1 luta, fechar,
      reabrir e confirmar o save.
- [ ] Documentação de usuário completa.
