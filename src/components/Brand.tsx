export function CloverMark({ small = false }: { small?: boolean }) {
  return (
    <svg className={small ? "clover clover--small" : "clover"} viewBox="0 0 72 72" aria-hidden="true">
      <path d="M36 33C11 31 11 7 26 7c7 0 10 6 10 13 0-7 3-13 10-13 15 0 15 24-10 26Z" />
      <path d="M39 36c2-25 26-25 26-10 0 7-6 10-13 10 7 0 13 3 13 10 0 15-24 15-26-10Z" />
      <path d="M36 39c25 2 25 26 10 26-7 0-10-6-10-13 0 7-3 13-10 13-15 0-15-24 10-26Z" />
      <path d="M33 36C31 61 7 61 7 46c0-7 6-10 13-10-7 0-13-3-13-10C7 11 31 11 33 36Z" />
      <circle cx="36" cy="36" r="6" className="clover__center" />
    </svg>
  );
}

export function Brand() {
  return (
    <div className="brand" aria-label="SlickSlax">
      <CloverMark small />
      <span>Slick</span><strong>Slax</strong>
      <em>alpha</em>
    </div>
  );
}

