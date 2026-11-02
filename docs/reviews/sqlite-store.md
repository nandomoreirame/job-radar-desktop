# Revisão — sqlite-store

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

<!-- checks:end -->

## Revisão degradada

A implementação foi validada pelo próprio opencode. A revisão independente foi tentada pela cadeia agy -> Codex -> Grok; os três retornaram exit 3 por auth/quota/rate-limit/network. Não há corroboração cross-model.

Validação manual pendente: `pnpm tauri dev` para confirmar o banco no diretório correto, persistência após reinício e o diálogo nativo ao abrir uma base corrompida.
