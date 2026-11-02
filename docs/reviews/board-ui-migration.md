# Revisão — board-ui-migration

- **Rodada:** 2026-11-02
- **Achados abertos:** 15
- **Estado:** aguardando correção
- **Implementado por:** opencode

> Arquivo reescrito por inteiro a cada `/rpi-review`. Um achado só desaparece
> daqui por ter deixado de existir no código: quem corrige não marca resolvido.

<!-- findings:begin -->

| Severidade | Arquivo:linha | Problema | Correção sugerida |
|------------|---------------|----------|-------------------|
| High | src/widgets/board/model/use-kanban-columns.ts:15 | REQ-003: a guarda por assinatura JSON ignora a lista nova e igual que o useBoard devolve quando o move_card falha, então um card arrastado para outra coluna continua lá mesmo com o store recusando o movimento (o rerender com [a, b] após mover 'a' para triage manteve inbox = ['b']) | Ressincronizar pela identidade do array (remover cardsSignature) ou chamar restore() quando o move falhar; adicionar teste que move uma coluna, faz rerender com uma cópia igual dos cards e espera as colunas do store |
| High | src/features/filter-board/model/filter.ts:7 | REQ-008: clampScore usa parseInt, então um valor válido de input number como '1e3' vira 1 em vez de 100 (o mesmo vale para '9e1' virar 9) | Usar Number(raw) com Number.isFinite, arredondar e limitar a 0-100; cobrir expoente e decimal no teste do clampScore |
| High | src/features/search-jobs/ui/JobCommand.tsx:6 | REQ-007: o value do CommandItem inclui o id interno do card, então a paleta casa buscas pelo id ('c0' lista todas as vagas e 'c1' traz c10 a c17), que o usuário não vê | Usar value={card.id} e keywords={[card.title, card.company]}, com o filter do Command casando só as keywords (filter recebe value, search, keywords); testar que um id não encontra nada |
| Medium | src/shared/styles/theme.css:1 | Pedido do usuário: aparece uma linha branca na divisão entre a sidebar e o board. Causa: o theme.css não tem a regra base do shadcn, e no Tailwind 4 a cor padrão de borda é currentColor, então o border-r da sidebar (e qualquer outra borda sem cor) sai na cor do texto | Adicionar ao theme.css `@layer base { * { @apply border-border outline-ring/50; } }` e conferir no `pnpm tauri dev` que a divisória fica em #26262b |
| Medium | src/widgets/app-shell/Sidebar.tsx:9 | Pedido do usuário: remover o 'terminal noir •' do rodapé da sidebar e mostrar mais informação do crawler: além de 'Crawler ativo' e da última varredura, a duração da última execução, a próxima execução agendada e as fontes com problema (ex.: '8/9 fontes ok'), mais um botão 'Executar agora' (ícone com tooltip quando a sidebar está recolhida) | Remover o parágrafo app.signature (e a chave app.signature das duas locales); estender o mock CRAWLER_STATUS com lastDurationSec, nextRun, sourcesOk, sourcesTotal; adicionar o botão 'Executar agora' desabilitado com 'Em breve' (como o Sincronizar Gmail) até o crawler existir no app; novas chaves crawler.* em pt-BR e en; teste que o rodapé mostra as novas informações e o botão |
| Medium | src/widgets/board/Board.tsx:33 | Pedido do usuário: as colunas estão estreitas e com espaçamento interno grande; e esticam até o fim da tela mesmo com poucos cards, quando deviam acompanhar a altura do conteúdo | Aumentar a coluna para auto-cols-[minmax(340px,1fr)], alinhar a grade com items-start e limitar cada coluna a max-h-full com rolagem interna; em BoardColumn.tsx reduzir o padding (header px-3 py-2.5, lista px-2 pb-2 gap-2) e no JobCard trocar p-4 por p-3 |
| Medium | src/widgets/board/BoardColumn.tsx:16 | Pedido do usuário: a primeira coluna (Caixa de entrada) precisa de um botão '+ Adicionar vaga' no fim da lista, para cadastrar uma vaga manualmente | Renderizar só na coluna inbox um botão de largura total '+ Adicionar vaga', desabilitado com 'Em breve' até existir o formulário e o comando add_card no Rust; novas chaves em pt-BR e en; teste que só a Caixa de entrada tem o botão |
| Medium | src/entities/job/ui/JobDialog.tsx:24 | Pedido do usuário: o dialog de detalhe está estreito e translúcido. A classe noir-panel aplica o gradiente e o backdrop-blur do modo glass, e o max-w-2xl perde para o sm:max-w-sm do DialogContent gerado, por isso a largura não muda | Trocar noir-panel por fundo sólido (bg-popover ou noir-card), usar sm:max-w-[42rem] (1,5x a largura atual) e remover o backdrop-blur do overlay só neste dialog (className no ResponsiveDialogOverlay ou prop do conteúdo); teste que o conteúdo não tem noir-panel |
| Medium | src/features/filter-board/model/filter.ts:4 | Pedido do usuário: o campo Score mín. deve abrir com 70 preenchido, não 0 | Criar DEFAULT_FILTERS com minScore: 70 e usar no estado inicial do BoardPage, mantendo EMPTY_FILTERS como 'sem filtro'; ajustar os testes que dependem do board inteiro e testar que o input começa em 70 |
| Medium | src/widgets/app-shell/TitleBar.tsx:20 | Pedido do usuário: em compositores tiling (Niri, Hyprland, Sway, i3, river e outros) os botões Minimizar e Maximizar não fazem sentido e devem sumir; só Fechar fica | Criar no Rust uma função pura is_tiling_wm(env) que olha XDG_CURRENT_DESKTOP/XDG_SESSION_DESKTOP e NIRI_SOCKET, HYPRLAND_INSTANCE_SIGNATURE, SWAYSOCK, I3SOCK, exposta por um comando window_controls; o TitleBar esconde minimize e toggleMaximize quando ele disser tiling; testes no Rust (cada compositor e um desktop flutuante) e no TitleBar |
| Low | src/pages/board/BoardPage.tsx:32 | Pedido do usuário: o título 'Board' não alinha com os campos de filtro, porque o cabeçalho usa items-center e os campos têm rótulo em cima | Usar items-end no cabeçalho e dar ao h1 a altura dos controles (h-9 com leading-9), para o texto alinhar com a linha dos inputs |
| Low | src/pages/board/BoardPage.tsx:31 | Pedido do usuário: a scrollbar horizontal do board deve encostar nas bordas da janela (inferior, esquerda e direita), mas o p-6 da página a afasta delas | Tirar o padding horizontal e inferior do contêiner da página para a área do board e aplicar px-6 no conteúdo rolável do KanbanBoard (padding interno), deixando a barra de rolagem rente às bordas |
| Low | src/widgets/app-shell/TitleBar.tsx:32 | Pedido do usuário: o nome no topo deve ser 'job-radar-desktop', não 'job-radar' | Trocar app.name para 'job-radar-desktop' nas duas locales e o title da janela em src-tauri/tauri.conf.json; ajustar os testes que procuram 'job-radar' |
| Low | src/features/search-jobs/ui/JobCommand.tsx:6 | Componentes do diff foram gravados com o JSX inteiro em uma linha (JobCommand, AppShell, Sidebar, Board, BoardColumn, use-kanban-columns), e o portão de prettier ficou pulado por falta da ferramenta, então nada reformata | Adicionar prettier como devDependency com script format e formatar os arquivos de src (fora src/shared/ui gerado), para o tier1 deixar de pular o portão |
| Low | docs/specs/board-ui-migration/SPEC.md:12 | A meta diz que o radix-ui sai da árvore de dependências, mas o cmdk (Command do shadcn) traz @radix-ui/react-dialog transitivamente; o REQ-001 só vale para dependência direta e imports em src | Reescrever a meta para 'radix-ui deixa de ser dependência direta e de ser importado em src; o cmdk ainda traz @radix-ui/react-dialog' |

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

Rodada 2, julgada pelo painel Claude (Claude Code), que não implementou a rodada: o código é do opencode. A cadeia do segundo revisor foi tentada inteira e caiu: agy, Codex e Grok saíram com exit 3 (auth/quota/rate-limit/network). Não há corroboração cross-model.

Exemplos próprios que quebraram, rodados numa cópia descartável: rerender com lista igual após move falho (REQ-003), clampScore('1e3') (REQ-008) e busca 'c0' na paleta (REQ-007). Os três viraram achados High.

## Ajustes de interface pedidos pelo usuário

Os achados marcados com 'Pedido do usuário' vieram da validação visual do usuário no `pnpm tauri dev` (Niri, 2026-11-02) e entram nesta rodada a pedido dele. Não têm requisito correspondente na spec; o implementador deve tratá-los como escopo desta feature. Os botões '+ Adicionar vaga' e 'Executar agora' ficam desabilitados com 'Em breve' até existirem o cadastro manual e o crawler no app.

Validação manual ainda pendente: arrasto por ponteiro e teclado, cancelamento ao encerrar, Ctrl+K com Enter, Ctrl+B após reinício e cores do Status no fundo Noir.
