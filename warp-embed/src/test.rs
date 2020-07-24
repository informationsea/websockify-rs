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
    assert_eq!(res.body(), "foo");

    let res = warp::test::request().path("/bar.txt").reply(&serve).await;
    assert_eq!(res.status(), 404);

    let res = warp::test::request()
        .path("/bar/hoge.txt")
        .reply(&serve)
        .await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), "hoge");
}
