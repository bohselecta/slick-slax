import { Check } from "lucide-react";
import type { Step } from "../types";

const items: { id: Step; label: string }[] = [
  { id: "source", label: "Slax" },
  { id: "drive", label: "Drive" },
  { id: "options", label: "Setup" },
  { id: "write", label: "Make" },
];

export function StepRail({ step }: { step: Step }) {
  const current = step === "done" ? items.length : items.findIndex((item) => item.id === step);
  return (
    <nav className="step-rail" aria-label="Installer progress">
      {items.map((item, index) => {
        const complete = index < current;
        const active = index === current;
        return (
          <div className={`step-pill ${complete ? "is-complete" : ""} ${active ? "is-active" : ""}`} key={item.id}>
            <span>{complete ? <Check size={13} strokeWidth={3} /> : index + 1}</span>
            {item.label}
          </div>
        );
      })}
    </nav>
  );
}

