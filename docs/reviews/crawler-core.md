# Revisão — crawler-core

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
| C1 | crawler::identity::tests::matches_the_python_vectors | yes |
| C2 | crawler::filters::tests::matches_the_python_vectors | yes |
| C3 | crawler::scoring::tests::matches_the_python_vectors | yes |
| C4 | crawler::profile::tests::matches_the_python_skills | yes |
| C5 | crawler::config::tests::refuses_a_missing_or_empty_profile | yes |
| C6 | crawler::config::tests::reads_defaults_and_names_the_invalid_field | yes |
| C7 | crawler::http::tests::honors_robots_pacing_and_retries | yes |
| C8 | crawler::http::tests::bounds_the_body_and_refuses_other_schemes | yes |
| C9 | crawler::robots::tests::matches_the_python_vectors | yes |
| C10 | crawler::sources::gupy::tests::parses_the_recorded_search_like_python | yes |
| C11 | crawler::sources::solides::tests::parses_the_recorded_pages_like_python | yes |
| C12 | crawler::pipeline::tests::routes_new_jobs_and_skips_known_ones | yes |
| C13 | crawler::pipeline::tests::a_failing_source_does_not_stop_the_others | yes |
| C14 | crawler::pipeline::tests::rescoring_keeps_user_edits_and_stage | yes |
| C15 | crawler::pipeline::tests::a_stored_description_needs_no_request | yes |
| C16 | crawler::pipeline::tests::a_low_rescore_stays_in_the_inbox | yes |
| C17 | crawler::pipeline::tests::closed_discards_and_failures_keep | yes |
| C18 | crawler::pipeline::tests::circuit_breaker_stops_a_mass_discard | yes |
| C19 | crawler::sources::tests::detail_outcomes_follow_the_closed_rules | yes |
| C20 | crawler::runner::tests::refuses_a_second_run | yes |
| C21 | CrawlerCard.test.tsx::disables the button while running | yes |
| C22 | sqlite_store::tests::the_last_run_survives_a_reopen | yes |
| C23 | CrawlerCard.test.tsx::explains a never run and a refused run | yes |

<!-- checks:end -->
