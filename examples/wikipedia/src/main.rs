use std::env;
use htmd::HtmlToMarkdown;
use serde::{Deserialize, Serialize};
use urlencoding::encode;
use url_builder::URLBuilder;

#[derive(Debug, Serialize, Deserialize)]
struct SearchResult {
    title: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResponse {
    query: SearchResponse2,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResponse2 {
    search: Vec<SearchResult>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch command line arguments
    let args: Vec<String> = env::args().collect();

    // Expecting exactly one argument (the article title to search)
    if args.len() != 2 {
        eprintln!("Usage: {} <article_title>", args[0]);
        std::process::exit(1);
    }

    // Extract the article title from command line arguments
    let article_title = encode(&args[1]);

    // Make the search request to Wikipedia API
    let mut ub = URLBuilder::new();

    ub.set_protocol("https")
        .set_host("en.wikipedia.org")
        .add_route("w")
        .add_route("api.php")
        .add_param("action", "query")
        .add_param("format", "json")
        .add_param("list", "search")
        .add_param("utf8", "")
        .add_param("srsearch", &article_title);

    let api_url = ub.build();
    let response = reqwest::get(&api_url).await?.json::<SearchResponse>().await?;

    // Extract the first result (assuming there's at least one result)
    if let Some(first_result) = response.query.search.first() {
        let url = format!("https://en.wikipedia.org/wiki/{}", first_result.title.replace(" ", "_"));

        // Fetch the content of the article
        let article_content = reqwest::get(&url).await?.text().await?;
        let converter = HtmlToMarkdown::builder()
            .skip_tags(vec!["script", "style"])
            .build();

        println!("# {}", first_result.title);
        println!("{}", converter.convert(&article_content).unwrap());
    } else {
        println!("No results found for '{}'", article_title);
    }

    Ok(())
}
