import { useTranslation } from "react-i18next";
import { OUTCOMES } from "@/entities/job/model/stages";
import type { Outcome } from "@/shared/bindings/Outcome";
import { Button } from "@/shared/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/shared/ui/dialog";

type Props = { open: boolean; onPick: (outcome: Outcome) => void; onCancel: () => void };

export function OutcomeDialog({ open, onPick, onCancel }: Props) {
  const { t } = useTranslation();
  return (
    <Dialog open={open} onOpenChange={(next) => !next && onCancel()}>
      <DialogContent showCloseButton={false} className="noir-panel">
        <DialogHeader>
          <DialogTitle>{t("move.pickOutcome")}</DialogTitle>
          <DialogDescription className="sr-only">{t("move.closeAs")}</DialogDescription>
        </DialogHeader>
        <div className="grid grid-cols-2 gap-2">
          {OUTCOMES.map((outcome) => (
            <Button key={outcome} variant="outline" onClick={() => onPick(outcome)}>
              {t(`outcome.${outcome}`)}
            </Button>
          ))}
        </div>
        <DialogFooter>
          <Button variant="ghost" onClick={onCancel}>
            {t("move.cancel")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
