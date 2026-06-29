import { notFound } from "next/navigation";
import { DocsShell } from "@/components/docs-shell";
import { getAllDocsPages } from "@/lib/docs";

type DocsPageProps = {
  params: Promise<{
    slug: string;
  }>;
};

export async function generateStaticParams() {
  const pages = await getAllDocsPages();

  return pages.map((page) => ({
    slug: page.slug,
  }));
}

export async function generateMetadata({ params }: DocsPageProps) {
  const { slug } = await params;
  const pages = await getAllDocsPages();
  const page = pages.find((docsPage) => docsPage.slug === slug);

  if (page) {
    return {
      title: `${page.frontmatter.title} | STACC Documentation`,
      description: page.frontmatter.description,
    };
  }

  return {
    title: "STACC Documentation",
  };
}

export default async function DocsPage({ params }: DocsPageProps) {
  const { slug } = await params;
  const pagesPromise = getAllDocsPages();

  const pages = await pagesPromise;
  const page = pages.find((docsPage) => docsPage.slug === slug);

  if (!page) {
    notFound();
  }

  return (
    <div className="docs-route">
      <DocsShell page={page} pages={pages} />
    </div>
  );
}
