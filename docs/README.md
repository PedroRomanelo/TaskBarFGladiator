# Documentação do projeto

Esta pasta contém o planejamento completo do jogo, do início ao lançamento da versão 1.0.

## Ordem de leitura recomendada

1. [`00-visao-geral.md`](00-visao-geral.md) — o que será construído, para quem, com quais limites.
2. [`01-arquitetura.md`](01-arquitetura.md) — como o código será organizado e por quê.
3. [`11-design-do-jogo.md`](11-design-do-jogo.md) — regras do jogo (atributos, ações, fórmulas).
4. Fases, em ordem cronológica:
   - [`02-fase-0-preparacao.md`](02-fase-0-preparacao.md)
   - [`03-fase-1-plataforma.md`](03-fase-1-plataforma.md)
   - [`04-fase-2-jogo.md`](04-fase-2-jogo.md)
   - [`05-fase-3-distribuicao.md`](05-fase-3-distribuicao.md)
5. Documentos transversais (consultados durante todas as fases):
   - [`06-casos-de-borda.md`](06-casos-de-borda.md)
   - [`07-seguranca.md`](07-seguranca.md)
   - [`08-memoria-e-desempenho.md`](08-memoria-e-desempenho.md)
   - [`09-verificacao-e-testes.md`](09-verificacao-e-testes.md)
6. [`10-cronograma-e-decisoes.md`](10-cronograma-e-decisoes.md) — prazos e registro de decisões.

## Convenções destes documentos

- **Tarefa**: item de trabalho com resultado verificável.
- **Preocupação**: risco conhecido da fase, com a medida planejada.
- **Caso de borda**: situação incomum que precisa de tratamento explícito no código.
- **Critério de aceite**: condição objetiva para considerar a fase concluída.
- Termos de API do Windows aparecem com o nome exato da função, mensagem ou constante (ex.:
  `SHAppBarMessage`, `WM_DPICHANGED`), para facilitar a busca na documentação da Microsoft.
- "px" sem qualificação significa pixel físico; "px lógico" significa pixel antes da escala de DPI.

## Manutenção

- Toda decisão que mude algo descrito aqui deve atualizar o documento correspondente no mesmo commit/PR.
- Decisões novas entram no registro em [`10-cronograma-e-decisoes.md`](10-cronograma-e-decisoes.md) com
  data, contexto, alternativa escolhida e motivo.
