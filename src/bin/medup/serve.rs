use std::error::Error;
use std::net::Ipv4Addr;
use std::path::Path;

use medup::config::{self, Config};
use medup::markdown::{self, Markdown};

use clap::ArgMatches;
use warp::filters::BoxedFilter;
use warp::hyper::StatusCode;
use warp::{Filter, Reply};

#[tokio::main]
pub async fn proc_serve(matches: &ArgMatches) {
    let (addr, port) = parse_ip_port(matches).unwrap();
    let cfg: Config = load_config(matches).unwrap();
    let dir = get_dir(matches, "dir");
    let sdir = get_dir(matches, "static-dir");

    println!(
        "---> the directory where markdown files are stored: \"{}\"",
        dir
    );
    println!(
        "---> the directory where static resources are stored: \"{}\"",
        sdir
    );

    // All filters are used to match requests
    let filters = static_filter(sdir.to_string())
        .or(articles_filter(cfg.clone(), dir.to_string()))
        .or(index_filter(cfg.clone(), dir.to_string()));

    println!("---> start to listen on address: \"{}:{}\"", addr, port);
    warp::serve(filters).run((addr, port)).await
}

// Get /static/*
fn static_filter(dir: String) -> BoxedFilter<(impl Reply,)> {
    warp::get()
        .and(warp::path("static"))
        .and(warp::fs::dir(dir))
        .with(warp::cors().allow_any_origin())
        .boxed()
}

// Get /:name (.e.g /demo.md)
fn articles_filter(cfg: Config, dir: String) -> BoxedFilter<(impl Reply,)> {
    warp::get()
        .and(warp::path::param::<String>())
        .and(warp::any().map(move || cfg.clone()))
        .and(warp::any().map(move || dir.to_string()))
        .map(|mut name: String, cfg: Config, dir: String| {
            if !name.ends_with(".md") {
                name.push_str(".md");
            }
            match Path::new(&dir).join(&name).to_str() {
                None => error_repsonse(
                    StatusCode::BAD_REQUEST,
                    format!(r#"failed to join the path: {}, {}"#, dir, name),
                ),
                Some(path) => match Markdown::new()
                    .config(cfg)
                    .path(path)
                    .map_mut(markdown::to_html)
                {
                    Err(e) => error_repsonse(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("failed to generate html from markdown: {}", e),
                    ),
                    Ok(v) => warp::reply::html(v).into_response(),
                },
            }
        })
        .with(warp::cors().allow_any_origin())
        .boxed()
}

// Get /, then read the index.md file
fn index_filter(cfg: Config, dir: String) -> BoxedFilter<(impl Reply,)> {
    warp::get()
        .and(warp::path::end())
        .and(warp::any().map(move || cfg.clone()))
        .and(warp::any().map(move || dir.to_string()))
        .map(
            |cfg: Config, dir: String| match Path::new(&dir).join("index.md").to_str() {
                None => error_repsonse(
                    StatusCode::BAD_REQUEST,
                    format!(r#"failed to join the path: {}, index.md"#, dir),
                ),
                Some(path) => match Markdown::new()
                    .config(cfg)
                    .path(path)
                    .map_mut(markdown::to_html)
                {
                    Err(e) => error_repsonse(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("failed to generate html from markdown: {}", e),
                    ),
                    Ok(v) => warp::reply::html(v).into_response(),
                },
            },
        )
        .with(warp::cors().allow_any_origin())
        .boxed()
}

fn get_dir<'get_dir>(matches: &'get_dir ArgMatches, name: &str) -> &'get_dir str {
    match matches.get_one::<String>(name) {
        None => ".",
        Some(path) => path,
    }
}

fn load_config(matches: &ArgMatches) -> Result<Config, Box<dyn Error>> {
    // read config path from cli
    let mut cfg = match matches.get_one::<String>("config-path") {
        None => Config::default(),
        Some(path) => config::read_config(path)
            .map_err(|e| (format!("failed to read config \"{}\": {}", path, e)))?,
    };
    if !medup::is_url(&cfg.css_href) {
        // add the static resource dir to css_href
        if let Some(href) = Path::new("/static")
            .join(&cfg.css_href)
            .to_str()
            .map(|s| s.to_string())
        {
            cfg.css_href = href;
        }
    }
    Ok(cfg)
}

fn parse_ip_port(matches: &ArgMatches) -> Result<(Ipv4Addr, u16), Box<dyn Error>> {
    // read listen addr from the args of command line
    let s = match matches.get_one::<String>("listen-addr") {
        None => ":8181",
        Some(s) => s,
    };

    let fields: Vec<&str> = s.split(':').collect();
    if fields.len() != 2 {
        return Err(format!("invalid address format: {}", s).into());
    }
    let ipaddr: Ipv4Addr = if fields[0].is_empty() {
        "0.0.0.0"
            .parse()
            .map_err(|e| format!("failed to parse ip addr 0.0.0.0: {}", e))?
    } else {
        fields[0]
            .parse()
            .map_err(|e| format!(r#"failed to parse ip addr "{}": {}"#, fields[0], e))?
    };
    let port: u16 = fields[1]
        .parse()
        .map_err(|e| format!("failed to parse port: {}: {}", fields[1], e))?;

    Ok((ipaddr, port))
}

fn error_repsonse(status_code: StatusCode, err_msg: String) -> warp::reply::Response {
    warp::http::Response::builder()
        .header("X-Powered-By", "Medup")
        .status(status_code)
        .body(err_msg)
        .into_response()
}
