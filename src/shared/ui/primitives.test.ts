/// <reference types="node" />
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

// Vitest runs with the repo root as cwd; import.meta.url is an http URL in jsdom.
const root = process.cwd();
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8")) as {
  dependencies: Record<string, string>;
};

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return sourceFiles(path);
    return /\.(ts|tsx)$/.test(name) ? [path] : [];
  });
}

describe("primitives", () => {
  it("keep radix-ui out of the dependencies", () => {
    expect(pkg.dependencies).not.toHaveProperty("radix-ui");
    expect(pkg.dependencies).toHaveProperty("@base-ui/react");
  });

  it("import no radix primitive under src", () => {
    const offenders = sourceFiles(join(root, "src")).filter((file) =>
      /from ["'](radix-ui|@radix-ui\/)/.test(readFileSync(file, "utf8")),
    );
    expect(offenders).toEqual([]);
  });
});
