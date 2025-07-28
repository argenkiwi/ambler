import { assertEquals } from "https://deno.land/std@0.224.0/assert/mod.ts";

Deno.test("should run the main script and produce the correct output", async () => {
  const command = new Deno.Command("deno", {
    args: ["run", "--allow-read", "typescript/main.ts"],
    stdin: "piped",
    stdout: "piped",
  });
  const process = command.spawn();
  const writer = process.stdin.getWriter();
  await writer.write(new TextEncoder().encode("10\n"));
  writer.releaseLock();
  const output = await process.output();
  const outputString = new TextDecoder().decode(output.stdout);
  assertEquals(outputString.includes("Starting count from 10."), true);
});