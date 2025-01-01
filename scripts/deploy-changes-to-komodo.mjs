import { KomodoClient } from "komodo_client";
if (!process.env.KOMODO_API_KEY || !process.env.KOMODO_API_SECRET) {
  console.error("Missing KOMODO_API_KEY or KOMODO_API_SECRET");
  process.exit(1);
}

async function main() {
  const komodo = KomodoClient("https://komodo.evercode.se", {
    type: "api-key",
    params: {
      key: process.env.KOMODO_API_KEY,
      secret: process.env.KOMODO_API_SECRET,
    },
  });
  try {
    const res = await komodo.execute("RunProcedure", {
      procedure: "deploy-multi-host",
    });
    console.log("Res", res);
  } catch (e) {
    console.log("error", e);
  }
}
main();
