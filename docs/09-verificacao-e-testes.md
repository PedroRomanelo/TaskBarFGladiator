# 09 — Verificação e testes

## 1. Pirâmide de testes

| Nível | Onde roda | O que cobre | Ferramenta |
|---|---|---|---|
| Unitário | Linux + Windows (CI) | fórmulas, RNG, ponto fixo, validação de save, primitivas de desenho | `cargo test` |
| Propriedade | Linux (CI) | invariantes do combate e do save | `proptest` |
| Fuzzing | Linux (CI curto + agendado semanal) | parser de save | `cargo-fuzz` |
| Golden image | Linux (CI) | cenas renderizadas | testes de `render` com hash |
| Simulação | Linux (CI reduzido; completo local) | balanceamento | crate `sim` |
| Benchmark | Linux (CI com limite) | tempo de `step` e renderização | harness próprio ou `criterion` |
| Smoke (Windows) | `windows-latest` (CI) | executável inicia, cria janela, fecha corretamente | script PowerShell |
| Manual de plataforma | Máquinas/VMs reais | integração com barra de tarefas | matriz da seção 3 |
| Soak | Máquina dedicada | vazamentos e estabilidade | overlay CSV + script de perturbação |

## 2. Testes automáticos em detalhe

### 2.1 Invariantes (`proptest`)
1. Toda luta termina em ≤ 60 turnos, para quaisquer lutadores válidos e qualquer semente.
2. HP ∈ [0, HP máx]; fôlego ∈ [0, fôlego máx] após qualquer ação.
3. Posições ∈ [0, 9] e os lutadores nunca ocupam a mesma casa.
4. Determinismo: mesma semente + mesma sequência de entradas → mesmo `GameState` final (comparação byte a
   byte do JSON serializado).
5. Round-trip: `deserialize(serialize(state)) == state`.
6. `validate(qualquer_state_gerado)` nunca falha para estados produzidos pelo próprio jogo.
7. Ouro e XP nunca diminuem por overflow.

### 2.2 Save
- Fixtures: um save real de cada versão do esquema em `crates/game/tests/fixtures/save_vN.json`;
  teste carrega cada um e confirma a migração para a versão atual.
- Arquivo vazio, JSON truncado, JSON com tipos errados, arquivo de 50 MB, caracteres inválidos em UTF-8:
  todos resultam em erro tratado (nunca panic).
- Recuperação: `save.json` inválido + `save.bak` válido → carrega o backup.

### 2.3 Renderização
- Golden images: cenas fixas (luta no turno 0, golpe crítico, tela de vitória, layout compacto, alto
  contraste) em alturas de 24, 40, 48, 60 e 72 px.
- Alocação: 1.000 quadros com alocador contador → 0 alocações após o primeiro.
- Premultiplicação: pixel com alpha 0 sempre tem cor 0 após conversão.

### 2.4 Smoke test no Windows (CI)
```powershell
$p = Start-Process -FilePath target\release\<jogo>.exe -PassThru
Start-Sleep -Seconds 3
# Verifica que a janela existe pela classe (via pequeno utilitário de teste ou Add-Type com FindWindow)
# Envia WM_CLOSE e aguarda saída com código 0
```
- O runner do GitHub pode não ter uma barra de tarefas real (sessão de serviço); nesse caso o jogo deve
  entrar no modo flutuante, o que também valida o fallback.
- Verifica que o log não contém `ERROR` nem arquivo de crash.

### 2.5 Benchmarks com limite
| Medida | Limite no CI |
|---|---|
| `game::step` (ação de combate) | < 50 µs |
| Renderizar quadro da faixa 48 px | < 1 ms |
| Carregar save de 10 KB | < 5 ms |
| Tamanho do exe de release | < 3 MB |

Limites generosos para tolerar variação dos runners; o objetivo é detectar regressões grandes.

## 3. Matriz manual (executar antes de cada release; Fase 1 em diante)

### 3.1 Sistemas
- [ ] Windows 10 22H2
- [ ] Windows 11 23H2
- [ ] Windows 11 24H2 ou mais recente

### 3.2 Configurações de exibição
- [ ] DPI 100 %
- [ ] DPI 125 %
- [ ] DPI 150 %
- [ ] DPI 175 %
- [ ] Dois monitores com DPI diferente (barra em ambos)
- [ ] Monitor secundário à esquerda do principal (coordenadas negativas)
- [ ] Resolução 1366×768

### 3.3 Barra de tarefas
- [ ] Ícones centralizados (Windows 11)
- [ ] Ícones à esquerda (Windows 11)
- [ ] Ícones pequenos (Windows 10)
- [ ] Auto-hide ligado e desligado
- [ ] "Mostrar a barra em todas as telas" ligado e desligado
- [ ] Barra vertical (Windows 10) → modo flutuante
- [ ] Muitos programas abertos (barra cheia)

### 3.4 Interação com o shell
- [ ] Clicar na barra: faixa continua visível
- [ ] Menu Iniciar por cima da faixa
- [ ] Pesquisa por cima da faixa
- [ ] Central de notificações / calendário por cima da faixa
- [ ] Overflow da bandeja por cima da faixa
- [ ] Alt+Tab e Win+Tab
- [ ] Win+D
- [ ] Áreas de trabalho virtuais (trocar de área)
- [ ] Clicar na faixa não tira o foco do editor de texto aberto

### 3.5 Tela cheia
- [ ] Vídeo em tela cheia no navegador
- [ ] Jogo em tela cheia exclusiva
- [ ] Jogo em tela cheia sem borda
- [ ] Apresentação do PowerPoint
- [ ] Ao sair da tela cheia a faixa volta

### 3.6 Robustez
- [ ] Reiniciar o Explorer (`taskkill /f /im explorer.exe` e `start explorer.exe`)
- [ ] Bloquear e desbloquear (Win+L)
- [ ] Suspender e retomar
- [ ] Conectar por RDP e desconectar
- [ ] Desconectar o monitor onde está a faixa
- [ ] Trocar tema claro/escuro com o jogo aberto
- [ ] Ativar alto contraste com o jogo aberto
- [ ] Abrir o exe duas vezes (segunda instância mostra a primeira e sai)

### 3.7 Arquivos
- [ ] Usuário do Windows com acento no nome
- [ ] Corromper `save.json` manualmente → recupera do `.bak`
- [ ] Apagar `save.json` e `save.bak` → jogo novo
- [ ] Desligar o PC no meio de uma luta → luta retomada no mesmo ponto
- [ ] Matar o processo no meio de uma luta → mesma luta, mesmo resultado (semente)
- [ ] Mover o exe para outra pasta → ícone da bandeja aparece; "iniciar com o Windows" atualizado

### 3.8 Métricas (anotar valores)
- [ ] Private Bytes parado: ____ MB (meta < 15)
- [ ] Private Bytes em combate: ____ MB (meta < 15)
- [ ] CPU parado (10 min): ____ % (meta 0,0)
- [ ] CPU em combate (10 min): ____ % (meta < 1)
- [ ] GDI / USER / handles após 1 h: ____ / ____ / ____

## 4. Soak test
Descrito em [`05-fase-3-distribuicao.md`](05-fase-3-distribuicao.md), seção 1.1. Executado no fim da
Fase 1 (24 h, cena de teste) e antes de cada versão maior (72 h, jogo real).

## 5. Definição de pronto (para cada PR)
- [ ] CI verde.
- [ ] Testes novos para a mudança (ou justificativa).
- [ ] Documentação em `docs/` atualizada se o comportamento descrito mudou.
- [ ] Novo caso de borda descoberto adicionado em [`06-casos-de-borda.md`](06-casos-de-borda.md).
- [ ] Sem novo `unsafe` fora de `platform`; `unsafe` novo com `// SAFETY:`.
- [ ] Sem nova dependência não justificada.
