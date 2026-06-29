import type { Metadata } from "next";
import { LandingPage } from "@/components/landing-page";

export const metadata: Metadata = {
  title: "STACC | Agent Config Suite",
  description: "Generate a universal agent configuration suite for rules, agents, hooks, and editor-ready scaffolding.",
};

export default function Page() {
  return <LandingPage />;
}
