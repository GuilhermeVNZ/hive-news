import { describe, expect, it } from "vitest";

import { cn } from "@/lib/utils";

describe("cn", () => {
  it("merges class names and removes duplicates", () => {
    const result = cn("p-4", false && "hidden", "text-white", "p-4");
    const parts = result.split(" ");
    expect(parts).toContain("p-4");
    expect(parts).toContain("text-white");
    expect(new Set(parts).size).toBe(2);
  });

  it("handles conditional values", () => {
    const shouldHighlight = true;
    const result = cn("btn", shouldHighlight && "btn--highlighted");
    expect(result.split(" ")).toEqual(["btn", "btn--highlighted"]);
  });
});

