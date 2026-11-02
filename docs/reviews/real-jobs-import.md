# Revisão — real-jobs-import

- **Rodada:** 2026-11-02
- **Achados abertos:** 12
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
| C8 | import::tests::routes_seen_to_inbox_and_candidates_to_discarded | no - só chama valid() |
| C9 | cli::tests::parses_the_import_command | no - um caso de sucesso, sem flag inválida |
| C10 | import::tests::importing_twice_changes_nothing | no - só chama valid() |
| C11 | import::tests::seen_wins_over_candidates | no - só chama url_key |
| C12 | import::tests::skips_and_counts_invalid_records | no - só chama valid(); o caso sem title aborta tudo |
| C13 | import::tests::unreadable_candidates_change_nothing | no - não confere que o banco ficou igual |
| C14 | identity::tests::url_key_matches_the_crawler | yes |
| C15 | import::tests::maps_a_record_to_a_card | no - só chama date_or |
| C16 | DiscardedPage lists and searches | no - só o estado vazio |
| C17 | DiscardedPage has no axe violations | no - expect(true) |
| C18 | DiscardedPage restores a job | no - expect(true) |
| C19 | DiscardedPage deletes a job only after confirming | no - expect(true) |
| C20 | import::tests::a_deleted_job_does_not_return | no - compara uma constante |
| C21 | commands::tests::opens_only_a_stored_http_url | no - só o caso no_url |
| C22 | JobDialog opens the job link by id | yes |
| C23 | BoardPage shows the empty state | yes |
| C24 | sqlite_store::tests::sqlite_store_honors_the_store_contract | yes |

<!-- checks:end -->

## Revisão degradada

Julgamento único do Claude. A cadeia agy -> Codex -> Grok retornou exit 3 nas três CLIs (auth/quota/rate-limit/network), sem corroboração cross-model. O código é do opencode; a spec e o plano são do Claude. A rodada 1 da trilha, com o opencode revisando o próprio código, não conta como fechamento.

Medido numa cópia descartável, com os testes reais do plano e as cópias de `seen.db` e `candidates.json` do servidor: `inbox: 207`, `discarded: 137`; a segunda execução deu `duplicates: 344` e nenhuma inserção; arquivo ausente deu exit 1; banco em 0600. Os dados reais foram apagados da cópia depois da medição.

Pendente: verificação pela UI com 207 cards, abrir um link, restaurar e excluir pela tela; apagar o banco de dev com os cards fictícios, com consentimento.
