# Revisão — real-jobs-import

- **Rodada:** 2026-11-02
- **Achados abertos:** 10
- **Estado:** aguardando correção
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
| C1 | tests::open_store_never_seeds | yes |
| C2 | cargo build e grep | yes |
| C3 | sqlite_store::tests::migration_2_keeps_existing_rows | yes |
| C4 | sqlite_store::tests::a_card_without_score_round_trips | yes |
| C5 | JobCard shows no score label | yes |
| C6 | filterCards keeps a card without a score | yes |
| C7 | groupByStage sorts the inbox | yes |
| C8 | import::tests::routes_seen_to_inbox_and_candidates_to_discarded | no - não passa candidates.json |
| C9 | cli::tests::parses_the_import_command | no - um caso de sucesso |
| C10 | import::tests::importing_twice_changes_nothing | no - não confere edições do usuário |
| C11 | import::tests::seen_wins_over_candidates | no - não usa candidates.json |
| C12 | import::tests::skips_and_counts_invalid_records | no - só seen.db; item inválido em candidates aborta tudo |
| C13 | import::tests::unreadable_candidates_change_nothing | yes |
| C14 | identity::tests::url_key_matches_the_crawler | yes |
| C15 | import::tests::maps_a_record_to_a_card | no - só conta cards |
| C16 | DiscardedPage lists and searches | no - só o estado vazio |
| C17 | DiscardedPage has no axe violations | no - expect(true) |
| C18 | DiscardedPage restores a job | no - expect(true) |
| C19 | DiscardedPage deletes a job only after confirming | no - expect(true) |
| C20 | import::tests::a_deleted_job_does_not_return | no - compara uma constante |
| C21 | commands::tests::opens_only_a_stored_http_url | no - só no_url |
| C22 | JobDialog opens the job link by id | yes |
| C23 | BoardPage shows the empty state | yes |
| C24 | sqlite_store::tests::sqlite_store_honors_the_store_contract | yes |

<!-- checks:end -->

## Revisão degradada

Julgamento único do Claude. agy, Codex e Grok retornaram exit 3 de novo, sem corroboração cross-model.

Rodada de correção (4632761): fechou a data malformada no evento found (`is_iso_date`) e o label sem limite. Os testes de importação passaram a chamar `import_history`, mas ainda não cobrem candidates.json nem a exclusão definitiva. Os outros achados não foram tocados.
