# Revisão — board-ui-migration

- **Rodada:** 2026-11-02
- **Achados abertos:** 0
- **Estado:** ciclo fechado
- **Implementado por:** opencode

> Arquivo reescrito por inteiro a cada `/rpi-review`. Um achado só desaparece
> daqui por ter deixado de existir no código: quem corrige não marca resolvido.

<!-- findings:begin -->

| Severidade | Arquivo:linha | Problema | Correção sugerida |
|------------|---------------|----------|-------------------|
| — | — | nenhum achado nesta rodada | — |

<!-- findings:end -->

## Checks

<!-- checks:begin -->

| Check | Test | Holds |
|-------|------|-------|
| C1 | primitives keep radix-ui out of the dependencies | yes |
| C2 | primitives import no radix primitive under src | yes |
| C3 | pnpm build | yes |
| C4 | Board labels each drag handle | yes |
| C5 | useKanbanColumns asks for an outcome when dropped on Encerradas | yes |
| C6 | useKanbanColumns restores the store columns | yes |
| C7 | useKanbanColumns ignores a reorder inside the column | yes |
| C8 | buildAnnouncements names the column of the card hovered | yes |
| C9 | JobDialog renders the dialog variant on desktop | yes |
| C10 | JobCard maps the outcome to a status variant | yes |
| C11 | JobCommand finds a job without accents | yes |
| C12 | BoardPage opens a job from the command palette | yes |
| C13 | Toolbar clamps the minimum score | yes |
| C14 | Toolbar filters unread only | yes |
| C15 | AppShell starts collapsed and remembers the choice | yes |
| C16 | BoardPage has no axe violations with the palette open | yes |
| C17 | locales have the same keys in pt-BR and en | yes |

<!-- checks:end -->

## Revisão da correção

Os 15 achados da rodada anterior foram tratados e revalidados localmente. A nova cadeia independente foi tentada novamente: agy e Codex retornaram exit 3 por auth/quota/rate-limit; Grok permanece indisponível nesta sessão. Portanto, esta rodada continua sem corroboração cross-model e a revisão externa segue pendente.

Validação manual ainda pendente no Niri: arrasto pointer/keyboard, cancelamento ao encerrar, Ctrl+K com Enter, Ctrl+B após reinício e cores do Status no fundo Noir.
