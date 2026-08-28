export interface DocHeading {
  id: string;
  text: string;
  level: number; // 2 for h2, 3 for h3
}

export interface DocItem {
  id: string; // e.g. "quickstart"
  slug: string; // e.g. "getting-started/quickstart"
  route: string; // e.g. "/docs/getting-started/quickstart"
  title: string;
  description?: string;
  rawContent: string;
  headings: DocHeading[];
  category: string;
  order?: number;
}

export interface DocCategory {
  id: string;
  label: string;
  items: DocItem[];
}

export interface SearchResultItem {
  item: DocItem;
  headingMatch?: string;
  snippet: string;
  score: number;
}
