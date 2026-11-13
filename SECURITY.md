# Política de segurança

## Como reportar uma vulnerabilidade

Não abra uma issue pública. Use o recurso **Report a vulnerability** na aba **Security** do repositório
(GitHub Private Vulnerability Reporting).

Inclua:
- versão do jogo e do Windows;
- descrição do problema e impacto;
- passos para reproduzir ou arquivo de prova de conceito (por exemplo, um save malformado).

## Prazos

- Confirmação de recebimento: até 7 dias.
- Avaliação inicial: até 14 dias.
- Correção: publicada em uma nova release, com nota no `CHANGELOG.md` e, quando aplicável, um GitHub
  Security Advisory.

## Escopo

O modelo de ameaças está em [`docs/07-seguranca.md`](docs/07-seguranca.md). Pontos principais:
- o jogo não usa rede e roda sem privilégios de administrador;
- a única entrada não confiável são os arquivos de save e configuração;
- código `unsafe` existe apenas em `crates/platform`.

Fora do escopo: alterar o próprio save para trapacear no jogo local.
