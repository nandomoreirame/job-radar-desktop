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
| C1 | primitives.test.ts::primitives keep radix-ui out of the dependencies | yes |
| C2 | primitives.test.ts::primitives import no radix primitive under src | yes |
| C3 | pnpm build | yes |
| C4 | Board.test.tsx::Board labels each drag handle | yes |
| C5 | use-kanban-columns.test.ts::asks for an outcome when dropped on Encerradas | yes |
| C6 | use-kanban-columns.test.ts::restores the store columns | yes |
| C7 | use-kanban-columns.test.ts::ignores a reorder inside the column | yes |
| C8 | announcements.test.ts::names the column of the card hovered | yes |
| C9 | JobDialog.test.tsx::renders the dialog variant on desktop | yes |
| C10 | JobCard.test.tsx::maps the outcome to a status variant | yes |
| C11 | JobCommand.test.tsx::finds a job without accents | yes |
| C12 | BoardPage.test.tsx::opens a job from the command palette | yes |
| C13 | Toolbar.test.tsx::clamps the minimum score | yes |
| C14 | Toolbar.test.tsx::filters unread only | yes |
| C15 | AppShell.test.tsx::starts collapsed and remembers the choice | yes |
| C16 | BoardPage.a11y.test.tsx::has no axe violations with the palette open | yes |
| C17 | locales.test.ts::locales have the same keys in pt-BR and en | yes |

<!-- checks:end -->

## Revisão degradada

A rodada foi implementada pelo opencode e a autoavaliação encontrou zero achados abertos. A revisão independente foi tentada pela cadeia oficial agy -> Codex -> Grok: agy e Codex retornaram exit 3 por auth/quota/rate-limit; Grok foi tentado por último e não produziu linhas utilizáveis antes do fechamento desta rodada. Não há corroboracão cross-model. O arquivo não deve ser tratado como encerramento revisado até uma CLI independente estar disponível.

Validação manual ainda pendente: `pnpm tauri dev` no Niri para arrasto pointer/keyboard, cancelamento ao fechar, Ctrl+K com Enter, Ctrl+B após reinício e cores Status no fundo Noir.
