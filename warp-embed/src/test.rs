use super::*;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "data"]
struct Data;

#[tokio::test]
async fn test_embed_file() {
    let serve = embed(&Data);
    let res = warp::test::request().path("/foo.txt").reply(&serve).await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.headers().get("content-type").unwrap(), "text/plain");
    assert_eq!(res.body(), "foo");

    let res = warp::test::request().path("/bar.txt").reply(&serve).await;
    assert_eq!(res.status(), 404);

    let res = warp::test::request()
        .path("/bar/hoge.txt")
        .reply(&serve)
        .await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.headers().get("content-type").unwrap(), "text/plain");
    assert_eq!(res.body(), "hoge");

    let res = warp::test::request()
        .path("/index.html")
        .reply(&serve)
        .await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), include_str!("../data/index.html"));
    assert_eq!(res.headers().get("content-type").unwrap(), "text/html");

    let res = warp::test::request().path("/").reply(&serve).await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), include_str!("../data/index.html"));
    assert_eq!(res.headers().get("content-type").unwrap(), "text/html");

    let res = warp::test::request().path("/bar").reply(&serve).await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), include_str!("../data/bar/index.htm"));
    assert_eq!(res.headers().get("content-type").unwrap(), "text/html");
}
