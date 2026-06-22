import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "STACC Documentation",
  description: "STACC agent configuration suite and documentation.",
};

const themeScript = `
(function () {
  try {
    var stored = window.localStorage.getItem("stacc-theme");
    var choice = stored === "light" || stored === "dark" || stored === "system" ? stored : "system";
    var systemDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    var resolved = choice === "system" ? (systemDark ? "dark" : "light") : choice;
    document.documentElement.dataset.theme = resolved;
    document.documentElement.dataset.themeChoice = choice;
    document.documentElement.style.colorScheme = resolved;
  } catch (error) {
    document.documentElement.dataset.theme = "light";
    document.documentElement.dataset.themeChoice = "system";
  }
})();
`;

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeScript }} />
      </head>
      <body>{children}</body>
    </html>
  );
}
