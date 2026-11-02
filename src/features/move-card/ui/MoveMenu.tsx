import { MoreHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { OUTCOMES, STAGES } from "@/entities/job/model/stages";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Outcome } from "@/shared/bindings/Outcome";
import type { Stage } from "@/shared/bindings/Stage";
import { Button } from "@/shared/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/shared/ui/dropdown-menu";

type Props = { card: BoardCard; onMove: (stage: Stage, outcome: Outcome | null) => void };

/** Keyboard path for every move, and the only way to close with an explicit outcome. */
export function MoveMenu({ card, onMove }: Props) {
  const { t } = useTranslation();
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <Button variant="ghost" size="icon" className="size-7" aria-label={t("card.menu", { title: card.title })} />
        }
      >
        <MoreHorizontal aria-hidden="true" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuGroup>
          <DropdownMenuLabel>{t("move.to")}</DropdownMenuLabel>
          {STAGES.filter((stage) => stage !== "closed" && stage !== card.stage).map((stage) => (
            <DropdownMenuItem key={stage} onClick={() => onMove(stage, null)}>
              {t(`stage.${stage}`)}
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuGroup>
          <DropdownMenuLabel>{t("move.closeAs")}</DropdownMenuLabel>
          {OUTCOMES.filter((outcome) => outcome !== card.outcome).map((outcome) => (
            <DropdownMenuItem key={outcome} onClick={() => onMove("closed", outcome)}>
              {t(`outcome.${outcome}`)}
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
