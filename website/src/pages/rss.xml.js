import rss from "@astrojs/rss";
import { getCollection } from "astro:content";
import sanitizeHtml from "sanitize-html";
import MarkdownIt from "markdown-it";
const parser = new MarkdownIt();

export async function GET(context) {
  const blog = await getCollection("blogPosts");
  return rss({
    title: "Trieve’s Blog",
    description:
      "Sell more and answer every question with Conversational Discovery. Trieve uses GenAI to show your users what they're looking for every time",
    site: context.site,
    items: blog.map((post) => ({
      link: `/blog/${post.id}/`,
      content: sanitizeHtml(parser.render(post.body), {
        allowedTags: sanitizeHtml.defaults.allowedTags.concat(["img"]),
      }),
      ...post.data,
    })),
    customData: `<language>en-us</language>`,
  });
}
