# Stitch prompts

Prompts used to design the screens in Google Stitch (project "job-radar desktop"). Each screen prompt is sent after the
shared base, so every screen matches the approved Board.

## Shared base

```text
Desktop app screen for "job-radar-desktop", a local-first job search tracker (Tauri + React,
shadcn/ui on Base UI). Canvas exactly 2560x1080 CSS px, scale 1. All UI text in Brazilian
Portuguese. Keep it visually identical to the existing Board screen of this project.

VISUAL STYLE "Terminal Noir": near-black background #0B0B0B with the rainy night city skyline
art at the bottom (charcoal buildings, a few amber #F5A524 and cream #F7D9A8 lit windows, one teal
#2DD4A7 window, thin rain, slate crescent moon top right). Panels and cards: #111113 / #16161A,
1px border #26262B, rounded-xl panels, rounded-lg cards, no blur, no heavy shadows.
Accent amber #F5A524 (primary actions, focus), teal #2DD4A7 (success), muted red #E5484D (errors).
Text #EDEDED, secondary #8B8B94. Inter for UI, JetBrains Mono only for numbers, dates, file sizes
and code. Comfortable spacing, nothing squashed, single-line buttons.

SHELL (same as the Board):
- Title bar 40px: amber dot + "job-radar-desktop", a small icon-only sidebar toggle next to it,
  only a close X on the far right (tiling window manager, no minimize/maximize).
- Left sidebar collapsed to a 48px icon rail: Board, Meu perfil, Meus currículos, Preparação,
  Configuração (the current page highlighted with an amber left indicator), small amber count
  badges on Meus currículos and Preparação. At the bottom, a teal status dot and a play icon
  button (crawler "Executar agora").
- Page content starts with a 24px gutter: page title (text-2xl) and its toolbar on one line.
```

## Meu perfil

```text
PAGE "Meu perfil": the career source of truth, a set of Markdown files about the user that the
AI actions read from. Layout: three panes.

LEFT PANE (300px, file tree panel): title "Arquivos do perfil", button "+ Novo arquivo", and a
list of .md files with document icons, last edited date in mono and a small "usado pela IA"
teal tag: "sobre-mim.md", "experiencias.md", "habilidades.md", "stack-tecnica.md",
"projetos.md", "formacao.md", "idiomas.md", "nao-incluir.md" (this one with a muted red tag
"nunca enviado"). The selected file has an amber left border.

CENTER PANE (flex, editor): file name "experiencias.md" with a "salvo há 2 min" mono status, a
segmented control "Editar | Visualizar | Dividido" (Dividido selected), then a split view:
left a Markdown source editor in JetBrains Mono with line numbers and headings like
"## Orbital Labs (nov/2025 a set/2026)" and bullet achievements; right the rendered preview with
headings, bullets and bold metrics. Fictional but realistic content in pt-BR.

RIGHT PANE (340px, summary cards): card "Resumo do perfil" with name, title "Engenheiro de
software sênior", location "São Paulo, SP", years of experience in mono; card "Habilidades"
with skill chips grouped by level (Avançado, Intermediário) like Python, TypeScript, Rust,
React, Next.js, AWS, ETL, LLMs; card "Dados técnicos" as a key/value list (Stack principal,
Pretensão PJ, Pretensão CLT, Modelo preferido "Remoto", Disponibilidade "Imediata");
card "Completo para a IA" with a progress ring "82%" and a checklist of missing items
("Adicionar métricas em 2 experiências").

Top toolbar: title "Meu perfil", search "Buscar no perfil...", button "Importar currículo (PDF)"
(outline) and primary amber "Salvar". Keyboard focus visible on the editor.
```

## Meus currículos

```text
PAGE "Meus currículos": generated and uploaded résumé PDFs, each one linked to jobs on the board.

TOOLBAR: title "Meus currículos", search "Buscar currículo ou vaga...", filters "Status"
(Todos, Rascunho, Para revisar, Aprovado, Enviado), "Vaga" (select), "Idioma" (pt-BR, en),
a view toggle with two icon buttons "Lista | Grade" (Grade selected), and primary amber
"+ Enviar PDF".

MAIN AREA split in two:
LEFT (about 60%): GRID of PDF cards, 4 per row. Each card: a real-looking first-page
thumbnail of a résumé (white page, dark text lines, a name header) on a dark frame; below it
file name in mono "cv-lumen-labs-staff-llm.pdf", size and date in mono ("214 KB · 02/11"),
a status pill (Status component: "Para revisar" amber, "Aprovado" teal, "Rascunho" gray,
"Enviado" cream), and linked job chips "Staff Engineer, LLM Platform · Lumen Labs" with a
small link icon. One card is selected with an amber border. One card shows "Claude gerando..."
with a pulsing indicator instead of the thumbnail.
Also show, as a small inset or second state, the LIST view: table rows with file icon, name,
linked job(s), status, language, version "v3", date, and row actions (abrir, baixar, mais).

RIGHT (about 40%): PDF READER PANEL for the selected file: header with file name, page counter
"Página 1 de 2" in mono, zoom "- 100% +", buttons "Baixar" and "Abrir em tela cheia"; the PDF
page rendered large and readable (fictional résumé in pt-BR: name, title, summary, experience
with bullet metrics, skills). Below the reader: "Vagas vinculadas" list with the linked
board cards (title, company, stage chip like "Candidatado") and a button "Vincular a outra
vaga"; and action buttons "Marcar como aprovado" (teal outline) and "Gerar nova versão".
```

## Preparação

```text
PAGE "Preparação": interview prep notes in Markdown, each file linked to a job and its
interview step, to read and edit before the interview.

TOOLBAR: title "Preparação", search "Buscar nas anotações...", filters "Vaga" and "Etapa"
(RH, Técnica, Case, Final), toggle "Somente próximas entrevistas" (checked), primary amber
"+ Novo roteiro".

LEFT PANE (380px): an "Agenda" strip at the top with the next interviews as compact cards
(date and time in mono, e.g. "Amanhã 15:00", company, step chip "TÉCNICA" amber, countdown
"em 22h"). Below it the list of prep files grouped by job: group header with job title,
company and board stage chip; files inside like "roteiro-tecnica-quasar-bio.md",
"perguntas-para-o-rh.md", "case-nexus-infra.md", each with a small checklist progress "5/8"
in mono. Selected file highlighted with an amber border.

CENTER PANE (flex): Markdown document "roteiro-tecnica-quasar-bio.md" in reading mode with an
"Editar" toggle: sections "Sobre a empresa", "Por que esta vaga", "Histórias STAR" (3 stories
with Situação/Tarefa/Ação/Resultado), "Perguntas técnicas prováveis" as a checklist with some
items checked, "Perguntas para fazer". Fictional pt-BR content. A floating small toolbar for
headings, bold, list, checklist.

RIGHT PANE (340px): card "Vaga vinculada" (mini job card: title, company, score pill, salary
"PJ R$ 34k · CLT R$ 26k" in mono, next action "Técnica amanhã 15:00", button "Abrir no board");
card "Checklist do dia" (testar câmera e áudio, revisar CV enviado v3, separar exemplos de
código); card "Ações de IA" with outline buttons "Gerar perguntas prováveis" and "Revisar
minhas respostas" (both showing a small "Ainda não implementado" hint).
```

## Configuração

```text
PAGE "Configuração": app and crawler settings, a single scrollable page with a left section
index.

LEFT INDEX (240px, sticky): "Geral", "Fontes de vagas", "Alvos e pontuação", "Crawler",
"Armazenamento", "CLIs de IA", "Notificações", "Aparência", "Sobre". Current section in amber.

RIGHT CONTENT (max 960px wide, sections as panels with a title and description, shadcn Field
layout, label left and control right):
- Geral: Idioma select (Português do Brasil, English), "Abrir no board ao iniciar" switch.
- Fontes de vagas: table of sources with toggle, name, type and health: Gupy, GitHub,
  ProgramaThor, Sólides, Greenhouse, Lever, Ashby, InHire, Alertas do Gmail; health as Status
  pills ("ok" teal, "falhando há 2 dias" red); button "+ Adicionar board ATS" and a textarea for
  ATS board slugs.
- Alvos e pontuação: target chips (AI/LLM, Full Stack, Tech Lead, Dados) with add/remove,
  number input "Score mínimo" 70, "Cortes obrigatórios" checkboxes (Somente remoto ou híbrido
  SP, Excluir estágio e júnior), keyword lists "Palavras que somam" and "Palavras que eliminam".
- Crawler: schedule select "A cada 6 horas", next run "hoje 14:00" in mono, last run summary
  (142 vagas, 9 novas, 38s, 8/9 fontes ok), buttons "Executar agora" and "Ver log".
- Armazenamento: radio cards "Local (SQLite)" selected with the file path in mono and
  "Postgres no servidor" (host, database, user fields disabled, a note that the password is
  kept in the system keyring), button "Testar conexão".
- CLIs de IA: rows for Claude Code, Codex and Grok with detected path in mono, version, a
  "Padrão" radio, and per-action allowlist chips ("Gmail somente leitura", "Sem shell");
  a warning callout that AI actions never send e-mail or run shell commands.
- Notificações: toggles for Discord and Telegram with masked webhook/token fields and
  "Enviar teste".
- Aparência: visual mode segmented control "Vidro | Sólido", "Mostrar skyline" switch.
- Sobre: version "0.1.0" in mono, links "Repositório", "Licença MIT", "Relatar problema".

Sticky bottom bar with "Descartar alterações" (ghost) and primary amber "Salvar".
```
