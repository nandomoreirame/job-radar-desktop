# Revisão — board-ui-migration

- **Rodada:** 2026-11-02
- **Achados abertos:** 6
- **Estado:** aguardando correção
- **Implementado por:** opencode

> Arquivo reescrito por inteiro a cada `/rpi-review`. Um achado só desaparece
> daqui por ter deixado de existir no código: quem corrige não marca resolvido.

<!-- findings:begin -->

| Severidade | Arquivo:linha | Problema | Correção sugerida |
|------------|---------------|----------|-------------------|
| High | src/widgets/board/model/use-kanban-columns.ts:25 | REQ-003: a guarda sameColumns compara só id e etapa, então qualquer mudança de dados de um card que fica na mesma coluna não chega ao board nem ao cardsRef. Exemplo que quebrou: um card em Encerradas como 'hired' reencerrado como 'rejected' pelo menu continua mostrando Contratado (rerender com outcome 'rejected' manteve 'hired'); o restore() também volta para esses dados velhos | Ressincronizar sempre que a identidade de cards mudar e não houver arrasto em andamento (remover sameColumns), mantendo cardsRef atualizado; adicionar o teste do reencerramento com outro motivo |
| Medium | src/features/search-jobs/ui/JobCommand.tsx:36 | REQ-007 / caso de borda da spec: o value do item agora é só título e empresa, então duas vagas com o mesmo título na mesma empresa colidem no cmdk (mesmo value), e uma delas fica inalcançável ou destacada junto | Usar value={card.id} e keywords={[card.title, card.company]}, com o filter (value, search, keywords) casando só keywords.join(' '); testar duas vagas iguais listadas e selecionáveis, e que o id não é buscável |
| Medium | src/widgets/board/BoardColumn.tsx:26 | Pedido do usuário: as colunas ainda ocupam a altura toda. O KanbanColumn gerado tem 'size-full' na base (src/shared/ui/kanban.tsx:727) e o tailwind-merge mantém size-full junto com max-h-full, então a altura continua 100% | Trocar 'size-full' por 'w-full' na base do KanbanColumn gerado (registrar a edição no topo do arquivo) ou passar h-auto e conferir no app; adicionar teste de que a coluna renderizada não tem size-full nem h-full |
| Medium | src/pages/board/BoardPage.tsx:31 | Pedido do usuário: a scrollbar horizontal ainda fica longe da borda inferior. O -mx-6 px-6 do KanbanBoard resolveu as laterais, mas o p-6 da página e o pb-2 do board mantêm a barra uns 30px acima do fim da janela | Usar px-6 pt-6 pb-0 no contêiner da página e deixar o espaço inferior dentro do conteúdo rolável (pb-4 no KanbanBoard), para a barra encostar na borda inferior; conferir no pnpm tauri dev |
| Medium | src/features/filter-board/ui/Toolbar.tsx:33 | Pedido do usuário: os rótulos Fonte, Alvo e Score mín. devem ficar na mesma linha dos campos, à esquerda, e não em cima | Usar Field orientation="horizontal" nos três campos, com FieldLabel em whitespace-nowrap, e voltar o cabeçalho do BoardPage para items-center para o título alinhar com a linha única de filtros |
| Medium | src/shared/ui/dialog.tsx:34 | Pedido do usuário (rodada 2): o dialog de detalhe ainda desfoca o board por trás. O fundo do conteúdo ficou sólido, mas o overlay do DialogOverlay mantém 'supports-backdrop-filter:backdrop-blur-xs' e o JobDialog não o sobrescreve | Tirar o backdrop-blur do overlay (no dialog.tsx, ou passando className sem blur ao ResponsiveDialogOverlay do JobDialog) e testar que o overlay do JobDialog não tem a classe de blur |

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

Rodada 4, julgada pelo painel Claude (Claude Code); o código é do opencode. A rodada 3 da trilha foi uma autoavaliação do próprio opencode (implementador e revisor iguais) e não conta como revisão. A cadeia do segundo revisor caiu inteira de novo: agy, Codex e Grok saíram com exit 3. Sem corroboração cross-model.

Das 15 pendências da rodada 2, 12 deixaram de existir no código: resync após move falho, clampScore com expoente, id fora da busca, cor da borda da sidebar, rodapé do crawler com 'Executar agora', '+ Adicionar vaga', dialog 42rem com fundo sólido, Score mín. 70, título alinhado, nome job-radar-desktop, Minimizar/Maximizar escondidos em tiling, prettier no tier1 e meta da spec. Continuam abertas, reescritas acima: altura das colunas, scrollbar e o blur do overlay. A correção do resync trouxe a regressão High da linha 25.

Exemplo próprio que quebrou, numa cópia descartável: reencerrar um card de 'hired' para 'rejected' (REQ-003).

Validação manual ainda pendente: arrasto por ponteiro e teclado, cancelamento ao encerrar, Ctrl+K com Enter, Ctrl+B após reinício, cores do Status no fundo Noir.
