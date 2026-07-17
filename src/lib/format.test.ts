import { describe, expect, it } from "vitest";
import { formatBytes, shortDeviceName } from "./format";

describe("formatBytes", () => {
  it("formats common drive sizes", () => {
    expect(formatBytes(32_000_000_000)).toBe("32 GB");
    expect(formatBytes(1_500_000_000)).toBe("1.5 GB");
  });

  it("handles empty values", () => expect(formatBytes(0)).toBe("0 B"));
});

describe("shortDeviceName", () => {
  it("shortens unix device names", () => expect(shortDeviceName("/dev/sdb")).toBe("sdb"));
  it("shortens windows physical drives", () => expect(shortDeviceName("\\\\.\\PHYSICALDRIVE2")).toBe("PHYSICALDRIVE2"));
});

