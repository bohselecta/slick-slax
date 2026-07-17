import { useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  ArrowLeft,
  ArrowRight,
  Check,
  ChevronRight,
  CircleHelp,
  Download,
  ExternalLink,
  FileArchive,
  HardDrive,
  LoaderCircle,
  LockKeyhole,
  RefreshCw,
  RotateCcw,
  ShieldCheck,
  Sparkles,
  Usb,
  X,
  Zap,
} from "lucide-react";
import { Brand, CloverMark } from "./components/Brand";
import { StepRail } from "./components/StepRail";
import {
  chooseIso,
  listDrives,
  nativeAvailable,
  openOfficialDownload,
  runDemoInstall,
  startInstall,
  subscribeToProgress,
} from "./lib/bridge";
import { formatBytes, shortDeviceName } from "./lib/format";
import type { InstallOptions, InstallProgress, IsoInfo, Step, UsbDrive } from "./types";

const defaultOptions: InstallOptions = {
  erase: true,
  label: "SLAX",
  persistenceGb: 16,
  verify: true,
};

function App() {
  const [step, setStep] = useState<Step>("source");
  const [iso, setIso] = useState<IsoInfo | null>(null);
  const [drives, setDrives] = useState<UsbDrive[]>([]);
  const [selectedDriveId, setSelectedDriveId] = useState<string>();
  const [options, setOptions] = useState(defaultOptions);
  const [progress, setProgress] = useState<InstallProgress | null>(null);
  const [scanning, setScanning] = useState(false);
  const [error, setError] = useState<string>();
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [confirmation, setConfirmation] = useState("");

  const selectedDrive = useMemo(
    () => drives.find((drive) => drive.id === selectedDriveId),
    [drives, selectedDriveId],
  );

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void subscribeToProgress((next) => {
      setProgress(next);
      if (next.phase === "complete") setStep("done");
      if (next.phase === "error") setError(next.detail);
    }).then((cleanup) => { unlisten = cleanup; });
    return () => unlisten?.();
  }, []);

  async function scanDrives() {
    setScanning(true);
    setError(undefined);
    try {
      const found = await listDrives();
      setDrives(found);
      if (found.length === 1) setSelectedDriveId(found[0].id);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setScanning(false);
    }
  }

  async function pickIso() {
    setError(undefined);
    try {
      const picked = await chooseIso();
      if (picked) setIso(picked);
    } catch (reason) {
      setError(String(reason));
    }
  }

  async function goToDrives() {
    setStep("drive");
    if (!drives.length) await scanDrives();
  }

  function goToOptions() {
    if (!selectedDrive || !iso) return;
    const windows = navigator.userAgent.includes("Windows");
    const allowed = [4, 8, 16, 32, 64].filter((size) =>
      size * 1_000_000_000 < selectedDrive.sizeBytes - iso.sizeBytes && (!windows || size <= 16),
    );
    if (!allowed.includes(options.persistenceGb)) {
      setOptions({ ...options, persistenceGb: allowed.at(-1) ?? 4 });
    }
    setStep("options");
  }

  async function beginInstall() {
    if (!iso || !selectedDrive || confirmation !== "SLAX") return;
    setConfirmOpen(false);
    setStep("write");
    setError(undefined);
    const request = {
      isoPath: iso.path,
      driveId: selectedDrive.id,
      device: selectedDrive.device,
      options,
      confirmation,
    };
    if (!nativeAvailable) {
      runDemoInstall((next) => {
        setProgress(next);
        if (next.phase === "complete") setStep("done");
      });
      return;
    }
    try {
      await startInstall(request);
    } catch (reason) {
      setError(String(reason));
      setProgress({ phase: "error", percent: 0, title: "Installation stopped", detail: String(reason) });
    }
  }

  function restart() {
    setStep("source");
    setIso(null);
    setSelectedDriveId(undefined);
    setOptions(defaultOptions);
    setProgress(null);
    setConfirmation("");
    setError(undefined);
  }

  return (
    <div className="app-shell">
      <div className="ambient ambient--one" />
      <div className="ambient ambient--two" />
      <header className="app-header">
        <Brand />
        <StepRail step={step} />
        <a className="icon-link" href="https://www.slax.org/starting.php" target="_blank" rel="noreferrer" aria-label="Slax help">
          <CircleHelp size={19} />
        </a>
      </header>

      <main className="workspace">
        {error && step !== "write" && (
          <div className="error-banner" role="alert">
            <AlertTriangle size={18} />
            <span>{error}</span>
            <button onClick={() => setError(undefined)} aria-label="Dismiss"><X size={16} /></button>
          </div>
        )}

        {step === "source" && <SourceStep iso={iso} onPick={pickIso} onDownload={openOfficialDownload} onContinue={goToDrives} />}
        {step === "drive" && (
          <DriveStep
            drives={drives}
            selectedId={selectedDriveId}
            scanning={scanning}
            onSelect={setSelectedDriveId}
            onRefresh={scanDrives}
            onBack={() => setStep("source")}
            onContinue={goToOptions}
          />
        )}
        {step === "options" && selectedDrive && iso && (
          <OptionsStep
            drive={selectedDrive}
            iso={iso}
            options={options}
            onChange={setOptions}
            onBack={() => setStep("drive")}
            onContinue={() => setConfirmOpen(true)}
          />
        )}
        {step === "write" && <WriteStep progress={progress} error={error} onBack={() => setStep("options")} />}
        {step === "done" && selectedDrive && <DoneStep drive={selectedDrive} onRestart={restart} />}
      </main>

      <footer className="app-footer">
        <span><ShieldCheck size={14} /> Open source. No telemetry.</span>
        <span className="footer-center">Built for Slax, with care.</span>
        <span>v0.1.0</span>
      </footer>

      {confirmOpen && selectedDrive && (
        <ConfirmDialog
          drive={selectedDrive}
          erase={options.erase}
          confirmation={confirmation}
          onConfirmation={setConfirmation}
          onCancel={() => setConfirmOpen(false)}
          onConfirm={beginInstall}
        />
      )}
    </div>
  );
}

function SourceStep({
  iso,
  onPick,
  onDownload,
  onContinue,
}: {
  iso: IsoInfo | null;
  onPick: () => void;
  onDownload: () => void;
  onContinue: () => void;
}) {
  return (
    <section className="screen screen--source">
      <div className="eyebrow"><Sparkles size={14} /> Your pocket OS, without the scavenger hunt</div>
      <h1>Let’s make a <span>Slax drive.</span></h1>
      <p className="lede">Choose your Slax ISO. SlickSlax handles the fussy bits—MBR, FAT32, copying, boot setup, and verification—in one safe pass.</p>

      {!iso ? (
        <div className="source-grid">
          <button className="choice-card choice-card--primary" onClick={onPick}>
            <div className="choice-icon"><FileArchive size={28} /></div>
            <span className="choice-kicker">Already downloaded?</span>
            <strong>Choose a Slax ISO</strong>
            <small>Select a .iso file from this computer</small>
            <span className="card-action">Browse files <ArrowRight size={16} /></span>
          </button>
          <button className="choice-card" onClick={onDownload}>
            <div className="choice-icon choice-icon--quiet"><Download size={28} /></div>
            <span className="choice-kicker">Need Slax?</span>
            <strong>Get it from Slax.org</strong>
            <small>Choose Debian or Slackware, 32 or 64-bit</small>
            <span className="card-action card-action--quiet">Open official download <ExternalLink size={15} /></span>
          </button>
        </div>
      ) : (
        <div className="selected-iso">
          <div className="file-badge"><FileArchive size={30} /></div>
          <div className="file-copy">
            <span>SLAX IMAGE</span>
            <strong>{iso.filename}</strong>
            <p>{[iso.edition, iso.architecture, formatBytes(iso.sizeBytes)].filter(Boolean).join("  ·  ")}</p>
          </div>
          <div className="verified"><Check size={15} /> {iso.slaxRootFound ? "Slax structure found" : "Slax ISO selected"}</div>
          <button className="text-button" onClick={onPick}>Change</button>
        </div>
      )}

      <div className="promise-row">
        <span><Zap size={16} /> One guided flow</span>
        <span><ShieldCheck size={16} /> Removable drives only</span>
        <span><LockKeyhole size={16} /> Confirmation before erase</span>
      </div>

      <div className="screen-actions screen-actions--right">
        <button className="primary-button" disabled={!iso} onClick={onContinue}>Choose a drive <ArrowRight size={17} /></button>
      </div>
    </section>
  );
}

function DriveStep({
  drives,
  selectedId,
  scanning,
  onSelect,
  onRefresh,
  onBack,
  onContinue,
}: {
  drives: UsbDrive[];
  selectedId?: string;
  scanning: boolean;
  onSelect: (id: string) => void;
  onRefresh: () => void;
  onBack: () => void;
  onContinue: () => void;
}) {
  return (
    <section className="screen">
      <div className="section-heading">
        <div><span className="section-number">02</span><h1>Pick the right <span>USB drive.</span></h1></div>
        <button className="secondary-button" onClick={onRefresh} disabled={scanning}>
          <RefreshCw size={15} className={scanning ? "spin" : ""} /> Scan again
        </button>
      </div>
      <p className="lede lede--compact">Only removable USB devices are shown. Unplug anything you don’t want to touch.</p>

      <div className="drive-list">
        {drives.length === 0 && !scanning && (
          <div className="empty-drive">
            <Usb size={34} />
            <strong>No removable drives found</strong>
            <span>Plug in a USB drive, then scan again.</span>
          </div>
        )}
        {scanning && drives.length === 0 && (
          <div className="empty-drive"><LoaderCircle className="spin" size={34} /><strong>Looking for USB drives…</strong></div>
        )}
        {drives.map((drive) => (
          <button
            key={drive.id}
            className={`drive-card ${selectedId === drive.id ? "is-selected" : ""}`}
            onClick={() => onSelect(drive.id)}
            disabled={drive.system}
          >
            <span className="radio-dot"><Check size={13} /></span>
            <span className="drive-icon"><Usb size={26} /></span>
            <span className="drive-main">
              <strong>{drive.vendor ? `${drive.vendor} ` : ""}{drive.name}</strong>
              <small>{shortDeviceName(drive.device)} · {drive.mountPoints[0] || "Not mounted"}</small>
            </span>
            <span className="drive-facts">
              <strong>{formatBytes(drive.sizeBytes)}</strong>
              <small>{[drive.partitionScheme, drive.filesystem].filter(Boolean).join(" · ") || "Unformatted"}</small>
            </span>
            <ChevronRight size={18} className="drive-chevron" />
          </button>
        ))}
      </div>

      <div className="info-strip"><ShieldCheck size={18} /><span><strong>Built-in guardrail:</strong> internal and system disks are excluded by the native scanner.</span></div>
      <div className="screen-actions">
        <button className="back-button" onClick={onBack}><ArrowLeft size={17} /> Back</button>
        <button className="primary-button" disabled={!selectedId} onClick={onContinue}>Set it up <ArrowRight size={17} /></button>
      </div>
    </section>
  );
}

function OptionsStep({ drive, iso, options, onChange, onBack, onContinue }: {
  drive: UsbDrive;
  iso: IsoInfo;
  options: InstallOptions;
  onChange: (options: InstallOptions) => void;
  onBack: () => void;
  onContinue: () => void;
}) {
  const windows = navigator.userAgent.includes("Windows");
  const persistenceOptions = [4, 8, 16, 32, 64].filter((size) =>
    size * 1_000_000_000 < drive.sizeBytes - iso.sizeBytes && (!windows || size <= 16),
  );
  return (
    <section className="screen">
      <div className="section-heading"><div><span className="section-number">03</span><h1>Make it <span>yours.</span></h1></div></div>
      <p className="lede lede--compact">Sensible defaults are ready. Adjust them if you know what you want.</p>

      <div className="options-layout">
        <div className="option-stack">
          <div className="option-card">
            <div className="option-copy"><span className="mini-icon"><HardDrive size={17} /></span><div><strong>Prepare the entire drive</strong><small>Erase it, create the required MBR + FAT32 layout, then install Slax.</small></div></div>
            <button className={`switch ${options.erase ? "is-on" : ""}`} onClick={() => onChange({ ...options, erase: !options.erase })} role="switch" aria-checked={options.erase}><span /></button>
          </div>

          <div className="option-card option-card--vertical">
            <div className="option-copy"><span className="mini-icon"><RotateCcw size={17} /></span><div><strong>Persistent storage</strong><small>Space for saved sessions, settings, and installed apps.</small></div></div>
            <div className="size-picker">
              {persistenceOptions.map((size) => <button key={size} className={options.persistenceGb === size ? "is-active" : ""} onClick={() => onChange({ ...options, persistenceGb: size })}>{size} GB</button>)}
            </div>
          </div>

          <label className="option-card input-card">
            <div className="option-copy"><span className="mini-icon"><Usb size={17} /></span><div><strong>Drive name</strong><small>Shown in Finder, Explorer, and file managers.</small></div></div>
            <input value={options.label} maxLength={11} onChange={(event) => onChange({ ...options, label: event.target.value.toUpperCase().replace(/[^A-Z0-9_-]/g, "") })} />
          </label>

          <div className="option-card">
            <div className="option-copy"><span className="mini-icon"><ShieldCheck size={17} /></span><div><strong>Verify after writing</strong><small>Check the Slax folder, bootloader, and critical files.</small></div></div>
            <button className={`switch ${options.verify ? "is-on" : ""}`} onClick={() => onChange({ ...options, verify: !options.verify })} role="switch" aria-checked={options.verify}><span /></button>
          </div>
        </div>

        <aside className="receipt-card">
          <span className="receipt-label">READY TO MAKE</span>
          <div className="receipt-drive"><Usb size={24} /><div><strong>{drive.vendor} {drive.name}</strong><small>{formatBytes(drive.sizeBytes)} · {shortDeviceName(drive.device)}</small></div></div>
          <div className="receipt-line"><span>Source</span><strong>{iso.architecture || "Slax ISO"}</strong></div>
          <div className="receipt-line"><span>Format</span><strong>MBR · FAT32</strong></div>
          <div className="receipt-line"><span>Persistence</span><strong>{options.persistenceGb} GB</strong></div>
          <div className="receipt-line"><span>Verification</span><strong>{options.verify ? "On" : "Off"}</strong></div>
          <div className="receipt-note"><AlertTriangle size={16} /><span>{options.erase ? "Everything currently on this USB drive will be erased." : "Existing files stay, but the Slax folder and boot setup may be replaced."}</span></div>
        </aside>
      </div>

      <div className="screen-actions">
        <button className="back-button" onClick={onBack}><ArrowLeft size={17} /> Back</button>
        <button className="primary-button" onClick={onContinue}><Zap size={17} fill="currentColor" /> Make my Slax drive</button>
      </div>
    </section>
  );
}

function WriteStep({ progress, error, onBack }: { progress: InstallProgress | null; error?: string; onBack: () => void }) {
  const current = progress ?? { phase: "preparing", percent: 2, title: "Getting ready", detail: "Starting the installer" };
  const phases = ["Prepare", "Format", "Copy", "Boot", "Verify"];
  const phaseMap: Record<string, number> = { preparing: 0, formatting: 1, copying: 2, bootloader: 3, verifying: 4, complete: 5, error: 0 };
  const activePhase = phaseMap[current.phase] ?? 0;
  if (error || current.phase === "error") {
    return (
      <section className="screen write-screen">
        <div className="failure-mark"><AlertTriangle size={35} /></div>
        <span className="write-kicker">INSTALLATION STOPPED SAFELY</span>
        <h1>Nothing was hidden.</h1>
        <p className="lede lede--compact">{error || current.detail}</p>
        <button className="secondary-button" onClick={onBack}><ArrowLeft size={16} /> Review setup</button>
      </section>
    );
  }
  return (
    <section className="screen write-screen">
      <div className="progress-orbit"><CloverMark /><svg viewBox="0 0 120 120"><circle cx="60" cy="60" r="54" /><circle cx="60" cy="60" r="54" style={{ strokeDashoffset: 339 - (339 * current.percent) / 100 }} /></svg><strong>{current.percent}%</strong></div>
      <span className="write-kicker">MAKING YOUR POCKET OS</span>
      <h1>{current.title}</h1>
      <p className="lede lede--compact">{current.detail}</p>
      <div className="phase-track">
        {phases.map((phase, index) => <div className={index < activePhase ? "is-done" : index === activePhase ? "is-active" : ""} key={phase}><span>{index < activePhase ? <Check size={12} /> : index + 1}</span><small>{phase}</small></div>)}
      </div>
      <div className="do-not-remove"><LoaderCircle size={16} className="spin" /><span>Keep the USB drive connected. SlickSlax will eject it when it’s safe.</span></div>
    </section>
  );
}

function DoneStep({ drive, onRestart }: { drive: UsbDrive; onRestart: () => void }) {
  return (
    <section className="screen done-screen">
      <div className="success-burst"><CloverMark /><span><Check size={20} /></span></div>
      <span className="write-kicker">ALL SLICK</span>
      <h1>Your pocket OS is <span>ready.</span></h1>
      <p className="lede lede--compact">{drive.vendor} {drive.name} is bootable, verified, and safe to remove.</p>
      <div className="boot-guide">
        <div><span>1</span><strong>Remove the USB</strong><small>It has been safely ejected.</small></div>
        <ChevronRight size={18} />
        <div><span>2</span><strong>Plug it into your PC</strong><small>Then power on or restart.</small></div>
        <ChevronRight size={18} />
        <div><span>3</span><strong>Open the boot menu</strong><small>Usually F11, F9, or Esc.</small></div>
      </div>
      <div className="screen-actions screen-actions--center">
        <button className="secondary-button" onClick={onRestart}><RotateCcw size={16} /> Make another</button>
        <a className="primary-button link-button" href="https://www.slax.org/starting.php" target="_blank" rel="noreferrer">Slax startup guide <ExternalLink size={15} /></a>
      </div>
    </section>
  );
}

function ConfirmDialog({ drive, erase, confirmation, onConfirmation, onCancel, onConfirm }: {
  drive: UsbDrive;
  erase: boolean;
  confirmation: string;
  onConfirmation: (value: string) => void;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  return (
    <div className="modal-backdrop" role="presentation" onMouseDown={(event) => event.target === event.currentTarget && onCancel()}>
      <div className="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="confirm-title">
        <button className="modal-close" onClick={onCancel} aria-label="Close"><X size={18} /></button>
        <div className="warning-icon"><AlertTriangle size={26} /></div>
        <span className="dialog-kicker">FINAL SAFETY CHECK</span>
        <h2 id="confirm-title">{erase ? "Erase this USB drive?" : "Write Slax to this drive?"}</h2>
        <p>{erase ? "SlickSlax will permanently erase every file on:" : "SlickSlax will replace any existing /slax folder on:"}</p>
        <div className="confirm-drive"><Usb size={22} /><div><strong>{drive.vendor} {drive.name}</strong><small>{formatBytes(drive.sizeBytes)} · {drive.device}</small></div></div>
        <label>Type <strong>SLAX</strong> to confirm<input autoFocus value={confirmation} onChange={(event) => onConfirmation(event.target.value.toUpperCase())} placeholder="SLAX" /></label>
        <div className="dialog-actions"><button className="secondary-button" onClick={onCancel}>Cancel</button><button className="danger-button" disabled={confirmation !== "SLAX"} onClick={onConfirm}><Zap size={16} /> {erase ? "Erase & make drive" : "Make Slax drive"}</button></div>
      </div>
    </div>
  );
}

export default App;
