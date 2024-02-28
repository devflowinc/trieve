use scraper::Html;

pub fn convert_html_to_text(html: &str) -> String {
    let dom = Html::parse_fragment(html);
    let text = dom.root_element().text().collect::<String>();
    text
}
