# Revisão — sqlite-store

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
| C1 | sqlite_store::tests::moves_and_edits_survive_a_reopen | yes |
| C2 | sqlite_store::tests::sqlite_store_honors_the_store_contract | yes |
| C3 | store::tests::mock_store_honors_the_store_contract | yes |
| C4 | sqlite_store::tests::migrates_once_to_the_latest_version | yes |
| C5 | sqlite_store::tests::refuses_a_newer_schema_without_touching_it | yes |
| C6 | db_path::tests::picks_the_directory_for_the_build | yes |
| C7 | db_path::tests::env_override_wins_unless_empty | yes |
| C8 | sqlite_store::tests::database_files_are_private | yes |
| C9 | tests::open_store_seeds_only_debug_and_only_when_empty | yes |
| C10 | sqlite_store::tests::a_failed_write_leaves_the_data_intact | yes |
| C11 | sqlite_store::tests::unknown_stored_values_are_a_storage_error | yes |
| C12 | store::tests::storage_errors_hide_the_detail | yes |
| C13 | tests::open_store_reports_the_path_and_cause | yes |
| C14 | git diff --exit-code -- src/shared/bindings | yes |
| C15 | db_path::tests::env_override_keeps_a_non_unicode_path | yes |
| C16 | tests::open_configured_store_honors_the_env_override | yes |
| C17 | commands::tests::every_command_logs_the_storage_detail_and_hides_it | yes |
| C18 | tests::dialog_needs_a_display | yes |

<!-- checks:end -->

## Sem revisor independente

Esta rodada é autoavaliação: o Claude escreveu as correções, a pedido do usuário, e também as julgou. A cadeia agy -> Codex -> Grok retornou exit 3 nas três CLIs (auth/quota/rate-limit/network). Zero achado aqui não equivale a revisão externa; ela segue pendente, como as rodadas 1, 3 e 5, em que o opencode revisou o próprio código.

Corrigido nesta rodada: C6 e C7 testavam uma função que a produção não chamava; travamento sem display; teste de `report` que não falhava (confirmado por mutação em `update_card_impl`); `dirs` em major duplicado; formatação do ROADMAP; claims da spec.

Medido com o binário real e banco inválido: diálogo no Niri e exit 1 ao fechar em 8 de 8 repetições; exit 1 sem display. Em 2 execuções anteriores o processo saiu com exit 1 sem janela detectada, causa não identificada e não reproduzida; a causa saiu no stderr nas duas.

Pendente: persistência testada pela UI e `check_proofs.py` em árvore limpa, depois do commit (18 de 18 PASS em árvore suja nesta rodada).
