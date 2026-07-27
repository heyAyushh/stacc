import type { Metadata } from "next";
import { LandingPage } from "@/components/landing-page";

export const metadata: Metadata = {
  title: "STACC | One setup for every coding agent",
  description:
    "Curated skills, rules, stacks, MCP servers, and plugins for Cursor, Claude Code, Codex, OpenCode, and Amp.",
};

export default function Page() {
  return <LandingPage />;
}
