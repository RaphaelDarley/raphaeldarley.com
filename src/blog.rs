use std::collections::HashMap;

use axum::{
    extract::Path,
    http::{header::TE, StatusCode},
    response::{Html, IntoResponse, Response},
};
use chrono::NaiveDate;
use include_dir::{include_dir, Dir, File};
use markdown::to_html;
use once_cell::sync::Lazy;
use serde::Serialize;
use tera::{Context, Tera};

static TEMPLATES: Lazy<Tera> = Lazy::new(|| Tera::new("content/templates/*").unwrap());

const BLOG_FILES: Dir = include_dir!("content/blog");

static BLOGS: Lazy<Vec<Blog>> = Lazy::new(generate_blogs);
static BLOG_MAP: Lazy<HashMap<&'static str, &'static Blog>> =
    Lazy::new(|| BLOGS.iter().map(|b| (b.title.as_str(), b)).collect());

#[derive(Serialize, Debug)]
struct Blog {
    title: String,
    date: NaiveDate,
    summary: Option<String>,
    md_content: String,
}

#[derive(Default, Debug)]
struct BlogBuilder {
    title: Option<String>,
    date: Option<NaiveDate>,
    summary: Option<String>,
    md_content: Option<String>,
}

impl BlogBuilder {
    fn finish(self) -> Option<Blog> {
        Some(Blog {
            title: self.title?,
            date: self.date?,
            summary: self.summary,
            md_content: self.md_content?,
        })
    }
}

pub async fn handler(Path(name): Path<String>) -> Response {
    match BLOG_MAP.get(name.as_str()) {
        Some(blog) => {
            let mut ctx = Context::new();
            ctx.insert("content", &to_html(&blog.md_content));
            let html = TEMPLATES.render("blog.html", &ctx).unwrap();
            Html(html).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn handler_root() -> impl IntoResponse {
    let mut ctx = Context::new();
    ctx.insert("blogs", &*BLOGS);

    let html = TEMPLATES
        .render("blog_list.html", &ctx)
        .expect("blog template should always succeed");
    Html(html)
}

fn parse_blog(file: &File) -> Blog {
    let mut acc = BlogBuilder::default();
    eprintln!("raw file: {:?}", file.contents_utf8().unwrap());
    let mut lines = file
        .contents_utf8()
        .unwrap()
        .lines()
        .skip_while(|l| l.chars().next().map(|c| c.is_whitespace()).unwrap_or(true))
        .skip_while(|l| l.starts_with('-'));

    for line in &mut lines {
        eprintln!("looking at line: {line}");
        if line.starts_with("-") {
            break;
        }
        if let Some((k, v)) = line.split_once(":") {
            eprintln!("k: {k}, v: {v}");
            match k {
                "date" => {
                    acc.date = Some(NaiveDate::parse_from_str(v, "%Y-%m-%d").unwrap());
                }
                "title" => acc.title = Some(v.trim().trim_matches('"').to_string()),
                "summary" => acc.summary = Some(v.trim().trim_matches('"').to_string()),
                _ => {}
            }
        }
    }

    let md_lines: Vec<&str> = lines.collect();

    acc.md_content = Some(md_lines.join("\n"));

    eprintln!("{:?}", acc);
    acc.finish().unwrap()
}

fn generate_blogs() -> Vec<Blog> {
    let mut blogs: Vec<Blog> = BLOG_FILES.files().map(parse_blog).collect();
    blogs.sort_by(|a, b| b.date.cmp(&a.date));

    blogs
}
