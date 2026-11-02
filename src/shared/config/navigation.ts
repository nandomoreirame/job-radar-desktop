export const PAGES = ["board", "profile", "resumes", "prep", "settings"] as const;
export type PageId = (typeof PAGES)[number];
