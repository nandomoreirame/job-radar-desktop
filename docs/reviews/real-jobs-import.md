# Revisão — real-jobs-import

- **Rodada:** 2026-11-02
- **Achados abertos:** 0
- **Estado:** ciclo fechado
- **Implementado por:** claude

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
| C1 | tests::open_store_never_seeds | yes |
| C2 | cargo build e grep | yes |
| C3 | sqlite_store::tests::migration_2_keeps_existing_rows | yes |
| C4 | sqlite_store::tests::a_card_without_score_round_trips | yes |
| C5 | JobCard shows no score label when the score is null | yes |
| C6 | filterCards keeps a card without a score at any minimum | yes |
| C7 | groupByStage sorts the inbox by score then newest | yes |
| C8 | import::tests::routes_seen_to_inbox_and_candidates_to_discarded | yes |
| C9 | cli::tests::parses_the_import_command | yes |
| C10 | import::tests::importing_twice_changes_nothing | yes |
| C11 | import::tests::seen_wins_over_candidates | yes |
| C12 | import::tests::skips_and_counts_invalid_records | yes |
| C13 | import::tests::unreadable_candidates_change_nothing | yes |
| C14 | identity::tests::url_key_matches_the_crawler | yes |
| C15 | import::tests::maps_a_record_to_a_card | yes |
| C16 | DiscardedPage lists and searches discarded jobs | yes |
| C17 | DiscardedPage has no axe violations | yes |
| C18 | DiscardedPage restores a job | yes |
| C19 | DiscardedPage deletes a job only after confirming | yes |
| C20 | import::tests::a_deleted_job_does_not_return | yes |
| C21 | commands::tests::opens_only_a_stored_http_url | yes |
| C22 | JobDialog opens the job link by id | yes |
| C23 | BoardPage shows the empty state with no cards | yes |
| C24 | sqlite_store::tests::sqlite_store_honors_the_store_contract | yes |

<!-- checks:end -->

## Sem revisor independente

Rodada 4, autoavaliação: o Claude corrigiu os achados restantes, a pedido do usuário, e julgou o próprio trabalho. agy, Codex e Grok retornaram exit 3. Zero achado aqui não equivale a revisão externa, que segue pendente.

Antes desta rodada, o commit 9301b16 do opencode reescreveu este arquivo com zero achados enquanto 10 seguiam abertos. Só o revisor escreve o estado de um achado.

Corrigido nesta rodada: os testes de importação, CLI, abrir link e página Descartadas agora são os do plano, sem resumo, mais casos para candidato malformado, URL com espaços e data impossível; abrir link com falha mostra um alerta nas duas telas; README nos dois idiomas; duplicatas de teste removidas; fixtures de paridade fora do prettier. Mutações conferidas: tirar a validação de URL, a aparação da URL ou a checagem de exclusão faz o teste correspondente falhar.

Pendente: `check_proofs.py` em árvore limpa depois do commit (24 de 24 PASS em árvore suja); verificação pela UI com as 207 vagas; apagar o banco de dev com os cards fictícios, com consentimento.
